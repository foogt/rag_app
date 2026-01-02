use yew::prelude::*;
use yew_router::prelude::*;
use gloo_net::http::Request;
use shared::{Task, InventoryItem};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use chrono::{Utc, Duration, Local, TimeZone};
use std::collections::HashMap;
use regex::Regex;
use gloo::storage::{LocalStorage, Storage};
use web_sys::{HtmlInputElement, HtmlTextAreaElement, InputEvent};
use crate::Route;
use crate::types::TaskPreset;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = drawGanttChart)]
    fn draw_gantt_chart(data: JsValue, on_select: &JsValue);
}

// Struct to pass data to JS
#[derive(serde::Serialize)]
struct JsTask {
    id: String,
    name: String,
    resource: String,
    start: i64, // ms timestamp
    end: i64,
}

#[function_component(Home)]
pub fn home() -> Html {
    let tasks = use_state(Vec::new);
    let form_op_id = use_state(|| "".to_string());
    let form_user_id = use_state(|| "".to_string());
    let form_date = use_state(|| Local::now().format("%Y-%m-%d").to_string());
    let form_start_hour = use_state(|| "09".to_string());
    let form_start_min = use_state(|| "00".to_string());
    let form_dur_hour = use_state(|| "1".to_string());
    let form_dur_min = use_state(|| "00".to_string());
    let form_materials = use_state(|| HashMap::<String, String>::new());
    let ai_prompt = use_state(|| "".to_string());
    let ai_suggestion = use_state(|| "".to_string());
    let selected_task_id = use_state(|| None::<String>);
    let inventory = use_state(Vec::new);
    let presets = use_state(|| HashMap::<String, TaskPreset>::new());
    let pending_preset_update = use_state(|| None::<(String, TaskPreset)>);

    // Fetch tasks
    let fetch_tasks = {
        let tasks = tasks.clone();
        Callback::from(move |_| {
            let tasks = tasks.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let fetched: Vec<Task> = Request::get("http://localhost:8081/tasks")
                    .send().await.unwrap().json().await.unwrap();
                tasks.set(fetched);
            });
        })
    };

    {
        let fetch_tasks = fetch_tasks.clone();
        use_effect_with_deps(move |_| {
            fetch_tasks.emit(());
            || {}
        }, ());
    }

    // Fetch Inventory
    {
        let inventory = inventory.clone();
        use_effect_with_deps(move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let fetched: Vec<InventoryItem> = Request::get("http://localhost:8081/inventory")
                    .send().await.unwrap().json().await.unwrap();
                inventory.set(fetched);
            });
        }, ());
    }

    // Load Presets
    {
        let presets = presets.clone();
        use_effect_with_deps(move |_| {
            let loaded: HashMap<String, TaskPreset> = LocalStorage::get("task_presets").unwrap_or_default();
            presets.set(loaded);
        }, ());
    }

    // Handle selection from Chart
    let on_select_task = {
        let tasks = tasks.clone();
        let form_op_id = form_op_id.clone();
        let form_user_id = form_user_id.clone();
        let form_date = form_date.clone();
        let form_start_hour = form_start_hour.clone();
        let form_start_min = form_start_min.clone();
        let form_dur_hour = form_dur_hour.clone();
        let form_dur_min = form_dur_min.clone();
        let form_materials = form_materials.clone();
        let selected_task_id = selected_task_id.clone();

        Callback::from(move |id: String| {
            if let Some(task) = tasks.iter().find(|t| t.id == id) {
                selected_task_id.set(Some(id));
                form_op_id.set(task.operation_id.clone());
                form_user_id.set(task.user_id.clone());
                let local_dt = task.start_time.with_timezone(&Local);
                form_date.set(local_dt.format("%Y-%m-%d").to_string());
                form_start_hour.set(local_dt.format("%H").to_string());
                form_start_min.set(local_dt.format("%M").to_string());
                form_dur_hour.set((task.expected_duration_minutes / 60).to_string());
                form_dur_min.set((task.expected_duration_minutes % 60).to_string());
                form_materials.set(task.materials.clone());
            }
        })
    };

    // Effect to redraw chart when tasks change
    {
        let tasks = tasks.clone();
        let form_date = form_date.clone();
        use_effect_with_deps(move |(tasks, date_handle, on_select)| {
            let date_str = &**date_handle;
            let selected_date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok();

            let js_data: Vec<JsTask> = tasks.iter()
                .filter(|t| {
                    if let Some(d) = selected_date {
                        t.start_time.with_timezone(&Local).date_naive() == d
                    } else {
                        true
                    }
                })
                .map(|t| {
                let end_time = t.start_time + Duration::minutes(t.expected_duration_minutes);
                JsTask {
                    id: t.id.clone(),
                    name: t.operation_id.clone(),
                    resource: t.user_id.clone(),
                    start: t.start_time.timestamp_millis(),
                    end: end_time.timestamp_millis(),
                }
            }).collect();
            
            let closure_handle = if !js_data.is_empty() {
                let val = serde_wasm_bindgen::to_value(&js_data).unwrap();
                
                let cb = on_select.clone();
                let closure = wasm_bindgen::closure::Closure::<dyn FnMut(JsValue)>::new(move |id: JsValue| {
                    if let Some(id_str) = id.as_string() {
                        cb.emit(id_str);
                    }
                });
                
                draw_gantt_chart(val, closure.as_ref().unchecked_ref());
                Some(closure)
            } else {
                None
            };
            move || drop(closure_handle)
        }, (tasks, form_date, on_select_task));
    }

    // Sync form input with selection
    {
        let tasks = tasks.clone();
        let form_op_id = form_op_id.clone();
        let form_user_id = form_user_id.clone();
        let selected_task_id = selected_task_id.clone();

        use_effect_with_deps(move |(tasks, op_id, u_id)| {
            let found = tasks.iter().find(|t| t.operation_id == **op_id && t.user_id == **u_id);
            
            if let Some(task) = found {
                if *selected_task_id != Some(task.id.clone()) {
                    selected_task_id.set(Some(task.id.clone()));
                }
            } else if selected_task_id.is_some() {
                selected_task_id.set(None);
            }
            || {}
        }, (tasks, form_op_id, form_user_id));
    }

    // Add Task Handler
    let on_add = {
        let op_id = form_op_id.clone();
        let u_id = form_user_id.clone();
        let date = form_date.clone();
        let start_h = form_start_hour.clone();
        let start_m = form_start_min.clone();
        let dur_h = form_dur_hour.clone();
        let dur_m = form_dur_min.clone();
        let mat = form_materials.clone();
        let fetch = fetch_tasks.clone();
        
        Callback::from(move |_| {
            let d_str = (*date).clone();
            let h: u32 = start_h.parse().unwrap_or(9);
            let m: u32 = start_m.parse().unwrap_or(0);
            let naive_date = chrono::NaiveDate::parse_from_str(&d_str, "%Y-%m-%d")
                .unwrap_or_else(|_| Local::now().date_naive());
            let naive_dt = naive_date.and_hms_opt(h, m, 0).unwrap();
            let start_time = Local.from_local_datetime(&naive_dt).unwrap().with_timezone(&Utc);
            let dh: i64 = dur_h.parse().unwrap_or(1);
            let dm: i64 = dur_m.parse().unwrap_or(0);
            let duration = dh * 60 + dm;

            let task = Task::new(
                (*u_id).clone(),
                (*op_id).clone(),
                start_time,
                duration,
                (*mat).clone()
            );
            
            let fetch = fetch.clone();
            wasm_bindgen_futures::spawn_local(async move {
                Request::post("http://localhost:8081/tasks")
                    .json(&task).unwrap().send().await.unwrap();
                fetch.emit(());
            });
        })
    };

    // Update Task Handler
    let on_update = {
        let op_id = form_op_id.clone();
        let u_id = form_user_id.clone();
        let date = form_date.clone();
        let start_h = form_start_hour.clone();
        let start_m = form_start_min.clone();
        let dur_h = form_dur_hour.clone();
        let dur_m = form_dur_min.clone();
        let mat = form_materials.clone();
        let fetch = fetch_tasks.clone();
        let selected_task_id = selected_task_id.clone();
        
        Callback::from(move |_| {
            if let Some(id) = &*selected_task_id {
                let d_str = (*date).clone();
                let h: u32 = start_h.parse().unwrap_or(9);
                let m: u32 = start_m.parse().unwrap_or(0);
                let naive_date = chrono::NaiveDate::parse_from_str(&d_str, "%Y-%m-%d")
                    .unwrap_or_else(|_| Local::now().date_naive());
                let naive_dt = naive_date.and_hms_opt(h, m, 0).unwrap();
                let start_time = Local.from_local_datetime(&naive_dt).unwrap().with_timezone(&Utc);
                let dh: i64 = dur_h.parse().unwrap_or(1);
                let dm: i64 = dur_m.parse().unwrap_or(0);
                let duration = dh * 60 + dm;

                let mut task = Task::new(
                    (*u_id).clone(),
                    (*op_id).clone(),
                    start_time,
                    duration,
                    (*mat).clone()
                );
                task.id = id.clone();
                
                let fetch = fetch.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    Request::post("http://localhost:8081/tasks")
                        .json(&task).unwrap().send().await.unwrap();
                    fetch.emit(());
                });
            }
        })
    };

    // Delete Handler
    let on_delete = {
        let selected_task_id = selected_task_id.clone();
        let fetch = fetch_tasks.clone();
        let form_op_id = form_op_id.clone();
        let form_user_id = form_user_id.clone();
        let form_materials = form_materials.clone();
        
        Callback::from(move |_| {
            if let Some(id) = &*selected_task_id {
                let id = id.clone();
                let fetch = fetch.clone();
                let selected_task_id = selected_task_id.clone();
                let form_op_id = form_op_id.clone();
                let form_user_id = form_user_id.clone();
                let form_materials = form_materials.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    Request::delete(&format!("http://localhost:8081/tasks/{}", id))
                        .send().await.unwrap();
                    fetch.emit(());
                    selected_task_id.set(None);
                    form_op_id.set("".to_string());
                    form_user_id.set("".to_string());
                    form_materials.set(HashMap::new());
                });
            }
        })
    };

    // AI Suggest Handler
    let on_suggest = {
        let prompt = ai_prompt.clone();
        let suggestion = ai_suggestion.clone();
        
        Callback::from(move |_| {
            let prompt_text = (*prompt).clone();
            let suggestion = suggestion.clone();
            
            wasm_bindgen_futures::spawn_local(async move {
                let body = serde_json::json!({ "requirement": prompt_text });
                let resp = Request::post("http://localhost:8081/suggest")
                    .header("Content-Type", "application/json")
                    .body(body.to_string()).unwrap()
                    .send().await.unwrap();
                    
                if resp.ok() {
                    let json: serde_json::Value = resp.json().await.unwrap();
                    suggestion.set(format!("Time: {} \nReason: {}", 
                        json["suggested_start_time"], json["reason"]));
                }
            });
        })
    };

    // Helper to calculate leftover
    let calculate_leftover = |mat_name: &str, req_qty: &str, inventory: &Vec<InventoryItem>| -> String {
        let inv_item = inventory.iter().find(|i| i.name == mat_name);
        if let Some(item) = inv_item {
            // Simple regex to split number and unit
            let re = Regex::new(r"^([\d\.\+\-eE]+)\s*(.*)$").unwrap();
            
            let req_caps = re.captures(req_qty);
            
            if let Some(rc) = req_caps {
                let r_val: f64 = rc[1].parse().unwrap_or(0.0);
                let r_unit = rc.get(2).map_or("", |m| m.as_str()).trim();
                
                if r_unit == item.unit {
                    return format!("{:.2} {}", item.quantity - r_val, item.unit);
                } else {
                    return "Unit Mismatch".to_string();
                }
            }
            "Parse Error".to_string()
        } else {
            "Not in Inventory".to_string()
        }
    };

    // Material Table Handlers
    let add_material_row = {
        let form_materials = form_materials.clone();
        Callback::from(move |_| {
            let mut m = (*form_materials).clone();
            m.insert("New Material".to_string(), "0".to_string());
            form_materials.set(m);
        })
    };

    let update_material_key = {
        let form_materials = form_materials.clone();
        Callback::from(move |(old_key, new_key): (String, String)| {
            let mut m = (*form_materials).clone();
            if let Some(val) = m.remove(&old_key) {
                m.insert(new_key, val);
                form_materials.set(m);
            }
        })
    };

    let update_material_val = {
        let form_materials = form_materials.clone();
        Callback::from(move |(key, val): (String, String)| {
            let mut m = (*form_materials).clone();
            m.insert(key, val);
            form_materials.set(m);
        })
    };

    let delete_material = {
        let form_materials = form_materials.clone();
        Callback::from(move |key: String| {
            let mut m = (*form_materials).clone();
            m.remove(&key);
            form_materials.set(m);
        })
    };

    let on_date_change = {
        let form_date = form_date.clone();
        let tasks = tasks.clone();
        let selected_task_id = selected_task_id.clone();
        let form_op_id = form_op_id.clone();
        let form_user_id = form_user_id.clone();
        let form_start_hour = form_start_hour.clone();
        let form_start_min = form_start_min.clone();
        let form_dur_hour = form_dur_hour.clone();
        let form_dur_min = form_dur_min.clone();
        let form_materials = form_materials.clone();

        Callback::from(move |e: InputEvent| {
            let date_val = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();
            form_date.set(date_val.clone());

            let selected_date = chrono::NaiveDate::parse_from_str(&date_val, "%Y-%m-%d").ok();
            
            let mut tasks_on_date: Vec<Task> = tasks.iter()
                .filter(|t| {
                    if let Some(d) = selected_date {
                        t.start_time.with_timezone(&Local).date_naive() == d
                    } else {
                        false
                    }
                })
                .cloned()
                .collect();
            
            tasks_on_date.sort_by_key(|t| t.start_time);

            if let Some(first_task) = tasks_on_date.first() {
                selected_task_id.set(Some(first_task.id.clone()));
                form_op_id.set(first_task.operation_id.clone());
                form_user_id.set(first_task.user_id.clone());
                let local_dt = first_task.start_time.with_timezone(&Local);
                form_start_hour.set(local_dt.format("%H").to_string());
                form_start_min.set(local_dt.format("%M").to_string());
                form_dur_hour.set((first_task.expected_duration_minutes / 60).to_string());
                form_dur_min.set((first_task.expected_duration_minutes % 60).to_string());
                form_materials.set(first_task.materials.clone());
            } else {
                selected_task_id.set(None);
                form_op_id.set("".to_string());
                form_user_id.set("".to_string());
                form_materials.set(HashMap::new());
                form_start_hour.set("09".to_string());
                form_start_min.set("00".to_string());
                form_dur_hour.set("1".to_string());
                form_dur_min.set("00".to_string());
            }
        })
    };

    // Preset Handlers
    let on_op_input = {
        let form_op_id = form_op_id.clone();
        let form_dur_hour = form_dur_hour.clone();
        let form_dur_min = form_dur_min.clone();
        let form_materials = form_materials.clone();
        let presets = presets.clone();
        
        Callback::from(move |e: InputEvent| {
            let val = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();
            form_op_id.set(val.clone());
            
            if let Some(preset) = presets.get(&val) {
                form_dur_hour.set((preset.duration_minutes / 60).to_string());
                form_dur_min.set((preset.duration_minutes % 60).to_string());
                form_materials.set(preset.materials.clone());
            }
        })
    };

    let on_update_preset = {
        let form_op_id = form_op_id.clone();
        let form_dur_hour = form_dur_hour.clone();
        let form_dur_min = form_dur_min.clone();
        let form_materials = form_materials.clone();
        let pending_preset_update = pending_preset_update.clone();
        
        Callback::from(move |_| {
            let op_id = (*form_op_id).clone();
            if op_id.trim().is_empty() { return; }
            
            let dh: i64 = form_dur_hour.parse().unwrap_or(1);
            let dm: i64 = form_dur_min.parse().unwrap_or(0);
            let duration = dh * 60 + dm;
            
            let new_preset = TaskPreset {
                duration_minutes: duration,
                materials: (*form_materials).clone(),
            };
            
            pending_preset_update.set(Some((op_id, new_preset)));
        })
    };

    let on_confirm_preset = {
        let pending_preset_update = pending_preset_update.clone();
        let presets = presets.clone();
        
        Callback::from(move |_| {
            if let Some((op_id, new_preset)) = &*pending_preset_update {
                let mut current_presets = (*presets).clone();
                current_presets.insert(op_id.clone(), new_preset.clone());
                LocalStorage::set("task_presets", &current_presets).unwrap();
                presets.set(current_presets);
                pending_preset_update.set(None);
            }
        })
    };

    let on_cancel_preset_update = {
        let pending_preset_update = pending_preset_update.clone();
        Callback::from(move |_| pending_preset_update.set(None))
    };

    let render_preset_diff = |op_id: &str, new: &TaskPreset| {
        let old = presets.get(op_id);

        let content = if let Some(old) = old {
            let mut changes = Vec::new();
            let mut keys: Vec<&String> = old.materials.keys().chain(new.materials.keys()).collect();
            keys.sort();
            keys.dedup();
            for k in keys {
                let o = old.materials.get(k);
                let n = new.materials.get(k);
                if o != n {
                    match (o, n) {
                        (Some(ov), Some(nv)) => changes.push(format!("{}: {} -> {}", k, ov, nv)),
                        (Some(ov), None) => changes.push(format!("Removed {}: {}", k, ov)),
                        (None, Some(nv)) => changes.push(format!("Added {}: {}", k, nv)),
                        _ => {}
                    }
                }
            }
            html! {
                <>
                    if old.duration_minutes != new.duration_minutes {
                        <div>
                            <strong>{"Duration: "}</strong>
                            {format!("{}m -> {}m", old.duration_minutes, new.duration_minutes)}
                        </div>
                    }
                    if !changes.is_empty() {
                        <div>
                            <strong>{"Material Changes:"}</strong>
                            <ul class="mb-0">
                                {for changes.iter().map(|c| html!{<li>{c}</li>})}
                            </ul>
                        </div>
                    }
                    if old.duration_minutes == new.duration_minutes && changes.is_empty() {
                        <div>{"No changes detected."}</div>
                    }
                </>
            }
        } else {
            html! { <div>{"New Preset will be created."}</div> }
        };

        html! {
            <div class="alert alert-info mt-2">
                <h5>{format!("Update Preset for '{}'?", op_id)}</h5>
                {content}
                <div class="mt-2">
                    <button class="btn btn-success btn-sm me-2" onclick={on_confirm_preset.clone()}>{"Confirm"}</button>
                    <button class="btn btn-secondary btn-sm" onclick={on_cancel_preset_update.clone()}>{"Cancel"}</button>
                </div>
            </div>
        }
    };

    html! {
        <div class="row">
            <div class="col-12 mb-3">
                <Link<Route> to={Route::Home} classes="btn btn-outline-primary me-2">{"Gantt Chart"}</Link<Route>>
                <Link<Route> to={Route::Inventory} classes="btn btn-outline-success">{"Inventory"}</Link<Route>>
                <Link<Route> to={Route::Presets} classes="btn btn-outline-info ms-2">{"Task Presets"}</Link<Route>>
            </div>

            <div class="col-md-8">
                <h2>{"Production Timetable"}</h2>
                <div class="mb-3">
                    <label class="form-label me-2">{"Date:"}</label>
                    <input type="date" class="form-control d-inline-block w-auto" 
                        value={(*form_date).clone()}
                        oninput={on_date_change} />
                </div>
                <div id="chart_div" style="width: 100%; height: 400px; border: 1px solid #ccc;"></div>
                <button onclick={let fetch = fetch_tasks.clone(); move |_| fetch.emit(())} class="btn btn-secondary mt-2">{"Refresh Data"}</button>
                
                <datalist id="inventory-list">
                    {for inventory.iter().map(|i| html! { <option value={i.name.clone()} /> })}
                </datalist>
                <datalist id="preset-list">
                    {for presets.keys().map(|k| html! { <option value={k.clone()} /> })}
                </datalist>

                <h4 class="mt-4">{"Material Requirements"}</h4>
                <table class="table table-bordered">
                    <thead>
                        <tr>
                            <th>{"Material Name"}</th>
                            <th>{"Required Quantity"}</th>
                            <th>{"Est. Leftover"}</th>
                            <th>{"Action"}</th>
                        </tr>
                    </thead>
                    <tbody>
                        {for form_materials.iter().map(|(name, qty)| {
                            let name_c = name.clone();
                            let name_c2 = name.clone();
                            let name_delete = name.clone();
                            let update_key = update_material_key.clone();
                            let update_val = update_material_val.clone();
                            let delete = delete_material.clone();
                            let leftover = calculate_leftover(name, qty, &inventory);
                            
                            let is_valid = inventory.iter().any(|i| i.name == *name);
                            let inv_item = inventory.iter().find(|i| i.name == *name);
                            
                            // Extract numeric value for input if unit exists
                            let (num_val, unit_label) = if let Some(item) = inv_item {
                                let re = Regex::new(r"^([\d\.\+\-eE]+)").unwrap();
                                let val = re.captures(qty).map(|c| c[1].to_string()).unwrap_or_default();
                                (val, Some(item.unit.clone()))
                            } else {
                                (qty.clone(), None)
                            };

                            html! {
                                <tr key={name.clone()}>
                                    <td>
                                        <input class={classes!("form-control", if !is_valid { "is-invalid" } else { "" })} 
                                            list="inventory-list"
                                            value={name.clone()} 
                                            onchange={Callback::from(move |e: Event| {
                                                let val = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();
                                                update_key.emit((name_c.clone(), val));
                                            })} />
                                    </td>
                                    <td>
                                        {if let Some(unit) = unit_label {
                                            let unit_cb = unit.clone();
                                            html! {
                                                <div class="input-group">
                                                    <input type="number" class="form-control" value={num_val} 
                                                        onchange={Callback::from(move |e: Event| {
                                                            let val = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();
                                                            update_val.emit((name_c2.clone(), format!("{} {}", val, unit_cb)));
                                                        })} />
                                                    <span class="input-group-text">{unit}</span>
                                                </div>
                                            }
                                        } else {
                                            html! {
                                                <input class="form-control" value={qty.clone()} 
                                                    onchange={Callback::from(move |e: Event| {
                                                        let val = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();
                                                        update_val.emit((name_c2.clone(), val));
                                                    })} />
                                            }
                                        }}
                                    </td>
                                    <td>{leftover}</td>
                                    <td>
                                        <button class="btn btn-danger btn-sm" onclick={Callback::from(move |_| delete.emit(name_delete.clone()))}>{"X"}</button>
                                    </td>
                                </tr>
                            }
                        })}
                    </tbody>
                </table>
                <button class="btn btn-sm btn-primary" onclick={add_material_row}>{"+ Add Material"}</button>
            </div>
            
            <div class="col-md-4">
                <div class="card p-3 mb-3">
                    <h4>{if selected_task_id.is_some() { "Edit Operation" } else { "Add Operation" }}</h4>
                    <input class="form-control mb-2" placeholder="User ID" 
                        value={(*form_user_id).clone()} 
                        oninput={
                            let form_user_id = form_user_id.clone();
                            Callback::from(move |e: InputEvent| form_user_id.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value()))
                        } />
                        
                    <input class="form-control mb-2" placeholder="Operation ID" 
                        list="preset-list"
                        value={(*form_op_id).clone()} 
                        oninput={on_op_input} />

                    <div class="row g-2 mb-2">
                        <div class="col">
                            <label class="form-label">{"Start (HH:MM)"}</label>
                            <div class="input-group">
                                <input type="number" class="form-control" placeholder="HH" min="0" max="23"
                                    value={(*form_start_hour).clone()}
                                    oninput={Callback::from(move |e: InputEvent| form_start_hour.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value()))} />
                                <input type="number" class="form-control" placeholder="MM" min="0" max="59"
                                    value={(*form_start_min).clone()}
                                    oninput={Callback::from(move |e: InputEvent| form_start_min.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value()))} />
                            </div>
                        </div>
                        <div class="col">
                            <label class="form-label">{"Duration (HH:MM)"}</label>
                            <div class="input-group">
                                <input type="number" class="form-control" placeholder="HH" min="0"
                                    value={(*form_dur_hour).clone()}
                                    oninput={Callback::from(move |e: InputEvent| form_dur_hour.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value()))} />
                                <input type="number" class="form-control" placeholder="MM" min="0" max="59"
                                    value={(*form_dur_min).clone()}
                                    oninput={Callback::from(move |e: InputEvent| form_dur_min.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value()))} />
                            </div>
                        </div>
                    </div>

                    <div class="d-flex gap-2">
                        <button onclick={on_add} class="btn btn-primary flex-grow-1" 
                            disabled={
                                form_user_id.trim().is_empty() || 
                                form_op_id.trim().is_empty() ||
                                !form_materials.keys().all(|k| inventory.iter().any(|i| i.name == *k))
                            }>
                            {"Add Task"}
                        </button>
                        if selected_task_id.is_some() {
                            <button onclick={on_delete} class="btn btn-danger">{"Delete"}</button>
                        }
                    </div>
                </div>

                <div class="card p-3">
                    <h4>{"AI Assistant"}</h4>
                    <textarea class="form-control mb-2" placeholder="Describe new request..."
                        value={(*ai_prompt).clone()}
                        oninput={Callback::from(move |e: InputEvent| ai_prompt.set(e.target_unchecked_into::<web_sys::HtmlTextAreaElement>().value()))}>
                    </textarea>
                    <button onclick={on_suggest} class="btn btn-success">{"Suggest Schedule"}</button>
                    <pre class="mt-2 p-2 bg-light">{(*ai_suggestion).clone()}</pre>
                </div>
                <div class="mt-3">
                    <button onclick={on_update} class="btn btn-warning w-100 mb-2"
                        disabled={
                            selected_task_id.is_none() ||
                            form_user_id.trim().is_empty() || 
                            form_op_id.trim().is_empty() ||
                            !form_materials.keys().all(|k| inventory.iter().any(|i| i.name == *k))
                        }>
                        {"Update Selected Task"}
                    </button>
                    
                    <button onclick={on_update_preset} class="btn btn-info w-100"
                        disabled={form_op_id.trim().is_empty()}>
                        {"Update Task Preset"}
                    </button>
                    
                    if let Some((op_id, new_preset)) = &*pending_preset_update {
                        {render_preset_diff(op_id, new_preset)}
                    }
                </div>
            </div>
        </div>
    }
}
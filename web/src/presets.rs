use yew::prelude::*;
use yew_router::prelude::*;
use std::collections::HashMap;
use gloo::storage::{LocalStorage, Storage};
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, InputEvent};
use crate::Route;
use crate::types::TaskPreset;

#[function_component(PresetsPage)]
pub fn presets_page() -> Html {
    let presets = use_state(|| {
        let loaded: HashMap<String, TaskPreset> = LocalStorage::get("task_presets").unwrap_or_default();
        loaded
    });

    {
        let presets = presets.clone();
        use_effect_with_deps(move |presets| {
            LocalStorage::set("task_presets", &**presets).unwrap();
            || {}
        }, presets);
    }

    let update_duration = {
        let presets = presets.clone();
        Callback::from(move |(op_id, val): (String, String)| {
            let mut current = (*presets).clone();
            if let Some(preset) = current.get_mut(&op_id) {
                if let Ok(d) = val.parse::<i64>() {
                    preset.duration_minutes = d;
                    presets.set(current);
                }
            }
        })
    };

    let add_material = {
        let presets = presets.clone();
        Callback::from(move |op_id: String| {
            let mut current = (*presets).clone();
            if let Some(preset) = current.get_mut(&op_id) {
                let mut i = 1;
                let mut name = "New Material".to_string();
                while preset.materials.contains_key(&name) {
                    name = format!("New Material {}", i);
                    i += 1;
                }
                preset.materials.insert(name, "0".to_string());
                presets.set(current);
            }
        })
    };

    let update_material_key = {
        let presets = presets.clone();
        Callback::from(move |(op_id, old_key, new_key): (String, String, String)| {
            let mut current = (*presets).clone();
            if let Some(preset) = current.get_mut(&op_id) {
                if !preset.materials.contains_key(&new_key) {
                    if let Some(val) = preset.materials.remove(&old_key) {
                        preset.materials.insert(new_key, val);
                        presets.set(current);
                    }
                }
            }
        })
    };

    let update_material_val = {
        let presets = presets.clone();
        Callback::from(move |(op_id, key, val): (String, String, String)| {
            let mut current = (*presets).clone();
            if let Some(preset) = current.get_mut(&op_id) {
                preset.materials.insert(key, val);
                presets.set(current);
            }
        })
    };

    let delete_material = {
        let presets = presets.clone();
        Callback::from(move |(op_id, key): (String, String)| {
            let mut current = (*presets).clone();
            if let Some(preset) = current.get_mut(&op_id) {
                preset.materials.remove(&key);
                presets.set(current);
            }
        })
    };

    let delete_preset = {
        let presets = presets.clone();
        Callback::from(move |op_id: String| {
            let mut current = (*presets).clone();
            current.remove(&op_id);
            presets.set(current);
        })
    };

    let filter_op_name = use_state(|| String::new());
    let filter_material = use_state(|| String::new());

    let filter_op = (*filter_op_name).to_lowercase();
    let filter_mat = (*filter_material).to_lowercase();

    let mut sorted_keys: Vec<String> = presets.iter()
        .filter(|(key, preset)| {
            let matches_op = key.to_lowercase().contains(&filter_op);
            let matches_mat = if filter_mat.is_empty() {
                true
            } else {
                preset.materials.keys().any(|k| k.to_lowercase().contains(&filter_mat))
            };
            matches_op && matches_mat
        })
        .map(|(k, _)| k.clone())
        .collect();
    sorted_keys.sort();

    html! {
        <div class="container">
            <div class="mb-3">
                <Link<Route> to={Route::Home} classes="btn btn-outline-primary me-2">{"Gantt Chart"}</Link<Route>>
                <Link<Route> to={Route::Inventory} classes="btn btn-outline-success me-2">{"Inventory"}</Link<Route>>
                <Link<Route> to={Route::Presets} classes="btn btn-outline-info">{"Task Presets"}</Link<Route>>
            </div>
            <h2>{"Operation Presets"}</h2>
            
            <div class="row mb-3">
                <div class="col">
                    <input 
                        type="text" 
                        class="form-control" 
                        placeholder="Filter by Operation Name" 
                        value={(*filter_op_name).clone()}
                        oninput={
                            let filter_op_name = filter_op_name.clone();
                            Callback::from(move |e: InputEvent| {
                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                filter_op_name.set(input.value());
                            })
                        }
                    />
                </div>
                <div class="col">
                    <input 
                        type="text" 
                        class="form-control" 
                        placeholder="Filter by Material Name" 
                        value={(*filter_material).clone()}
                        oninput={
                            let filter_material = filter_material.clone();
                            Callback::from(move |e: InputEvent| {
                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                filter_material.set(input.value());
                            })
                        }
                    />
                </div>
            </div>
            
            {for sorted_keys.iter().map(|key| {
                let preset = presets.get(key).unwrap();
                let key_c = key.clone();
                let key_add = key.clone();
                let key_del_preset = key.clone();
                
                let mut materials: Vec<_> = preset.materials.iter().collect();
                materials.sort_by_key(|k| k.0);

                html! {
                    <div class="card mb-3">
                        <div class="card-header d-flex justify-content-between align-items-center">
                            <h5 class="mb-0">{key}</h5>
                            <button class="btn btn-danger btn-sm" onclick={
                                let delete_preset = delete_preset.clone();
                                let k = key_del_preset.clone();
                                move |_| delete_preset.emit(k.clone())
                            }>{"Delete Preset"}</button>
                        </div>
                        <div class="card-body">
                            <div class="mb-3 row">
                                <label class="col-sm-2 col-form-label">{"Duration (min)"}</label>
                                <div class="col-sm-10">
                                    <input type="number" class="form-control" 
                                        value={preset.duration_minutes.to_string()} 
                                        oninput={
                                            let update_duration = update_duration.clone();
                                            let k = key_c.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let val = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();
                                                update_duration.emit((k.clone(), val));
                                            })
                                        }
                                    />
                                </div>
                            </div>
                            
                            <h6>{"Materials"}</h6>
                            <table class="table table-bordered table-sm">
                                <thead>
                                    <tr>
                                        <th>{"Material Name"}</th>
                                        <th>{"Quantity"}</th>
                                        <th>{"Action"}</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {
                                        materials.into_iter().map(|(m_name, m_qty)| {
                                            let m_name_c = m_name.clone();
                                            let m_name_val = m_name.clone();
                                            let m_name_del = m_name.clone();
                                            let k_key = key_c.clone();
                                            let k_val = key_c.clone();
                                            let k_del = key_c.clone();
                                            let update_material_key = update_material_key.clone();
                                            let update_material_val = update_material_val.clone();
                                            let delete_material = delete_material.clone();

                                            html! {
                                                <tr key={m_name.clone()}>
                                                    <td>
                                                        <input class="form-control form-control-sm" 
                                                            value={m_name.clone()} 
                                                            onchange={Callback::from(move |e: Event| {
                                                                let val = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();
                                                                update_material_key.emit((k_key.clone(), m_name_c.clone(), val));
                                                            })}
                                                        />
                                                    </td>
                                                    <td>
                                                        <input class="form-control form-control-sm" 
                                                            value={m_qty.clone()} 
                                                            oninput={Callback::from(move |e: InputEvent| {
                                                                let val = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();
                                                                update_material_val.emit((k_val.clone(), m_name_val.clone(), val));
                                                            })}
                                                        />
                                                    </td>
                                                    <td>
                                                        <button class="btn btn-danger btn-sm" onclick={
                                                            Callback::from(move |_| delete_material.emit((k_del.clone(), m_name_del.clone())))
                                                        }>{"X"}</button>
                                                    </td>
                                                </tr>
                                            }
                                        }).collect::<Html>()
                                    }
                                </tbody>
                            </table>
                            <button class="btn btn-secondary btn-sm" onclick={
                                let add_material = add_material.clone();
                                let k = key_add.clone();
                                move |_| add_material.emit(k.clone())
                            }>{"+ Add Material"}</button>
                        </div>
                    </div>
                }
            })}
        </div>
    }
}
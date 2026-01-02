use yew::prelude::*;
use yew_router::prelude::*;
use gloo_net::http::Request;
use shared::InventoryItem;
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, InputEvent};
use crate::Route;

#[function_component(Inventory)]
pub fn inventory_page() -> Html {
    let inventory = use_state(Vec::new);
    let new_name = use_state(|| "".to_string());
    let new_qty = use_state(|| "".to_string());
    let new_unit = use_state(|| "".to_string());

    let fetch_inv = {
        let inventory = inventory.clone();
        Callback::from(move |_| {
            let inventory = inventory.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let fetched: Vec<InventoryItem> = Request::get("http://localhost:8081/inventory")
                    .send().await.unwrap().json().await.unwrap();
                inventory.set(fetched);
            });
        })
    };

    {
        let fetch_inv = fetch_inv.clone();
        use_effect_with_deps(move |_| {
            fetch_inv.emit(());
            || {}
        }, ());
    }

    let on_add = {
        let name = new_name.clone();
        let qty = new_qty.clone();
        let unit = new_unit.clone();
        let fetch = fetch_inv.clone();
        Callback::from(move |_| {
            let item = InventoryItem {
                name: (*name).clone(),
                quantity: (*qty).parse().unwrap_or(0.0),
                unit: (*unit).clone(),
            };
            let fetch = fetch.clone();
            wasm_bindgen_futures::spawn_local(async move {
                Request::post("http://localhost:8081/inventory")
                    .json(&item).unwrap().send().await.unwrap();
                fetch.emit(());
            });
        })
    };

    html! {
        <div class="container">
            <div class="mb-3">
                <Link<Route> to={Route::Home} classes="btn btn-outline-primary me-2">{"Gantt Chart"}</Link<Route>>
                <Link<Route> to={Route::Inventory} classes="btn btn-outline-success">{"Inventory"}</Link<Route>>
            </div>
            <h2>{"Inventory Management"}</h2>
            <div class="row mb-3">
                <div class="col">
                    <input class="form-control" placeholder="Item Name" value={(*new_name).clone()} 
                        oninput={Callback::from(move |e: InputEvent| new_name.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value()))} />
                </div>
                <div class="col">
                    <input type="number" class="form-control" placeholder="Quantity" value={(*new_qty).clone()} 
                        oninput={Callback::from(move |e: InputEvent| new_qty.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value()))} />
                </div>
                <div class="col">
                    <input class="form-control" placeholder="Unit (e.g. kg)" value={(*new_unit).clone()} 
                        oninput={Callback::from(move |e: InputEvent| new_unit.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value()))} />
                </div>
                <div class="col">
                    <button class="btn btn-primary" onclick={on_add}>{"Add Item"}</button>
                </div>
            </div>
            <ul class="list-group">
                {for inventory.iter().map(|item| html! {
                    <li class="list-group-item d-flex justify-content-between align-items-center">
                        {format!("{}: {} {}", item.name, item.quantity, item.unit)}
                    </li>
                })}
            </ul>
        </div>
    }
}
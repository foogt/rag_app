mod home;
mod inventory;
mod presets;
mod types;

use yew::prelude::*;
use yew_router::prelude::*;
use home::Home;
use inventory::Inventory;
use presets::PresetsPage;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/inventory")]
    Inventory,
    #[at("/presets")]
    Presets,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home /> },
        Route::Inventory => html! { <Inventory /> },
        Route::Presets => html! { <PresetsPage /> },
    }
}

#[function_component(Main)]
fn main_app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<Main>::new().render();
}
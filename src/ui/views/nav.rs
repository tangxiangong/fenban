use crate::ui::{LOGO, route::Route};
use dioxus::prelude::*;

#[component]
pub fn NavBar() -> Element {
    rsx! {
        div { class: "h-screen flex flex-col",
            div { class: "navbar bg-base-100 shadow-sm flex-shrink-0",
                div { class: "navbar-start",
                    div { class: "tooltip tooltip-right",
                        div { class: "tooltip-content bg-base-100 text-red-400", "Home" }
                        Link { to: Route::Home {},
                            img { src: LOGO, width: "60px" }
                        }
                    }
                }
                div { class: "navbar-center", "" }
            }
        }
    }
}

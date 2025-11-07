use crate::ui::views::UpdateWindow;
use dioxus::prelude::*;

pub mod components;
pub mod route;
pub mod views;

// icon assets
pub static LOGO: Asset = asset!("assets/transparent_logo.png");
pub static ERR_ICON: Asset = asset!("assets/icons/error.svg");
pub static OK_ICON: Asset = asset!("assets/icons/ok.svg");
pub static COPY_ICON: Asset = asset!("assets/icons/copy.svg");
pub static ADD_ICON: Asset = asset!("assets/icons/add.svg");
pub static CANCEL_ICON: Asset = asset!("assets/icons/cancel.svg");
pub static DELETE_ICON: Asset = asset!("assets/icons/delete.svg");
pub static DETAILS_ICON: Asset = asset!("assets/icons/details.svg");

// tailwindcss
pub static TAILWINDCSS: Asset = asset!("assets/tailwind.css");

#[component]
pub fn App() -> Element {
    let mut show_check_window = use_signal(|| false);

    rsx! {
        document::Stylesheet { href: TAILWINDCSS }
        Router::<route::Route> {}
        if show_check_window() {
            div { class: "modal modal-open",
                div { class: "modal-box w-1/2",
                    UpdateWindow { show_window: show_check_window }
                }
                div {
                    class: "modal-backdrop",
                    onclick: move |_| show_check_window.set(false),
                }
            }
        }
    }
}

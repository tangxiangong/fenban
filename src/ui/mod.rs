use crate::ui::views::{Home, UpdateWindow};
use dioxus::{desktop::use_muda_event_handler, prelude::*};

pub mod components;
pub mod views;

// icon assets
pub static LOGO: Asset = asset!("assets/logo.png");
pub static ICON_DOCUMENT: Asset = asset!("assets/icons/document.svg");
pub static ICON_INFO: Asset = asset!("assets/icons/info.svg");
pub static ICON_ERROR: Asset = asset!("assets/icons/error.svg");
pub static ICON_SUCCESS: Asset = asset!("assets/icons/success.svg");
pub static ICON_WARNING: Asset = asset!("assets/icons/warning.svg");
pub static ICON_TRASH: Asset = asset!("assets/icons/trash.svg");
pub static ICON_SETTINGS: Asset = asset!("assets/icons/settings.svg");
pub static ICON_HISTORY: Asset = asset!("assets/icons/history.svg");

// tailwindcss
pub static TAILWINDCSS: Asset = asset!("assets/tailwind.css");

#[component]
pub fn App() -> Element {
    let mut show_update_window = use_signal(|| false);

    // Handle menu events
    use_muda_event_handler(move |event| {
        if event.id().0 == "check_update" {
            show_update_window.set(true);
        }
    });

    rsx! {
        document::Stylesheet { href: TAILWINDCSS }
        Home {}

        // Update modal
        if show_update_window() {
            div {
                class: "modal modal-open",
                onclick: move |_| show_update_window.set(false),
                div {
                    class: "modal-box max-w-md",
                    onclick: move |e| e.stop_propagation(),
                    UpdateWindow { show_window: show_update_window }
                }
                div { class: "modal-backdrop" }
            }
        }
    }
}

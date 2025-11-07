use crate::ui::views::{Home, UpdateWindow};
#[cfg(not(any(target_os = "ios", target_os = "android")))]
use dioxus::desktop::use_muda_event_handler;
use dioxus::prelude::*;

pub mod components;
pub mod views;

// icon assets
pub static LOGO: Asset = asset!("assets/transparent_logo.png");

// tailwindcss
pub static TAILWINDCSS: Asset = asset!("assets/tailwind.css");

#[component]
pub fn App() -> Element {
    let mut show_update_window = use_signal(|| false);

    // Handle menu events
    #[cfg(not(any(target_os = "ios", target_os = "android")))]
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

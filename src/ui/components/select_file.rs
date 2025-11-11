use crate::ui::{CSV, EXCEL};
use dioxus::prelude::*;

#[component]
pub fn SelectFileView(on_select: EventHandler<()>) -> Element {
    rsx! {
        div { class: "text-center py-12",
            div { class: "mb-6",
                div { class: "flex items-center justify-center",
                    img { class: "w-24 h-24 mr-4", src: EXCEL }
                    img { class: "w-24 h-24", src: CSV }
                }
            }
            h2 { class: "text-2xl font-bold mb-4", "选择文件" }
            p { class: "text-base-content/70 mb-6", "支持 .csv .xls 和 .xlsx 格式" }
            button {
                class: "btn btn-primary btn-lg",
                onclick: move |_| on_select.call(()),
                "选择文件"
            }
        }
    }
}

use dioxus::prelude::*;

#[component]
pub fn SelectFileView(on_select: EventHandler<()>) -> Element {
    rsx! {
        div { class: "text-center py-12",
            div { class: "mb-6",
                svg {
                    class: "w-24 h-24 mx-auto text-primary",
                    xmlns: "http://www.w3.org/2000/svg",
                    fill: "none",
                    view_box: "0 0 24 24",
                    stroke: "currentColor",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z",
                    }
                }
            }
            h2 { class: "text-2xl font-bold mb-4", "选择 Excel 文件" }
            p { class: "text-base-content/70 mb-6", "支持 .xls 和 .xlsx 格式" }
            button {
                class: "btn btn-primary btn-lg",
                onclick: move |_| on_select.call(()),
                "选择文件"
            }
        }
    }
}

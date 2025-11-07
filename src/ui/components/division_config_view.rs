use dioxus::prelude::*;

#[component]
pub fn DivisionConfigView(
    num_classes: Signal<usize>,
    on_start: EventHandler<()>,
    on_back: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            h2 { class: "text-2xl font-bold mb-6", "分班参数设置" }

            div { class: "space-y-6 mb-8",
                div { class: "form-control w-16",
                    label { class: "label",
                        span { class: "label-text font-medium", "班级数量" }
                        input {
                            r#type: "number",
                            class: "input input-bordered",
                            value: "{num_classes}",
                            min: "2",
                            max: "100",
                            oninput: move |evt| {
                                if let Ok(val) = evt.value().parse::<usize>() && (2..=100).contains(&val) {
                                    num_classes.set(val);
                                }
                            },
                        }
                        label { class: "label",
                            span { class: "label-text-alt text-base-content/60",
                                "💡 根据学生总数合理设置"
                            }
                        }
                    }

                }

                div { class: "alert alert-info",
                    svg {
                        class: "stroke-current shrink-0 h-6 w-6",
                        xmlns: "http://www.w3.org/2000/svg",
                        fill: "none",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z",
                        }
                    }
                    div {
                        h3 { class: "font-bold", "分班约束" }
                        ul { class: "list-disc list-inside text-sm mt-2",
                            li { "平均分差值 ≤ 1 分" }
                            li { "性别比例差 ≤ 10%" }
                            li { "班级人数差 ≤ 5 人" }
                        }
                    }
                }
            }

            div { class: "flex justify-between",
                button {
                    class: "btn btn-outline",
                    onclick: move |_| on_back.call(()),
                    "返回"
                }
                button {
                    class: "btn btn-primary",
                    onclick: move |_| on_start.call(()),
                    "开始分班"
                }
            }
        }
    }
}

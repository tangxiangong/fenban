use super::types::{ColumnMapping, ColumnType};
use dioxus::prelude::*;

#[component]
pub fn ColumnConfigView(
    column_mappings: Signal<Vec<ColumnMapping>>,
    on_confirm: EventHandler<()>,
    on_back: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            h2 { class: "text-2xl font-bold mb-4", "配置数据列" }
            p { class: "text-base-content/70 mb-6",
                "请为每一列指定其类型（系统已自动识别）"
            }

            div { class: "overflow-x-auto mb-6",
                table { class: "table table-zebra w-full",
                    thead {
                        tr {
                            th { "列名" }
                            th { "列号" }
                            th { "类型" }
                        }
                    }
                    tbody {
                        for (idx , mapping) in column_mappings.read().iter().enumerate() {
                            tr { key: "{idx}",
                                td { class: "font-medium", "{mapping.name}" }
                                td { "{mapping.index + 1}" }
                                td {
                                    select {
                                        class: "select select-bordered select-sm w-25",
                                        value: "{mapping.column_type.to_string()}",
                                        onchange: move |evt| {
                                            let new_type = ColumnType::from_string(&evt.value());
                                            let mut mappings = column_mappings.write();
                                            if let Some(m) = mappings.get_mut(idx) {
                                                m.column_type = new_type;
                                            }
                                        },
                                        option { value: "name", "姓名" }
                                        option { value: "gender", "性别" }
                                        option { value: "student_id", "学号" }
                                        option { value: "subject", "科目成绩" }
                                        option { value: "total", "总成绩" }
                                        option { value: "extra", "保留列" }
                                        option { value: "ignore", "忽略" }
                                    }
                                }
                            }
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
                    onclick: move |_| on_confirm.call(()),
                    "下一步"
                }
            }
        }
    }
}

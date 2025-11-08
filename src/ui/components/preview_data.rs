use dioxus::prelude::*;

#[component]
pub fn PreviewDataView(
    headers: Signal<Vec<String>>,
    data: Signal<Vec<Vec<String>>>,
    on_confirm: EventHandler<()>,
    on_back: EventHandler<()>,
) -> Element {
    let mut current_page = use_signal(|| 0);
    let page_size = 8;

    let total_rows = data.read().len();
    let total_pages = total_rows.div_ceil(page_size);
    let start_idx = *current_page.read() * page_size;
    let end_idx = (start_idx + page_size).min(total_rows);

    rsx! {
        div {
            h2 { class: "text-2xl font-bold mb-4", "数据预览" }
            p { class: "text-base-content/70 mb-4",
                "共 {total_rows} 行数据，确认无误后继续"
            }

            div { class: "overflow-x-auto mb-4",
                table { class: "table table-zebra table-sm w-full",
                    thead {
                        tr {
                            th { class: "bg-base-300", "#" }
                            for (idx , header) in headers.read().iter().enumerate() {
                                th { key: "{idx}", class: "bg-base-300", "{header}" }
                            }
                        }
                    }
                    tbody {
                        for (row_idx , row) in data.read().iter().enumerate().skip(start_idx).take(end_idx - start_idx) {
                            tr { key: "{row_idx}",
                                td { class: "font-semibold", "{row_idx + 1}" }
                                for (col_idx , cell) in row.iter().enumerate() {
                                    td { key: "{col_idx}", "{cell}" }
                                }
                            }
                        }
                    }
                }
            }

            // 分页控件
            if total_pages > 1 {
                div { class: "flex justify-center mb-6",
                    div { class: "join",
                        button {
                            class: "join-item btn btn-sm",
                            disabled: *current_page.read() == 0,
                            onclick: move |_| {
                                let page = *current_page.read();
                                if page > 0 {
                                    current_page.set(page - 1);
                                }
                            },
                            "«"
                        }
                        // 显示页码按钮
                        {
                            let current = *current_page.read();
                            let mut pages_to_show = Vec::new();

                            // 始终显示第一页
                            pages_to_show.push(0);

                            // 显示当前页附近的页码
                            let start = if current > 2 { current - 1 } else { 1 };
                            let end = (current + 2).min(total_pages - 1);

                            for i in start..=end {
                                if i > 0 && i < total_pages - 1 && !pages_to_show.contains(&i) {
                                    pages_to_show.push(i);
                                }
                            }

                            // 始终显示最后一页
                            if total_pages > 1 && !pages_to_show.contains(&(total_pages - 1)) {
                                pages_to_show.push(total_pages - 1);
                            }

                            pages_to_show.sort();

                            let mut elements = Vec::new();
                            for (idx, &page) in pages_to_show.iter().enumerate() {
                                // 如果页码不连续，添加省略号
                                if idx > 0 && page > pages_to_show[idx - 1] + 1 {
                                    elements.push(rsx! {
                                        button { class: "join-item btn btn-sm btn-disabled", "..." }
                                    });
                                }

                                let is_current = page == current;
                                elements.push(rsx! {
                                    button {
                                        class: if is_current { "join-item btn btn-sm btn-active" } else { "join-item btn btn-sm" },
                                        onclick: move |_| current_page.set(page),
                                        "{page + 1}"
                                    }
                                });
                            }

                            rsx! {
                                {elements.into_iter()}
                            }
                        }
                        button {
                            class: "join-item btn btn-sm",
                            disabled: *current_page.read() >= total_pages - 1,
                            onclick: move |_| {
                                let page = *current_page.read();
                                if page < total_pages - 1 {
                                    current_page.set(page + 1);
                                }
                            },
                            "»"
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
                    "确认并继续"
                }
            }
        }
    }
}

use crate::{
    core::model::{Class, Student},
    ui::{
        ICON_WARNING,
        components::types::{ColumnMapping, ColumnType},
    },
};
use dioxus::prelude::*;

#[component]
pub fn ResultsView(
    classes: Signal<Vec<Class>>,
    summary: Option<String>,
    column_mappings: Signal<Vec<ColumnMapping>>,
    on_export: EventHandler<String>,
    on_restart: EventHandler<()>,
    on_back: EventHandler<()>,
) -> Element {
    let mut current_page = use_signal(|| 0);
    let page_size = 8;

    // 获取科目列表
    let subjects: Vec<String> = column_mappings
        .read()
        .iter()
        .filter(|m| m.column_type == ColumnType::Subject)
        .map(|m| m.name.clone())
        .collect();

    // 获取额外字段
    let extra_fields: Vec<String> = column_mappings
        .read()
        .iter()
        .filter(|m| m.column_type == ColumnType::Extra)
        .map(|m| m.name.clone())
        .collect();

    // 计算分页
    let classes_read = classes.read();
    let all_students: Vec<(usize, &Student)> = classes_read
        .iter()
        .flat_map(|class| {
            class
                .students
                .iter()
                .map(move |student| (class.id, student))
        })
        .collect();

    let total_rows = all_students.len();
    let total_pages = total_rows.div_ceil(page_size);
    let start_idx = *current_page.read() * page_size;
    let end_idx = (start_idx + page_size).min(total_rows);

    // 选项卡状态
    let mut active_tab = use_signal(|| "statistics");

    // 检查是否有数据
    if classes_read.is_empty() {
        return rsx! {
            div { class: "text-center py-12",
                div { class: "mb-6",
                    img {
                        class: "w-16 h-16 mx-auto text-warning",
                        src: ICON_WARNING,
                    }
                }
                h2 { class: "text-2xl font-bold mb-4", "没有分班数据" }
                p { class: "text-base-content/70 mb-6", "分班结果为空，请重新尝试" }
                button {
                    class: "btn btn-primary",
                    onclick: move |_| on_restart.call(()),
                    "重新开始"
                }
            }
        };
    }

    rsx! {
        div { class: "space-y-6",
            // 统计信息
            if let Some(summary_text) = summary {
                div { class: "alert alert-info",
                    div { class: "w-full",
                        h3 { class: "font-bold mb-2", "分班统计" }
                        pre { class: "whitespace-pre-wrap text-sm", "{summary_text}" }
                    }
                }
            }

            // 选项卡
            div { class: "tabs tabs-boxed bg-base-200 p-1",
                a {
                    class: if *active_tab.read() == "statistics" { "tab tab-active" } else { "tab" },
                    onclick: move |_| active_tab.set("statistics"),
                    "班级统计"
                }
                a {
                    class: if *active_tab.read() == "students" { "tab tab-active" } else { "tab" },
                    onclick: move |_| active_tab.set("students"),
                    "学生分班结果"
                }
            }

            // 选项卡内容
            match &**active_tab.read() {
                "statistics" => rsx! {
                    // 班级统计表
                    div {
                        div { class: "overflow-x-auto",
                            table { class: "table table-sm",
                                thead {
                                    tr {
                                        th { "班级" }
                                        th { "人数" }
                                        th { "男生" }
                                        th { "女生" }
                                        th { "男生比例" }
                                        for subject in subjects.iter() {
                                            th { key: "{subject}", "{subject}" }
                                        }
                                        th { "平均总分" }
                                    }
                                }
                                tbody {
                                    for class in classes.read().iter() {
                                        tr { key: "{class.id}",
                                            td { class: "font-semibold", "班级 {class.id + 1}" }
                                            td { "{class.students.len()}" }
                                            td { "{class.male_count()}" }
                                            td { "{class.female_count()}" }
                                            td { "{class.gender_ratio() * 100.0:.1}%" }
                                            for subject in subjects.iter() {
                                                td { key: "{subject}", "{class.avg_subject_score(subject):.2}" }
                                            }
                                            td { "{class.avg_total_score():.2}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                "students" => rsx! {
                    // 学生分班结果表
                    div {
                        div { class: "overflow-x-auto",
                            table { class: "table table-zebra table-sm",
                                thead {
                                    tr {
                                        th { "#" }
                                        th { "班级" }
                                        th { "姓名" }
                                        th { "性别" }
                                        for field in extra_fields.iter() {
                                            th { key: "{field}", "{field}" }
                                        }
                                        for subject in subjects.iter() {
                                            th { key: "{subject}", "{subject}" }
                                        }
                                        th { "总分" }
                                    }
                                }
                                tbody {
                                    for (idx , (class_id , student)) in all_students.iter().enumerate().skip(start_idx).take(end_idx - start_idx) {
                                        tr { key: "{idx}",
                                            td { "{idx + 1}" }
                                            td { class: "font-semibold", "{class_id + 1}" }
                                            td { "{student.name}" }
                                            td {
                                                if student.gender == crate::core::model::Gender::Male {
                                                    "男"
                                                } else {
                                                    "女"
                                                }
                                            }
                                            for field in extra_fields.iter() {
                                                td { key: "{field}",
                                                    {student.extra_fields.get(field).map(|s| s.as_str()).unwrap_or("")}
                                                }
                                            }
                                            for subject in subjects.iter() {
                                                td { key: "{subject}", "{student.scores.get(subject).unwrap_or(&0.0):.1}" }
                                            }
                                            td { "{student.total_score:.2}" }
                                        }
                                    }
                                }
                            }
                        }
                        if total_pages > 1 {
                            div { class: "flex justify-center mt-4",
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
                                    {
                                        let current = *current_page.read();
                                        let mut pages_to_show = Vec::new();
                                        pages_to_show.push(0);



                                        let start = if current > 2 { current - 1 } else { 1 };
                                        let end = (current + 2).min(total_pages - 1);

                                        for i in start..=end {
                                            if i > 0 && i < total_pages - 1 && !pages_to_show.contains(&i) {
                                                pages_to_show.push(i);
                                            }
                                        }

                                        if total_pages > 1 && !pages_to_show.contains(&(total_pages - 1)) {
                                            pages_to_show.push(total_pages - 1);
                                        }

                                        pages_to_show.sort();

                                        let mut elements = Vec::new();
                                        for (idx, &page) in pages_to_show.iter().enumerate() {
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
                    }
                },
                _ => rsx! {
                    div {}
                },
            }

            // 操作按钮
            div { class: "flex justify-center gap-4",
                button {
                    class: "btn btn-outline",
                    onclick: move |_| on_back.call(()),
                    "← 上一步"
                }
                button {
                    class: "btn btn-primary",
                    onclick: move |_| on_export.call("xlsx".to_string()),
                    "导出 Excel"
                }
                button {
                    class: "btn btn-secondary",
                    onclick: move |_| on_export.call("csv".to_string()),
                    "导出 CSV"
                }
                button {
                    class: "btn btn-outline",
                    onclick: move |_| on_restart.call(()),
                    "重新开始"
                }
            }
        }
    }
}

use crate::core::history::{HistoryManager, HistoryRecord};
use crate::ui::{ICON_SETTINGS, ICON_TRASH};
use dioxus::prelude::*;

// 折叠版本的历史记录组件
#[component]
pub fn HistoryCollapsedView(refresh_trigger: u32) -> Element {
    let mut records = use_signal(Vec::<HistoryRecord>::new);
    let mut error_msg = use_signal(|| None::<String>);
    let mut show_config = use_signal(|| None::<HistoryRecord>);

    // 加载历史记录 - 当 refresh_trigger 改变时重新加载
    use_effect(move || {
        // 使用 refresh_trigger 以触发重新执行
        let _ = refresh_trigger;
        spawn(async move {
            match HistoryManager::new() {
                Ok(manager) => match manager.load() {
                    Ok(history) => records.set(history),
                    Err(e) => error_msg.set(Some(format!("加载历史记录失败: {}", e))),
                },
                Err(e) => error_msg.set(Some(format!("初始化历史管理器失败: {}", e))),
            }
        });
    });

    let clear_history = move |_| {
        spawn(async move {
            if let Ok(manager) = HistoryManager::new() {
                if let Err(e) = manager.clear() {
                    error_msg.set(Some(format!("清空历史记录失败: {}", e)));
                } else {
                    records.set(Vec::new());
                }
            }
        });
    };

    rsx! {
        div { class: "space-y-4",
            if let Some(err) = error_msg.read().as_ref() {
                div { class: "alert alert-error alert-sm",
                    span { class: "text-xs", "{err}" }
                }
            }

            if records.read().is_empty() {
                div { class: "text-center py-8 text-base-content/60 text-sm",
                    p { "暂无历史记录" }
                }
            } else {
                div { class: "flex justify-end mb-2",
                    button {
                        class: "btn btn-xs btn-ghost gap-1",
                        onclick: clear_history,
                        img { class: "w-3 h-3", src: ICON_TRASH }
                        "清空全部"
                    }
                }
                div { class: "overflow-x-auto max-h-96 overflow-y-auto",
                    table { class: "table table-zebra table-xs",
                        thead {
                            tr {
                                th { class: "text-xs", "时间" }
                                th { class: "text-xs", "输入文件" }
                                th { class: "text-xs", "输出文件" }
                                th { class: "text-xs", "班级数" }
                                th { class: "text-xs", "学生数" }
                                th { class: "text-xs", "操作" }
                            }
                        }
                        tbody {
                            for record in records.read().iter() {
                                tr { key: "{record.timestamp}",
                                    td { class: "text-xs", "{record.timestamp}" }
                                    td {
                                        a {
                                            class: "link link-primary text-xs truncate max-w-xs block",
                                            href: "#",
                                            title: "{record.input_path}",
                                            onclick: {
                                                let path = record.input_path.clone();
                                                move |e| {
                                                    e.prevent_default();
                                                    let path = path.clone();
                                                    spawn(async move {
                                                        let _ = opener::open(&path);
                                                    });
                                                }
                                            },
                                            {
                                                std::path::Path::new(&record.input_path)
                                                    .file_name()
                                                    .and_then(|n| n.to_str())
                                                    .unwrap_or(&record.input_path)
                                            }
                                        }
                                    }
                                    td {
                                        if let Some(output) = &record.output_path {
                                            a {
                                                class: "link link-primary text-xs truncate max-w-xs block",
                                                href: "#",
                                                title: "{output}",
                                                onclick: {
                                                    let path = output.clone();
                                                    move |e| {
                                                        e.prevent_default();
                                                        let path = path.clone();
                                                        spawn(async move {
                                                            let _ = opener::open(&path);
                                                        });
                                                    }
                                                },
                                                {
                                                    std::path::Path::new(output)
                                                        .file_name()
                                                        .and_then(|n| n.to_str())
                                                        .unwrap_or(output)
                                                }
                                            }
                                        } else {
                                            span { class: "text-xs text-base-content/50",
                                                "-"
                                            }
                                        }
                                    }
                                    td { class: "text-xs", "{record.num_classes}" }
                                    td { class: "text-xs", "{record.num_students}" }
                                    td {
                                        div { class: "flex gap-1",
                                            button {
                                                class: "btn btn-xs btn-ghost",
                                                title: "查看配置",
                                                onclick: {
                                                    let rec = record.clone();
                                                    move |_| {
                                                        show_config.set(Some(rec.clone()));
                                                    }
                                                },
                                                img {
                                                    class: "w-3 h-3",
                                                    src: ICON_SETTINGS,
                                                }
                                            }
                                            button {
                                                class: "btn btn-xs btn-ghost text-error",
                                                title: "删除",
                                                onclick: {
                                                    let timestamp = record.timestamp.clone();
                                                    move |_| {
                                                        let ts = timestamp.clone();
                                                        spawn(async move {
                                                            if let Ok(manager) = HistoryManager::new() {
                                                                if manager.delete(&ts).is_ok() {
                                                                    // 重新加载列表
                                                                    if let Ok(history) = manager.load() {
                                                                        records.set(history);
                                                                    }
                                                                } else {
                                                                    error_msg.set(Some("删除失败".to_string()));
                                                                }
                                                            }
                                                        });
                                                    }
                                                },
                                                img {
                                                    class: "w-3 h-3",
                                                    src: ICON_TRASH,
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 配置详情模态框
        if let Some(record) = show_config.read().as_ref() {
            div {
                class: "modal modal-open",
                onclick: move |_| show_config.set(None),
                div {
                    class: "modal-box",
                    onclick: move |e| e.stop_propagation(),
                    h3 { class: "font-bold text-lg mb-4", "分班配置参数" }
                    div { class: "space-y-2 text-sm",
                        div { class: "grid grid-cols-2 gap-2",
                            div { class: "font-semibold", "时间:" }
                            div { "{record.timestamp}" }
                            div { class: "font-semibold", "班级数:" }
                            div { "{record.num_classes}" }
                            div { class: "font-semibold", "学生数:" }
                            div { "{record.num_students}" }
                            div { class: "font-semibold", "输出格式:" }
                            div { {record.format.to_uppercase()} }
                        }
                        div { class: "divider text-xs", "优化参数" }
                        div { class: "grid grid-cols-2 gap-2 text-xs",
                            div { class: "font-semibold", "总分最大差值:" }
                            div { "{record.optimization_params.max_score_diff}" }
                            div { class: "font-semibold", "单科最大差值:" }
                            div { "{record.optimization_params.max_subject_score_diff}" }
                            div { class: "font-semibold", "班级人数最大差值:" }
                            div { "{record.optimization_params.max_class_size_diff}" }
                            div { class: "font-semibold", "性别比例最大差值:" }
                            div { "{record.optimization_params.max_gender_ratio_diff}" }
                            div { class: "font-semibold", "初始温度:" }
                            div { "{record.optimization_params.initial_temperature}" }
                            div { class: "font-semibold", "冷却速率:" }
                            div { "{record.optimization_params.cooling_rate}" }
                        }
                    }
                    div { class: "modal-action",
                        button {
                            class: "btn btn-sm",
                            onclick: move |_| show_config.set(None),
                            "关闭"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn HistoryView(on_close: EventHandler<()>, on_open_file: EventHandler<String>) -> Element {
    let mut records = use_signal(Vec::<HistoryRecord>::new);
    let mut error_msg = use_signal(|| None::<String>);

    // 加载历史记录
    use_effect(move || {
        spawn(async move {
            match HistoryManager::new() {
                Ok(manager) => match manager.load() {
                    Ok(history) => records.set(history),
                    Err(e) => error_msg.set(Some(format!("加载历史记录失败: {}", e))),
                },
                Err(e) => error_msg.set(Some(format!("初始化历史管理器失败: {}", e))),
            }
        });
    });

    let clear_history = move |_| {
        spawn(async move {
            if let Ok(manager) = HistoryManager::new() {
                if let Err(e) = manager.clear() {
                    error_msg.set(Some(format!("清空历史记录失败: {}", e)));
                } else {
                    records.set(Vec::new());
                }
            }
        });
    };

    rsx! {
        div { class: "space-y-4",
            div { class: "flex justify-between items-center",
                h2 { class: "text-2xl font-bold", "历史记录" }
                div { class: "flex gap-2",
                    if !records.read().is_empty() {
                        button {
                            class: "btn btn-sm btn-ghost",
                            onclick: clear_history,
                            "🗑️ 清空"
                        }
                    }
                    button {
                        class: "btn btn-sm btn-ghost",
                        onclick: move |_| on_close.call(()),
                        "✕"
                    }
                }
            }

            if let Some(err) = error_msg.read().as_ref() {
                div { class: "alert alert-error",
                    span { "{err}" }
                }
            }

            if records.read().is_empty() {
                div { class: "text-center py-12 text-base-content/60",
                    p { "暂无历史记录" }
                }
            } else {
                div { class: "overflow-x-auto",
                    table { class: "table table-zebra table-sm",
                        thead {
                            tr {
                                th { "时间" }
                                th { "输入文件" }
                                th { "班级数" }
                                th { "学生数" }
                                th { "输出格式" }
                                th { "操作" }
                            }
                        }
                        tbody {
                            for record in records.read().iter() {
                                tr { key: "{record.timestamp}",
                                    td { class: "text-xs", "{record.timestamp}" }
                                    td {
                                        div {
                                            class: "tooltip tooltip-right",
                                            "data-tip": "{record.input_path}",
                                            span { class: "text-xs truncate max-w-xs block",
                                                {
                                                    std::path::Path::new(&record.input_path)
                                                        .file_name()
                                                        .and_then(|n| n.to_str())
                                                        .unwrap_or(&record.input_path)
                                                }
                                            }
                                        }
                                    }
                                    td { "{record.num_classes}" }
                                    td { "{record.num_students}" }
                                    td {
                                        span { class: "badge badge-sm", {record.format.to_uppercase()} }
                                    }
                                    td {
                                        div { class: "flex gap-1",
                                            button {
                                                class: "btn btn-xs btn-ghost",
                                                title: "打开输入文件",
                                                onclick: {
                                                    let path = record.input_path.clone();
                                                    move |_| {
                                                        let path = path.clone();
                                                        spawn(async move {
                                                            let _ = opener::open(&path);
                                                        });
                                                    }
                                                },
                                                "📂"
                                            }
                                            if let Some(output) = &record.output_path {
                                                button {
                                                    class: "btn btn-xs btn-ghost",
                                                    title: "打开输出文件",
                                                    onclick: {
                                                        let path = output.clone();
                                                        move |_| {
                                                            let path = path.clone();
                                                            spawn(async move {
                                                                let _ = opener::open(&path);
                                                            });
                                                        }
                                                    },
                                                    "📊"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

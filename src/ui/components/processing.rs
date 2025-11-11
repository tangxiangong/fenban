use crate::core::algorithm::OptimizationParams;
use dioxus::prelude::*;

#[component]
pub fn ProcessingView(
    num_classes: ReadSignal<usize>,
    optimization_params: ReadSignal<OptimizationParams>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut show_params = use_signal(|| false);

    rsx! {
            div { class: "text-center py-16 px-4",
                // 主要加载动画
                div { class: "flex justify-center mb-6",
                    div { class: "loading loading-spinner loading-lg text-primary" }
                }

                // 标题和提示
                h2 { class: "text-3xl font-bold mb-3", "正在分班中..." }
                p { class: "text-base-content/70 mb-6", "算法正在优化班级分配，请稍候" }

                // 配置参数展示（可折叠）
                div { class: "mt-12 max-w-2xl mx-auto",
                    div { class: "collapse collapse-arrow bg-base-200 shadow-xl",
                        input {
                            r#type: "checkbox",
                            checked: *show_params.read(),
                            onchange: move |evt| {
                                show_params.set(evt.checked());
                            },
                        }
                        div { class: "collapse-title text-lg font-bold",
                            "当前配置参数"
                        }
                        div { class: "collapse-content",
                            div { class: "pt-4",

                            // 基础参数
                            div { class: "grid grid-cols-1 md:grid-cols-2 gap-4 mb-4",
                                div { class: "stat bg-base-100 rounded-lg p-4",
                                    div { class: "stat-title text-xs", "班级数量" }
                                    div { class: "stat-value text-2xl text-primary", "{num_classes}" }
                                    div { class: "stat-desc", "个班级" }
                                }
                                div { class: "stat bg-base-100 rounded-lg p-4",
                                    div { class: "stat-title text-xs", "初始温度" }
                                    div { class: "stat-value text-2xl text-secondary",
                                        "{optimization_params.read().initial_temperature:.0}"
                                    }
                                    div { class: "stat-desc", "模拟退火起始温度" }
                                }
                            }

                            // 硬约束阈值
                            div { class: "divider text-xs font-bold", "硬约束阈值" }
                            div { class: "grid grid-cols-2 gap-3 text-sm mb-4",
                                div { class: "flex justify-between items-center bg-base-100 rounded px-3 py-2",
                                    span { class: "text-base-content/70", "平均分差值" }
                                    span { class: "font-mono font-bold",
                                        "≤ {optimization_params.read().max_score_diff} 分"
                                    }
                                }
                                div { class: "flex justify-between items-center bg-base-100 rounded px-3 py-2",
                                    span { class: "text-base-content/70", "单科分差值" }
                                    span { class: "font-mono font-bold",
                                        "≤ {optimization_params.read().max_subject_score_diff} 分"
                                    }
                                }
                                div { class: "flex justify-between items-center bg-base-100 rounded px-3 py-2",
                                    span { class: "text-base-content/70", "性别比例差" }
                                    span { class: "font-mono font-bold",
                                        "≤ {(optimization_params.read().max_gender_ratio_diff * 100.0):.1}%"
                                    }
                                }
                                div { class: "flex justify-between items-center bg-base-100 rounded px-3 py-2",
                                    span { class: "text-base-content/70", "班级人数差" }
                                    span { class: "font-mono font-bold",
                                        "≤ {optimization_params.read().max_class_size_diff} 人"
                                    }
                                }
                            }

                            // 惩罚权重
                            div { class: "divider text-xs font-bold", "惩罚权重" }
                            div { class: "grid grid-cols-1 gap-2 text-xs mb-4",
                                div { class: "flex justify-between items-center bg-base-100 rounded px-3 py-2",
                                    span { class: "text-base-content/70", "总分差值惩罚" }
                                    span { class: "font-mono",
                                        "{optimization_params.read().total_score_penalty_weight:.0}"
                                    }
                                }
                                div { class: "flex justify-between items-center bg-base-100 rounded px-3 py-2",
                                    span { class: "text-base-content/70", "科目分差值惩罚" }
                                    span { class: "font-mono",
                                        "{optimization_params.read().subject_score_penalty_weight:.0}"
                                    }
                                }
                                div { class: "flex justify-between items-center bg-base-100 rounded px-3 py-2",
                                    span { class: "text-base-content/70", "性别比例惩罚" }
                                    span { class: "font-mono",
                                        "{optimization_params.read().gender_ratio_penalty_weight:.0}"
                                    }
                                }
                            }

                            // 优化权重
                            div { class: "divider text-xs font-bold", "优化权重" }
                            div { class: "grid grid-cols-3 gap-2 text-xs mb-4",
                                div { class: "flex flex-col items-center bg-base-100 rounded px-2 py-2",
                                    span { class: "text-base-content/70 mb-1", "总分方差" }
                                    span { class: "font-mono font-bold",
                                        "{optimization_params.read().total_variance_weight:.0}"
                                    }
                                }
                                div { class: "flex flex-col items-center bg-base-100 rounded px-2 py-2",
                                    span { class: "text-base-content/70 mb-1", "性别方差" }
                                    span { class: "font-mono font-bold",
                                        "{optimization_params.read().gender_variance_weight:.0}"
                                    }
                                }
                                div { class: "flex flex-col items-center bg-base-100 rounded px-2 py-2",
                                    span { class: "text-base-content/70 mb-1", "科目方差" }
                                    span { class: "font-mono font-bold",
                                        "{optimization_params.read().subject_variance_weight:.0}"
                                    }
                                }
                            }

                            // 算法参数
                            div { class: "divider text-xs font-bold", "算法参数" }
                            div { class: "grid grid-cols-2 gap-2 text-xs",
                                div { class: "flex justify-between items-center bg-base-100 rounded px-3 py-2",
                                    span { class: "text-base-content/70", "冷却速率" }
                                    span { class: "font-mono",
                                        "{optimization_params.read().cooling_rate:.5}"
                                    }
                                }
                                div { class: "flex justify-between items-center bg-base-100 rounded px-3 py-2",
                                    span { class: "text-base-content/70", "惩罚幂次" }
                                    span { class: "font-mono", "{optimization_params.read().penalty_power}" }
                                }
                                div { class: "flex justify-between items-center bg-base-100 rounded px-3 py-2",
                                    span { class: "text-base-content/70", "良好解阈值" }
                                    span { class: "font-mono",
                                        "{optimization_params.read().good_solution_threshold:.1}"
                                    }
                                }
                                div { class: "flex justify-between items-center bg-base-100 rounded px-3 py-2",
                                    span { class: "text-base-content/70", "重加热次数" }
                                    span { class: "font-mono",
                                        "{optimization_params.read().reheat_after_iterations}"
                                    }
                                }
                            }
                        }
                    }
                }

                // 取消按钮
                div { class: "mt-8",
                    button {
                        class: "btn btn-outline btn-error",
                        onclick: move |_| on_cancel.call(()),
                        "取消分班"
                    }
                }
            }
        }
    }
}

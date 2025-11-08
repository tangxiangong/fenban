use crate::{core::algorithm::OptimizationParams, ui::ICON_INFO};
use dioxus::prelude::*;

#[component]
pub fn DivisionConfigView(
    num_classes: Signal<usize>,
    optimization_params: Signal<OptimizationParams>,
    on_start: EventHandler<()>,
    on_back: EventHandler<()>,
) -> Element {
    // 高级参数展开状态
    let mut show_advanced = use_signal(|| false);

    // 预设方案选择
    let mut preset = use_signal(|| "default".to_string());

    rsx! {
        div {
            h2 { class: "text-2xl font-bold mb-6", "分班参数设置" }

            div { class: "space-y-6 mb-8",
                // 基础参数
                div { class: "form-control w-full max-w-xs",
                    label { class: "label",
                        span { class: "label-text font-medium", "班级数量" }
                        input {
                            r#type: "number",
                            class: "input input-bordered w-16 mr-2",
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
                                "💡 根据学生总数合理设置，建议每班 30-50 人"
                            }
                        }
                    }
                }
                // 预设方案选择
                div { class: "form-control w-full max-w-xs",
                    label { class: "label",
                        span { class: "label-text font-medium", "优化方案" }
                    }
                    select {
                        class: "select select-bordered w-40 mx-2",
                        value: "{preset}",
                        onchange: move |evt| {
                            let value = evt.value();
                            preset.set(value.clone());
                            match value.as_str() {
                                "relaxed" => optimization_params.set(OptimizationParams::relaxed()),
                                "strict" => optimization_params.set(OptimizationParams::strict()),
                                _ => optimization_params.set(OptimizationParams::default()),
                            }
                        },
                        option { value: "default", "默认（推荐）" }
                        option { value: "relaxed", "宽松（更快速）" }
                        option { value: "strict", "严格（更精确）" }
                    }
                    label { class: "label",
                        span { class: "label-text-alt text-base-content/60",
                            "💡 默认方案适合大多数场景"
                        }
                    }
                }

                // 约束说明
                div { class: "alert alert-info",
                    img {
                        class: "stroke-current shrink-0 h-6 w-6",
                        src: ICON_INFO,
                    }
                    div {
                        h3 { class: "font-bold", "当前方案约束" }
                        ul { class: "list-disc list-inside text-sm mt-2",
                            li { "总分差值 ≤ {optimization_params.read().max_score_diff} 分" }
                            li {
                                "单科分差值 ≤ {optimization_params.read().max_subject_score_diff} 分"
                            }
                            li {
                                "性别比例差 ≤ {(optimization_params.read().max_gender_ratio_diff * 100.0):.1}%"
                            }
                            li {
                                "班级人数差 ≤ {optimization_params.read().max_class_size_diff} 人"
                            }
                        }
                    }
                }

                // 高级参数（可折叠）
                div { class: "collapse collapse-arrow bg-base-200 rounded-box",
                    input {
                        r#type: "checkbox",
                        checked: *show_advanced.read(),
                        onchange: move |evt| {
                            show_advanced.set(evt.checked());
                        },
                    }
                    div { class: "collapse-title text-lg font-medium", "⚙️ 高级优化参数" }
                    div { class: "collapse-content",
                        div { class: "space-y-6 pt-4",

                            // ===== 硬约束阈值 =====
                            div { class: "divider divider-start text-sm font-bold text-primary",
                                "硬约束阈值"
                            }

                            div { class: "space-y-3",
                                // 平均分最大差值
                                div { class: "flex items-center gap-4",
                                    label { class: "shrink-0 w-48 text-sm font-medium",
                                        "平均分最大差值（分）"
                                    }
                                    input {
                                        r#type: "number",
                                        class: "input input-bordered input-sm w-20",
                                        value: "{optimization_params.read().max_score_diff}",
                                        step: "0.1",
                                        min: "0.1",
                                        oninput: move |evt| {
                                            if let Ok(val) = evt.value().parse::<f64>() && val > 0.0 {
                                                optimization_params.write().max_score_diff = val;
                                            }
                                        },
                                    }
                                    span { class: "text-xs text-base-content/60 flex-1",
                                        "各班级平均总分之间允许的最大差值。值越小约束越严格。"
                                    }
                                }

                                // 单科平均分最大差值
                                div { class: "flex items-center gap-4",
                                    label { class: "shrink-0 w-48 text-sm font-medium",
                                        "单科平均分最大差值（分）"
                                    }
                                    input {
                                        r#type: "number",
                                        class: "input input-bordered input-sm w-20",
                                        value: "{optimization_params.read().max_subject_score_diff}",
                                        step: "0.1",
                                        min: "0.1",
                                        oninput: move |evt| {
                                            if let Ok(val) = evt.value().parse::<f64>() && val > 0.0 {
                                                optimization_params.write().max_subject_score_diff = val;
                                            }
                                        },
                                    }
                                    span { class: "text-xs text-base-content/60 flex-1",
                                        "各班级单科平均分之间允许的最大差值。独立控制各科目均衡度。"
                                    }
                                }

                                // 班级人数最大差值
                                div { class: "flex items-center gap-4",
                                    label { class: "shrink-0 w-48 text-sm font-medium",
                                        "班级人数最大差值（人）"
                                    }
                                    input {
                                        r#type: "number",
                                        class: "input input-bordered input-sm w-20",
                                        value: "{optimization_params.read().max_class_size_diff}",
                                        min: "1",
                                        oninput: move |evt| {
                                            if let Ok(val) = evt.value().parse::<usize>() && val > 0 {
                                                optimization_params.write().max_class_size_diff = val;
                                            }
                                        },
                                    }
                                    span { class: "text-xs text-base-content/60 flex-1",
                                        "各班级人数之间允许的最大差值。确保班级规模相对均衡。"
                                    }
                                }

                                // 性别比例最大差值
                                div { class: "flex items-center gap-4",
                                    label { class: "shrink-0 w-48 text-sm font-medium",
                                        "性别比例最大差值"
                                    }
                                    input {
                                        r#type: "number",
                                        class: "input input-bordered input-sm w-20",
                                        value: "{optimization_params.read().max_gender_ratio_diff}",
                                        step: "0.01",
                                        min: "0.01",
                                        max: "1.0",
                                        oninput: move |evt| {
                                            if let Ok(val) = evt.value().parse::<f64>() && val > 0.0 && val <= 1.0 {
                                                optimization_params.write().max_gender_ratio_diff = val;
                                            }
                                        },
                                    }
                                    span { class: "text-xs text-base-content/60 flex-1",
                                        "各班级男生比例之间允许的最大差值（0.1 = 10%）。"
                                    }
                                }
                            }

                            // ===== 硬约束惩罚权重 =====
                            div { class: "divider divider-start text-sm font-bold text-primary",
                                "硬约束惩罚权重"
                            }

                            div { class: "space-y-3",
                                // 总分惩罚权重
                                div { class: "flex items-center gap-4",
                                    label { class: "shrink-0 w-48 text-sm font-medium",
                                        "总分差值惩罚权重"
                                    }
                                    input {
                                        r#type: "number",
                                        class: "input input-bordered input-sm w-36",
                                        value: "{optimization_params.read().total_score_penalty_weight}",
                                        step: "1000000",
                                        min: "1000000",
                                        oninput: move |evt| {
                                            if let Ok(val) = evt.value().parse::<f64>() && val > 0.0 {
                                                optimization_params.write().total_score_penalty_weight = val;
                                            }
                                        },
                                    }
                                    span { class: "text-xs text-base-content/60 flex-1",
                                        "总分差值超出阈值时的惩罚系数。值越大，约束越强。"
                                    }
                                }

                                // 科目分惩罚权重
                                div { class: "flex items-center gap-4",
                                    label { class: "shrink-0 w-48 text-sm font-medium",
                                        "科目分差值惩罚权重"
                                    }
                                    input {
                                        r#type: "number",
                                        class: "input input-bordered input-sm w-36",
                                        value: "{optimization_params.read().subject_score_penalty_weight}",
                                        step: "1000000",
                                        min: "1000000",
                                        oninput: move |evt| {
                                            if let Ok(val) = evt.value().parse::<f64>() && val > 0.0 {
                                                optimization_params.write().subject_score_penalty_weight = val;
                                            }
                                        },
                                    }
                                    span { class: "text-xs text-base-content/60 flex-1",
                                        "各科目平均分差值超出阈值时的惩罚系数。"
                                    }
                                }

                                // 性别比例惩罚权重
                                div { class: "flex items-center gap-4",
                                    label { class: "shrink-0 w-48 text-sm font-medium",
                                        "性别比例惩罚权重"
                                    }
                                    input {
                                        r#type: "number",
                                        class: "input input-bordered input-sm w-40",
                                        value: "{optimization_params.read().gender_ratio_penalty_weight}",
                                        step: "1000000000",
                                        min: "1000000",
                                        oninput: move |evt| {
                                            if let Ok(val) = evt.value().parse::<f64>() && val > 0.0 {
                                                optimization_params.write().gender_ratio_penalty_weight = val;
                                            }
                                        },
                                    }
                                    span { class: "text-xs text-base-content/60 flex-1",
                                        "性别比例差值超出阈值时的惩罚系数。推荐设置较高值。"
                                    }
                                }

                                // 惩罚幂次
                                div { class: "flex items-center gap-4",
                                    label { class: "shrink-0 w-48 text-sm font-medium",
                                        "惩罚函数幂次"
                                    }
                                    input {
                                        r#type: "number",
                                        class: "input input-bordered input-sm w-16",
                                        value: "{optimization_params.read().penalty_power}",
                                        min: "1",
                                        max: "10",
                                        oninput: move |evt| {
                                            if let Ok(val) = evt.value().parse::<i32>() && val > 0 && val <= 10 {
                                                optimization_params.write().penalty_power = val;
                                            }
                                        },
                                    }
                                    span { class: "text-xs text-base-content/60 flex-1",
                                        "惩罚函数的幂次（1-10）。值越高，对违反约束的惩罚越严厉。"
                                    }
                                }
                            }

                            // ===== 软约束优化权重 =====
                            div { class: "divider divider-start text-sm font-bold text-primary",
                                "软约束优化权重"
                            }

                            div { class: "space-y-3",
                                // 总分方差权重
                                div { class: "flex items-center gap-4",
                                    label { class: "shrink-0 w-48 text-sm font-medium",
                                        "总分方差权重"
                                    }
                                    input {
                                        r#type: "number",
                                        class: "input input-bordered input-sm w-24",
                                        value: "{optimization_params.read().total_variance_weight}",
                                        step: "1",
                                        min: "0",
                                        oninput: move |evt| {
                                            if let Ok(val) = evt.value().parse::<f64>() && val >= 0.0 {
                                                optimization_params.write().total_variance_weight = val;
                                            }
                                        },
                                    }
                                    span { class: "text-xs text-base-content/60 flex-1",
                                        "总分方差的优化权重。在满足硬约束后，进一步减小总分波动。"
                                    }
                                }

                                // 性别方差权重
                                div { class: "flex items-center gap-4",
                                    label { class: "shrink-0 w-48 text-sm font-medium",
                                        "性别方差权重"
                                    }
                                    input {
                                        r#type: "number",
                                        class: "input input-bordered input-sm w-24",
                                        value: "{optimization_params.read().gender_variance_weight}",
                                        step: "100",
                                        min: "0",
                                        oninput: move |evt| {
                                            if let Ok(val) = evt.value().parse::<f64>() && val >= 0.0 {
                                                optimization_params.write().gender_variance_weight = val;
                                            }
                                        },
                                    }
                                    span { class: "text-xs text-base-content/60 flex-1",
                                        "性别比例方差的优化权重。使各班级性别比例更加均衡。"
                                    }
                                }

                                // 科目方差权重
                                div { class: "flex items-center gap-4",
                                    label { class: "shrink-0 w-48 text-sm font-medium",
                                        "科目方差权重"
                                    }
                                    input {
                                        r#type: "number",
                                        class: "input input-bordered input-sm w-24",
                                        value: "{optimization_params.read().subject_variance_weight}",
                                        step: "10",
                                        min: "0",
                                        oninput: move |evt| {
                                            if let Ok(val) = evt.value().parse::<f64>() && val >= 0.0 {
                                                optimization_params.write().subject_variance_weight = val;
                                            }
                                        },
                                    }
                                    span { class: "text-xs text-base-content/60 flex-1",
                                        "各科目平均分方差的优化权重。使各科目分数更加均衡。"
                                    }
                                }
                            }

                            // ===== 模拟退火参数 =====
                            div { class: "divider divider-start text-sm font-bold text-primary",
                                "模拟退火算法参数"
                            }

                            div { class: "space-y-3",
                                // 初始温度
                                div { class: "flex items-center gap-4",
                                    label { class: "shrink-0 w-48 text-sm font-medium",
                                        "初始温度"
                                    }
                                    input {
                                        r#type: "number",
                                        class: "input input-bordered input-sm w-28",
                                        value: "{optimization_params.read().initial_temperature}",
                                        step: "1000",
                                        min: "1000",
                                        oninput: move |evt| {
                                            if let Ok(val) = evt.value().parse::<f64>() && val > 0.0 {
                                                optimization_params.write().initial_temperature = val;
                                            }
                                        },
                                    }
                                    span { class: "text-xs text-base-content/60 flex-1",
                                        "算法起始温度。值越高，初期探索能力越强。"
                                    }
                                }

                                // 冷却速率
                                div { class: "flex items-center gap-4",
                                    label { class: "shrink-0 w-48 text-sm font-medium",
                                        "冷却速率"
                                    }
                                    input {
                                        r#type: "number",
                                        class: "input input-bordered input-sm w-28",
                                        value: "{optimization_params.read().cooling_rate}",
                                        step: "0.00001",
                                        min: "0.9",
                                        max: "0.99999",
                                        oninput: move |evt| {
                                            if let Ok(val) = evt.value().parse::<f64>() && val > 0.9 && val < 1.0 {
                                                optimization_params.write().cooling_rate = val;
                                            }
                                        },
                                    }
                                    span { class: "text-xs text-base-content/60 flex-1",
                                        "温度下降速率（0.9-0.99999）。越接近 1，降温越慢，搜索越细致。"
                                    }
                                }

                                // 温度多样性增量
                                div { class: "flex items-center gap-4",
                                    label { class: "shrink-0 w-48 text-sm font-medium",
                                        "温度多样性增量"
                                    }
                                    input {
                                        r#type: "number",
                                        class: "input input-bordered input-sm w-28",
                                        value: "{optimization_params.read().temperature_diversity_delta}",
                                        step: "100",
                                        min: "0",
                                        oninput: move |evt| {
                                            if let Ok(val) = evt.value().parse::<f64>() && val >= 0.0 {
                                                optimization_params.write().temperature_diversity_delta = val;
                                            }
                                        },
                                    }
                                    span { class: "text-xs text-base-content/60 flex-1",
                                        "为不同并行实例增加的温度差异。增加搜索多样性。"
                                    }
                                }
                            }

                            // ===== 早停与重启参数 =====
                            div { class: "divider divider-start text-sm font-bold text-primary",
                                "早停与重启参数"
                            }

                            div { class: "space-y-3",
                                // 良好解阈值
                                div { class: "flex items-center gap-4",
                                    label { class: "shrink-0 w-48 text-sm font-medium",
                                        "良好解阈值"
                                    }
                                    input {
                                        r#type: "number",
                                        class: "input input-bordered input-sm w-20",
                                        value: "{optimization_params.read().good_solution_threshold}",
                                        step: "0.1",
                                        min: "0",
                                        oninput: move |evt| {
                                            if let Ok(val) = evt.value().parse::<f64>() && val >= 0.0 {
                                                optimization_params.write().good_solution_threshold = val;
                                            }
                                        },
                                    }
                                    span { class: "text-xs text-base-content/60 flex-1",
                                        "找到满足约束的解的代价阈值。达到此值时提前结束搜索。"
                                    }
                                }

                                // 重新加热迭代次数
                                div { class: "flex items-center gap-4",
                                    label { class: "shrink-0 w-48 text-sm font-medium",
                                        "重新加热迭代次数"
                                    }
                                    input {
                                        r#type: "number",
                                        class: "input input-bordered input-sm w-24",
                                        value: "{optimization_params.read().reheat_after_iterations}",
                                        step: "100",
                                        min: "100",
                                        oninput: move |evt| {
                                            if let Ok(val) = evt.value().parse::<usize>() && val >= 100 {
                                                optimization_params.write().reheat_after_iterations = val;
                                            }
                                        },
                                    }
                                    span { class: "text-xs text-base-content/60 flex-1",
                                        "无改进时触发重新加热的迭代次数。防止陷入局部最优。"
                                    }
                                }

                                // 重新加热温度倍数
                                div { class: "flex items-center gap-4",
                                    label { class: "shrink-0 w-48 text-sm font-medium",
                                        "重新加热温度倍数"
                                    }
                                    input {
                                        r#type: "number",
                                        class: "input input-bordered input-sm w-20",
                                        value: "{optimization_params.read().reheat_temperature_factor}",
                                        step: "0.1",
                                        min: "0.1",
                                        max: "2.0",
                                        oninput: move |evt| {
                                            if let Ok(val) = evt.value().parse::<f64>() && val > 0.0 && val <= 2.0 {
                                                optimization_params.write().reheat_temperature_factor = val;
                                            }
                                        },
                                    }
                                    span { class: "text-xs text-base-content/60 flex-1",
                                        "重新加热时相对于当前温度的倍数（0.1-2.0）。"
                                    }
                                }

                                // 重新加热最小接受次数
                                div { class: "flex items-center gap-4",
                                    label { class: "shrink-0 w-48 text-sm font-medium",
                                        "重新加热最小接受次数"
                                    }
                                    input {
                                        r#type: "number",
                                        class: "input input-bordered input-sm w-24",
                                        value: "{optimization_params.read().reheat_min_accept_count}",
                                        step: "10",
                                        min: "10",
                                        oninput: move |evt| {
                                            if let Ok(val) = evt.value().parse::<usize>() && val >= 10 {
                                                optimization_params.write().reheat_min_accept_count = val;
                                            }
                                        },
                                    }
                                    span { class: "text-xs text-base-content/60 flex-1",
                                        "触发重新加热所需的最小接受次数阈值。"
                                    }
                                }
                            }

                            // 重置按钮
                            div { class: "mt-6 flex justify-end gap-2",
                                button {
                                    class: "btn btn-outline btn-sm",
                                    onclick: move |_| {
                                        preset.set("default".to_string());
                                        optimization_params.set(OptimizationParams::default());
                                    },
                                    "重置为默认"
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
                    onclick: move |_| on_start.call(()),
                    "开始分班"
                }
            }
        }
    }
}

use dioxus::prelude::*;

#[component]
pub fn ProcessingView() -> Element {
    rsx! {
        div { class: "text-center py-16",
            // 主要加载动画
            div { class: "flex justify-center mb-6",
                div { class: "loading loading-spinner loading-lg text-primary" }
            }

            // 标题和提示
            h2 { class: "text-3xl font-bold mb-3", "正在分班中..." }
            p { class: "text-base-content/70 mb-6", "算法正在优化班级分配，请稍候" }

            // 进度提示卡片
            div { class: "flex flex-wrap justify-center gap-3 mt-8 max-w-md mx-auto",
                div { class: "badge badge-lg badge-primary gap-2", "🎯 平衡分数" }
                div { class: "badge badge-lg badge-secondary gap-2", "⚖️ 均衡性别" }
                div { class: "badge badge-lg badge-accent gap-2", "📊 优化人数" }
            }

            // 处理步骤
            div { class: "mt-8 text-sm text-base-content/60",
                div { class: "loading loading-dots loading-sm inline-block mr-2" }
                "使用模拟退火算法进行多目标优化"
            }
        }
    }
}

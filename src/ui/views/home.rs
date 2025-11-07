use crate::core::{
    algorithm::{DivideConfig, divide_students, validate_constraints_with_params},
    io::{
        ExcelColumnConfig, export_classes_to_excel_with_extras,
        read_students_from_excel_with_config,
    },
    model::{Class, Student},
};
use calamine::{Data, Reader, Xls, Xlsx, open_workbook};
use dioxus::prelude::*;
use rfd::AsyncFileDialog;

#[derive(Clone, Debug, PartialEq)]
enum AppStep {
    SelectFile,
    PreviewData,
    ConfigureColumns,
    ConfigureDivision,
    Processing,
    Results,
}

#[derive(Clone, Debug)]
struct ColumnMapping {
    name: String,
    index: usize,
    column_type: ColumnType,
}

#[derive(Clone, Debug, PartialEq)]
enum ColumnType {
    Name,
    Gender,
    StudentId,
    TotalScore,
    Subject,
    Extra,
    Ignore,
}

#[component]
pub fn Home() -> Element {
    let mut step = use_signal(|| AppStep::SelectFile);
    let mut file_path = use_signal(|| None::<String>);
    let mut headers = use_signal(Vec::<String>::new);
    let mut preview_data = use_signal(Vec::<Vec<String>>::new);
    let mut column_mappings = use_signal(Vec::<ColumnMapping>::new);
    let num_classes = use_signal(|| 10);
    let mut processing = use_signal(|| false);
    let mut error_message = use_signal(|| None::<String>);
    let mut success_message = use_signal(|| None::<String>);
    let mut result_classes = use_signal(Vec::<Class>::new);
    let mut result_summary = use_signal(|| None::<String>);

    // 文件选择处理
    let select_file = move |_| {
        spawn(async move {
            if let Some(file) = AsyncFileDialog::new()
                .add_filter("Excel Files", &["xls", "xlsx"])
                .pick_file()
                .await
            {
                let path = file.path().to_string_lossy().to_string();

                // 读取表头和所有数据
                match read_excel_all_data(&path) {
                    Ok((header_list, data_rows)) => {
                        file_path.set(Some(path));
                        headers.set(header_list.clone());
                        preview_data.set(data_rows);

                        // 初始化列映射
                        let mappings: Vec<ColumnMapping> = header_list
                            .iter()
                            .enumerate()
                            .map(|(idx, name)| {
                                let col_type = infer_column_type(name);
                                ColumnMapping {
                                    name: name.clone(),
                                    index: idx,
                                    column_type: col_type,
                                }
                            })
                            .collect();

                        column_mappings.set(mappings);
                        step.set(AppStep::PreviewData);
                        error_message.set(None);
                    }
                    Err(e) => {
                        error_message.set(Some(format!("读取文件失败: {}", e)));
                    }
                }
            }
        });
    };

    // 确认预览并进入列配置
    let confirm_preview = move |_| {
        step.set(AppStep::ConfigureColumns);
    };

    // 列配置确认
    let confirm_columns = move |_| {
        let has_name = column_mappings
            .read()
            .iter()
            .any(|m| m.column_type == ColumnType::Name);
        let has_gender = column_mappings
            .read()
            .iter()
            .any(|m| m.column_type == ColumnType::Gender);
        let has_subjects = column_mappings
            .read()
            .iter()
            .any(|m| m.column_type == ColumnType::Subject);

        if !has_name {
            error_message.set(Some("请指定姓名列".to_string()));
            return;
        }
        if !has_gender {
            error_message.set(Some("请指定性别列".to_string()));
            return;
        }
        if !has_subjects {
            error_message.set(Some("请至少指定一个科目列".to_string()));
            return;
        }

        error_message.set(None);
        step.set(AppStep::ConfigureDivision);
    };

    // 开始分班
    let start_division = move |_| {
        let path = match file_path.read().clone() {
            Some(p) => p,
            None => return,
        };

        let mappings = column_mappings.read().clone();
        let classes = *num_classes.read();

        // 立即切换到 Processing 状态
        processing.set(true);
        step.set(AppStep::Processing);
        error_message.set(None);

        spawn(async move {
            // 给 UI 一点时间渲染
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;

            // 构建列配置
            let mut config_builder = ExcelColumnConfig::builder();
            let mut subject_names = Vec::new();
            let mut extra_field_names = Vec::new();

            for mapping in &mappings {
                match mapping.column_type {
                    ColumnType::Name => {
                        config_builder = config_builder.name_column(mapping.index);
                    }
                    ColumnType::Gender => {
                        config_builder = config_builder.gender_column(mapping.index);
                    }
                    ColumnType::StudentId => {
                        config_builder = config_builder.student_id_column(mapping.index);
                    }
                    ColumnType::TotalScore => {
                        config_builder = config_builder.total_score_column(mapping.index);
                    }
                    ColumnType::Subject => {
                        config_builder =
                            config_builder.add_subject(mapping.name.clone(), mapping.index);
                        subject_names.push(mapping.name.clone());
                    }
                    ColumnType::Extra => {
                        config_builder =
                            config_builder.add_extra_column(mapping.name.clone(), mapping.index);
                        extra_field_names.push(mapping.name.clone());
                    }
                    ColumnType::Ignore => {}
                }
            }

            // 执行分班
            match config_builder.build() {
                Ok(config) => match read_students_from_excel_with_config(&path, &config) {
                    Ok(students) => {
                        let divide_config = DivideConfig::new(classes);
                        let classes_result = divide_students(&students, divide_config.clone());
                        let validation = validate_constraints_with_params(
                            &classes_result,
                            &divide_config.optimization_params,
                        );

                        let summary = format!(
                            "学生总数: {}\n班级数量: {}\n总分最大差值: {:.2}分\n性别比例最大差: {:.1}%",
                            students.len(),
                            classes,
                            validation.max_score_diff,
                            validation.max_gender_ratio_diff * 100.0,
                        );

                        result_summary.set(Some(summary));
                        result_classes.set(classes_result);
                        success_message.set(Some("分班成功！".to_string()));
                        step.set(AppStep::Results);
                    }
                    Err(e) => {
                        error_message.set(Some(format!("读取学生数据失败: {}", e)));
                        step.set(AppStep::ConfigureDivision);
                    }
                },
                Err(e) => {
                    error_message.set(Some(format!("配置错误: {}", e)));
                    step.set(AppStep::ConfigureDivision);
                }
            }

            processing.set(false);
        });
    };

    // 导出结果
    let export_results = move |_| {
        let classes = result_classes.read().clone();
        let mappings = column_mappings.read().clone();

        spawn(async move {
            // 让用户选择保存位置
            if let Some(file) = AsyncFileDialog::new()
                .set_file_name("分班结果.xlsx")
                .add_filter("Excel Files", &["xlsx"])
                .save_file()
                .await
            {
                let output_path = file.path().to_string_lossy().to_string();

                let subject_names: Vec<String> = mappings
                    .iter()
                    .filter(|m| m.column_type == ColumnType::Subject)
                    .map(|m| m.name.clone())
                    .collect();

                let extra_field_names: Vec<String> = mappings
                    .iter()
                    .filter(|m| m.column_type == ColumnType::Extra)
                    .map(|m| m.name.clone())
                    .collect();

                let subjects_refs: Vec<&str> = subject_names.iter().map(|s| s.as_str()).collect();
                let extras_refs: Vec<&str> = extra_field_names.iter().map(|s| s.as_str()).collect();

                match export_classes_to_excel_with_extras(
                    &classes,
                    &output_path,
                    &subjects_refs,
                    &extras_refs,
                ) {
                    Ok(_) => {
                        success_message
                            .set(Some(format!("导出成功！\n文件已保存至: {}", output_path)));
                    }
                    Err(e) => {
                        error_message.set(Some(format!("导出失败: {}", e)));
                    }
                }
            }
        });
    };

    rsx! {
        div { class: "min-h-screen bg-base-200 p-4 md:p-8",
            div { class: "max-w-7xl mx-auto",
                // 标题
                div { class: "text-center mb-6",
                    h1 { class: "text-3xl md:text-4xl font-bold text-primary mb-2",
                        "分班系统"
                    }

                }

                // 步骤指示器
                div { class: "mb-6",
                    ul { class: "steps steps-horizontal w-full text-xs md:text-sm",
                        li { class: if matches!(*step.read(), AppStep::SelectFile) { "step step-primary" } else { "step" },
                            "选择文件"
                        }
                        li { class: if matches!(*step.read(), AppStep::PreviewData) { "step step-primary" } else if matches!(*step.read(), AppStep::SelectFile) { "step" } else { "step step-primary" },
                            "预览数据"
                        }
                        li { class: if matches!(*step.read(), AppStep::ConfigureColumns) { "step step-primary" } else if matches!(*step.read(), AppStep::SelectFile | AppStep::PreviewData) { "step" } else { "step step-primary" },
                            "配置列"
                        }
                        li { class: if matches!(*step.read(), AppStep::ConfigureDivision) { "step step-primary" } else if matches!(*step.read(), AppStep::Results | AppStep::Processing) { "step step-primary" } else { "step" },
                            "设置参数"
                        }
                        li { class: if matches!(*step.read(), AppStep::Results | AppStep::Processing) { "step step-primary" } else { "step" },
                            "完成"
                        }
                    }
                }

                // 错误消息
                if let Some(err) = error_message.read().as_ref() {
                    div { class: "alert alert-error mb-4 animate-fade-in shadow-lg",
                        svg {
                            class: "stroke-current shrink-0 h-6 w-6",
                            xmlns: "http://www.w3.org/2000/svg",
                            fill: "none",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z",
                            }
                        }
                        span { "{err}" }
                        button {
                            class: "btn btn-sm btn-ghost",
                            onclick: move |_| error_message.set(None),
                            "✕"
                        }
                    }
                }

                // 成功消息
                if let Some(msg) = success_message.read().as_ref() {
                    div { class: "alert alert-success mb-4 animate-fade-in shadow-lg",
                        svg {
                            class: "stroke-current shrink-0 h-6 w-6",
                            xmlns: "http://www.w3.org/2000/svg",
                            fill: "none",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z",
                            }
                        }
                        span { "{msg}" }
                        button {
                            class: "btn btn-sm btn-ghost",
                            onclick: move |_| success_message.set(None),
                            "✕"
                        }
                    }
                }

                // 主内容卡片
                div { class: "card bg-base-100 shadow-xl",
                    div { class: "card-body p-4 md:p-8",
                        match *step.read() {
                            AppStep::SelectFile => rsx! {
                                SelectFileView { on_select: select_file }
                            },
                            AppStep::PreviewData => rsx! {
                                PreviewDataView {
                                    headers,
                                    data: preview_data,
                                    on_confirm: confirm_preview,
                                    on_back: move |_| step.set(AppStep::SelectFile),
                                }
                            },
                            AppStep::ConfigureColumns => rsx! {
                                ColumnConfigView {
                                    column_mappings,
                                    on_confirm: confirm_columns,
                                    on_back: move |_| step.set(AppStep::PreviewData),
                                }
                            },
                            AppStep::ConfigureDivision => rsx! {
                                DivisionConfigView {
                                    num_classes,
                                    on_start: start_division,
                                    on_back: move |_| step.set(AppStep::ConfigureColumns),
                                }
                            },
                            AppStep::Processing => rsx! {
                                ProcessingView {}
                            },
                            AppStep::Results => rsx! {
                                ResultsView {
                                    classes: result_classes,
                                    summary: result_summary.read().clone(),
                                    column_mappings,
                                    on_export: export_results,
                                    on_restart: move |_| {
                                        step.set(AppStep::SelectFile);
                                        file_path.set(None);
                                        headers.set(Vec::new());
                                        preview_data.set(Vec::new());
                                        column_mappings.set(Vec::new());
                                        result_classes.set(Vec::new());
                                        result_summary.set(None);
                                        success_message.set(None);
                                        error_message.set(None);
                                    },
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

// ============ 视图组件 ============

#[component]
fn SelectFileView(on_select: EventHandler<()>) -> Element {
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

#[component]
fn PreviewDataView(
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

#[component]
fn ColumnConfigView(
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
                                        class: "select select-bordered select-sm",
                                        value: "{column_type_to_string(&mapping.column_type)}",
                                        onchange: move |evt| {
                                            let new_type = string_to_column_type(&evt.value());
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

#[component]
fn DivisionConfigView(
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

#[component]
fn ProcessingView() -> Element {
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

#[component]
fn ResultsView(
    classes: Signal<Vec<Class>>,
    summary: Option<String>,
    column_mappings: Signal<Vec<ColumnMapping>>,
    on_export: EventHandler<()>,
    on_restart: EventHandler<()>,
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
                    svg {
                        class: "w-16 h-16 mx-auto text-warning",
                        xmlns: "http://www.w3.org/2000/svg",
                        fill: "none",
                        view_box: "0 0 24 24",
                        stroke: "currentColor",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z",
                        }
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
            div { class: "text-center",
                h2 { class: "text-2xl font-bold", "分班完成！" }
            }

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
                        h3 { class: "text-lg font-bold mb-3", "班级统计" }
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
                        h3 { class: "text-lg font-bold mb-3", "学生分班结果" }
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
                    class: "btn btn-primary",
                    onclick: move |_| on_export.call(()),
                    "导出结果"
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

// ============ 辅助函数 ============
#[allow(clippy::type_complexity)]
fn read_excel_all_data(
    file_path: &str,
) -> Result<(Vec<String>, Vec<Vec<String>>), Box<dyn std::error::Error>> {
    let (headers, rows) = if file_path.to_lowercase().ends_with(".xls") {
        let mut workbook: Xls<_> = open_workbook(file_path)?;
        let sheet_name = workbook.sheet_names()[0].clone();
        let range = workbook.worksheet_range(&sheet_name)?;

        let mut all_rows: Vec<Vec<String>> = range
            .rows()
            .map(|row| row.iter().map(cell_to_string).collect())
            .collect();

        if all_rows.is_empty() {
            return Err("文件没有数据".into());
        }

        let headers = all_rows.remove(0);
        (headers, all_rows)
    } else {
        let mut workbook: Xlsx<_> = open_workbook(file_path)?;
        let sheet_name = workbook.sheet_names()[0].clone();
        let range = workbook.worksheet_range(&sheet_name)?;

        let mut all_rows: Vec<Vec<String>> = range
            .rows()
            .map(|row| row.iter().map(cell_to_string).collect())
            .collect();

        if all_rows.is_empty() {
            return Err("文件没有数据".into());
        }

        let headers = all_rows.remove(0);
        (headers, all_rows)
    };

    Ok((headers, rows))
}

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::String(s) => s.clone(),
        Data::Int(i) => i.to_string(),
        Data::Float(f) => f.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(dt) => dt.to_string(),
        Data::DateTimeIso(dt) => dt.clone(),
        Data::DurationIso(d) => d.clone(),
        Data::Error(_) | Data::Empty => String::new(),
    }
}

fn infer_column_type(name: &str) -> ColumnType {
    let name_lower = name.to_lowercase();

    if name_lower.contains("姓名") || name_lower.contains("name") {
        ColumnType::Name
    } else if name_lower.contains("性别") || name_lower.contains("gender") {
        ColumnType::Gender
    } else if name_lower.contains("学号")
        || name_lower.contains("id")
        || name_lower.contains("编号")
    {
        ColumnType::StudentId
    } else if name_lower.contains("总分") || name_lower.contains("总成绩") || name_lower == "total"
    {
        ColumnType::TotalScore
    } else if name_lower.contains("语文")
        || name_lower.contains("数学")
        || name_lower.contains("英语")
        || name_lower.contains("日语")
        || name_lower.contains("物理")
        || name_lower.contains("化学")
        || name_lower.contains("生物")
        || name_lower.contains("政治")
        || name_lower.contains("历史")
        || name_lower.contains("地理")
        || name_lower.contains("外语")
    {
        ColumnType::Subject
    } else if name_lower.contains("班级")
        || name_lower.contains("备注")
        || name_lower.contains("原班级")
    {
        ColumnType::Extra
    } else {
        ColumnType::Ignore
    }
}

fn column_type_to_string(col_type: &ColumnType) -> &'static str {
    match col_type {
        ColumnType::Name => "name",
        ColumnType::Gender => "gender",
        ColumnType::StudentId => "student_id",
        ColumnType::TotalScore => "total",
        ColumnType::Subject => "subject",
        ColumnType::Extra => "extra",
        ColumnType::Ignore => "ignore",
    }
}

fn string_to_column_type(s: &str) -> ColumnType {
    match s {
        "name" => ColumnType::Name,
        "gender" => ColumnType::Gender,
        "student_id" => ColumnType::StudentId,
        "total" => ColumnType::TotalScore,
        "subject" => ColumnType::Subject,
        "extra" => ColumnType::Extra,
        _ => ColumnType::Ignore,
    }
}

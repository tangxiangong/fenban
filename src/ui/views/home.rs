use crate::core::{
    algorithm::{
        DivideConfig, OptimizationParams, divide_students, validate_constraints_with_params,
    },
    io::{
        ExcelColumnConfig, export_classes_to_csv_with_extras, export_classes_to_excel_with_extras,
        read_students_from_csv_with_config, read_students_from_excel_with_config,
    },
    model::Class,
};
use crate::ui::components::*;
use crate::ui::{ICON_ERROR, ICON_SUCCESS, LOGO};
use dioxus::prelude::*;
use rfd::AsyncFileDialog;

#[component]
pub fn Home() -> Element {
    let mut step = use_signal(|| AppStep::SelectFile);
    let mut file_path = use_signal(|| None::<String>);
    let mut headers = use_signal(Vec::<String>::new);
    let mut preview_data = use_signal(Vec::<Vec<String>>::new);
    let mut column_mappings = use_signal(Vec::<ColumnMapping>::new);
    let num_classes = use_signal(|| 2);
    let mut processing = use_signal(|| false);
    let mut error_message = use_signal(|| None::<String>);
    let mut success_message = use_signal(|| None::<String>);
    let mut result_classes = use_signal(Vec::<Class>::new);
    let mut result_summary = use_signal(|| None::<String>);
    let mut optimization_params = use_signal(OptimizationParams::default);

    // 文件选择处理
    let select_file = move |_| {
        spawn(async move {
            if let Some(file) = AsyncFileDialog::new()
                .add_filter("数据文件", &["xls", "xlsx", "csv"])
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
        let opt_params = optimization_params.read().clone();

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
                Ok(config) => {
                    // 根据文件类型选择读取方式
                    let students_result = if path.to_lowercase().ends_with(".csv") {
                        read_students_from_csv_with_config(&path, &config)
                    } else {
                        read_students_from_excel_with_config(&path, &config)
                    };

                    match students_result {
                        Ok(students) => {
                            let divide_config = DivideConfig::new(classes)
                                .with_optimization_params(opt_params.clone());
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
                    }
                }
                Err(e) => {
                    error_message.set(Some(format!("配置错误: {}", e)));
                    step.set(AppStep::ConfigureDivision);
                }
            }

            processing.set(false);
        });
    };

    // 导出结果
    let export_results = move |format: String| {
        let classes = result_classes.read().clone();
        let mappings = column_mappings.read().clone();

        spawn(async move {
            // 根据格式设置默认文件名和过滤器
            let (file_name, filter_name, filter_ext) = match format.as_str() {
                "csv" => ("分班结果.csv", "CSV Files", vec!["csv"]),
                _ => ("分班结果.xlsx", "Excel Files", vec!["xlsx"]),
            };

            // 让用户选择保存位置
            if let Some(file) = AsyncFileDialog::new()
                .set_file_name(file_name)
                .add_filter(filter_name, &filter_ext)
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

                // 根据文件扩展名选择导出格式
                let export_result = if output_path.to_lowercase().ends_with(".csv") {
                    export_classes_to_csv_with_extras(
                        &classes,
                        &output_path,
                        &subjects_refs,
                        &extras_refs,
                    )
                } else {
                    export_classes_to_excel_with_extras(
                        &classes,
                        &output_path,
                        &subjects_refs,
                        &extras_refs,
                    )
                };

                match export_result {
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
                    div { class: "flex items-center justify-center gap-3 mb-2",
                        img {
                            class: "w-12 h-12 md:w-16 md:h-16 cursor-pointer hover:opacity-80 transition-opacity",
                            src: LOGO,
                            alt: "分班",
                            onclick: move |_| {
                                step.set(AppStep::SelectFile);
                                file_path.set(None);
                                headers.set(Vec::new());
                                preview_data.set(Vec::new());
                                column_mappings.set(Vec::new());
                                result_classes.set(Vec::new());
                                result_summary.set(None);
                                success_message.set(None);
                                error_message.set(None);
                                optimization_params.set(OptimizationParams::default());
                            },
                        }
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
                        img {
                            class: "stroke-current shrink-0 h-6 w-6",
                            src: ICON_ERROR,
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
                        img {
                            class: "stroke-current shrink-0 h-6 w-6",
                            src: ICON_SUCCESS,
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
                                    optimization_params,
                                    on_start: start_division,
                                    on_back: move |_| step.set(AppStep::ConfigureColumns),
                                }
                            },
                            AppStep::Processing => rsx! {
                                ProcessingView { num_classes, optimization_params }
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
                                        optimization_params.set(OptimizationParams::default());
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

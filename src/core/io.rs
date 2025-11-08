use super::model::{Class, Gender, Student};
use calamine::{Data, DataType, Reader, Xls, Xlsx, open_workbook};
use csv::{Reader as CsvReader, Writer as CsvWriter};
use fs_err::File;
use rayon::prelude::*;
use rust_xlsxwriter::{Format, Workbook};
use std::collections::HashMap;

/// Excel 列配置
#[derive(Debug, Clone)]
pub struct ExcelColumnConfig {
    /// 学生姓名所在列（0-based）
    pub name_column: usize,
    /// 学号所在列（0-based），None 表示没有学号列
    pub student_id_column: Option<usize>,
    /// 性别所在列（0-based）
    pub gender_column: usize,
    /// 总成绩所在列（0-based），None 表示自动计算
    pub total_score_column: Option<usize>,
    /// 单科成绩列（列名 -> 列索引）
    pub subject_columns: HashMap<String, usize>,
    /// 需要保留的额外列（列名 -> 列索引）
    pub extra_columns: HashMap<String, usize>,
}

impl ExcelColumnConfig {
    /// 创建默认配置（兼容旧格式）
    /// 格式：姓名 | 性别 | 科目1 | 科目2 | ...
    pub fn default_legacy() -> Self {
        Self {
            name_column: 0,
            student_id_column: None,
            gender_column: 1,
            total_score_column: None,
            subject_columns: HashMap::new(), // 需要后续填充
            extra_columns: HashMap::new(),
        }
    }

    /// 从表头自动推断配置
    pub fn from_header(header: &[Data]) -> Self {
        let mut config = Self {
            name_column: 0,
            student_id_column: None,
            gender_column: 1,
            total_score_column: None,
            subject_columns: HashMap::new(),
            extra_columns: HashMap::new(),
        };

        // 从第3列开始是科目
        for (idx, cell) in header.iter().enumerate().skip(2) {
            if let Some(name) = cell.get_string() {
                config.subject_columns.insert(name.to_string(), idx);
            }
        }

        config
    }

    /// 手动构建配置
    pub fn builder() -> ExcelColumnConfigBuilder {
        ExcelColumnConfigBuilder::default()
    }
}

/// Excel 列配置构建器
#[derive(Debug, Default)]
pub struct ExcelColumnConfigBuilder {
    name_column: Option<usize>,
    student_id_column: Option<usize>,
    gender_column: Option<usize>,
    total_score_column: Option<usize>,
    subject_columns: HashMap<String, usize>,
    extra_columns: HashMap<String, usize>,
}

impl ExcelColumnConfigBuilder {
    pub fn name_column(mut self, col: usize) -> Self {
        self.name_column = Some(col);
        self
    }

    pub fn student_id_column(mut self, col: usize) -> Self {
        self.student_id_column = Some(col);
        self
    }

    pub fn gender_column(mut self, col: usize) -> Self {
        self.gender_column = Some(col);
        self
    }

    pub fn total_score_column(mut self, col: usize) -> Self {
        self.total_score_column = Some(col);
        self
    }

    pub fn add_subject(mut self, name: String, col: usize) -> Self {
        self.subject_columns.insert(name, col);
        self
    }

    pub fn add_extra_column(mut self, name: String, col: usize) -> Self {
        self.extra_columns.insert(name, col);
        self
    }

    pub fn build(self) -> anyhow::Result<ExcelColumnConfig> {
        Ok(ExcelColumnConfig {
            name_column: self
                .name_column
                .ok_or_else(|| anyhow::anyhow!("姓名列未指定"))?,
            student_id_column: self.student_id_column,
            gender_column: self
                .gender_column
                .ok_or_else(|| anyhow::anyhow!("性别列未指定"))?,
            total_score_column: self.total_score_column,
            subject_columns: self.subject_columns,
            extra_columns: self.extra_columns,
        })
    }
}

/// Excel 工作簿枚举，支持 .xls 和 .xlsx 格式
enum ExcelWorkbook {
    Xls(Xls<std::io::BufReader<std::fs::File>>),
    Xlsx(Xlsx<std::io::BufReader<std::fs::File>>),
}

impl ExcelWorkbook {
    /// 打开 Excel 文件（自动识别 .xls 和 .xlsx 格式）
    fn open(file_path: &str) -> anyhow::Result<Self> {
        if file_path.to_lowercase().ends_with(".xls") {
            let workbook: Xls<_> = open_workbook(file_path)?;
            Ok(ExcelWorkbook::Xls(workbook))
        } else if file_path.to_lowercase().ends_with(".xlsx") {
            let workbook: Xlsx<_> = open_workbook(file_path)?;
            Ok(ExcelWorkbook::Xlsx(workbook))
        } else {
            anyhow::bail!("不支持的文件格式，仅支持 .xls 和 .xlsx 文件");
        }
    }

    /// 获取工作表名称列表
    fn sheet_names(&self) -> Vec<String> {
        match self {
            ExcelWorkbook::Xls(wb) => wb.sheet_names().to_vec(),
            ExcelWorkbook::Xlsx(wb) => wb.sheet_names().to_vec(),
        }
    }

    /// 获取工作表范围
    fn worksheet_range(&mut self, name: &str) -> anyhow::Result<calamine::Range<Data>> {
        match self {
            ExcelWorkbook::Xls(wb) => Ok(wb.worksheet_range(name)?),
            ExcelWorkbook::Xlsx(wb) => Ok(wb.worksheet_range(name)?),
        }
    }
}

/// 从 Excel 读取学生数据（使用列配置）
pub fn read_students_from_excel_with_config(
    file_path: &str,
    config: &ExcelColumnConfig,
) -> anyhow::Result<Vec<Student>> {
    let mut workbook = ExcelWorkbook::open(file_path)?;
    let sheet_name = workbook.sheet_names()[0].clone();
    let range = workbook.worksheet_range(&sheet_name)?;

    let rows: Vec<_> = range.rows().collect();
    if rows.len() <= 1 {
        anyhow::bail!("Excel 文件没有数据");
    }

    // 并行处理学生数据
    let students: Vec<Student> = rows
        .par_iter()
        .enumerate()
        .skip(1) // 跳过表头
        .filter_map(|(row_idx, row)| {
            // 读取姓名
            let name = get_cell_string(row, config.name_column)?;
            if name.is_empty() {
                return None;
            }

            // 读取学号（如果没有学号列，使用行号）
            let student_id = if let Some(col) = config.student_id_column {
                get_cell_string(row, col)
            } else {
                Some(format!("R{}", row_idx + 1))
            };

            // 读取性别
            let gender_str = get_cell_string(row, config.gender_column)?;
            let gender = gender_str.parse::<Gender>().ok()?;

            // 读取科目成绩
            let mut scores = HashMap::with_capacity(config.subject_columns.len());
            for (subject, &col_idx) in &config.subject_columns {
                let score = get_cell_score(row, col_idx);
                scores.insert(subject.clone(), score);
            }

            // 读取或计算总分
            let total_score = if let Some(col) = config.total_score_column {
                get_cell_score(row, col)
            } else {
                scores.values().sum()
            };

            // 读取额外字段
            let mut extra_fields = HashMap::with_capacity(config.extra_columns.len());
            for (field_name, &col_idx) in &config.extra_columns {
                if let Some(value) = get_cell_string(row, col_idx) {
                    extra_fields.insert(field_name.clone(), value);
                }
            }

            Some(Student {
                name,
                student_id,
                gender,
                scores,
                total_score,
                extra_fields,
            })
        })
        .collect();

    if students.is_empty() {
        anyhow::bail!("未读取到任何学生数据");
    }

    Ok(students)
}

/// 从 Excel 读取学生数据（保持向后兼容）
pub fn read_students_from_excel(file_path: &str) -> anyhow::Result<(Vec<Student>, Vec<String>)> {
    let mut workbook = ExcelWorkbook::open(file_path)?;
    let sheet_name = workbook.sheet_names()[0].clone();
    let range = workbook.worksheet_range(&sheet_name)?;

    let rows: Vec<_> = range.rows().collect();
    if rows.is_empty() {
        anyhow::bail!("Excel 文件为空");
    }

    // 读取表头
    let subjects: Vec<String> = rows[0]
        .iter()
        .skip(2)
        .filter_map(|cell| cell.get_string().map(|s| s.to_string()))
        .collect();

    if subjects.is_empty() {
        anyhow::bail!("未找到科目列");
    }

    // 并行处理学生数据
    let students: Vec<Student> = rows
        .par_iter()
        .enumerate()
        .skip(1)
        .filter_map(|(row_idx, row)| {
            if row.len() < 2 {
                return None;
            }

            let name = row[0].get_string()?.to_string();
            if name.is_empty() {
                return None;
            }

            let gender_str = row[1].get_string()?;
            let gender = gender_str.parse::<Gender>().ok()?;

            let mut scores = HashMap::with_capacity(subjects.len());
            let mut total = 0.0;

            for (idx, subject) in subjects.iter().enumerate() {
                let score = if idx + 2 < row.len() {
                    parse_score(&row[idx + 2])
                } else {
                    0.0
                };
                scores.insert(subject.clone(), score);
                total += score;
            }

            Some(Student {
                name,
                student_id: Some(format!("R{}", row_idx + 1)),
                gender,
                scores,
                total_score: total,
                extra_fields: HashMap::new(),
            })
        })
        .collect();

    if students.is_empty() {
        anyhow::bail!("未读取到任何学生数据");
    }

    Ok((students, subjects))
}

// 辅助函数：从单元格读取字符串
fn get_cell_string(row: &[Data], col: usize) -> Option<String> {
    if col < row.len() {
        match &row[col] {
            Data::String(s) => Some(s.clone()),
            Data::Int(i) => Some(i.to_string()),
            Data::Float(f) => Some(f.to_string()),
            Data::Bool(b) => Some(b.to_string()),
            Data::DateTime(dt) => Some(dt.to_string()),
            Data::DateTimeIso(dt) => Some(dt.clone()),
            Data::DurationIso(d) => Some(d.clone()),
            Data::Error(_) | Data::Empty => None,
        }
    } else {
        None
    }
}

// 辅助函数：从单元格读取分数
fn get_cell_score(row: &[Data], col: usize) -> f64 {
    if col < row.len() {
        parse_score(&row[col])
    } else {
        0.0
    }
}

#[inline]
fn parse_score(cell: &calamine::Data) -> f64 {
    cell.get_float().unwrap_or_else(|| {
        cell.get_int()
            .map(|i| i as f64)
            .or_else(|| cell.get_string().and_then(|s| s.parse::<f64>().ok()))
            .unwrap_or(0.0)
    })
}

/// 辅助函数：检查是否有真实学号（不是自动生成的行号）
fn has_real_student_ids(classes: &[Class]) -> bool {
    classes.iter().any(|class| {
        class.students.iter().any(|student| {
            student
                .student_id
                .as_ref()
                .is_some_and(|id| !id.starts_with("R"))
        })
    })
}

/// 导出分班结果到 Excel（带额外字段）
pub fn export_classes_to_excel_with_extras(
    classes: &[Class],
    file_path: &str,
    subjects: &[&str],
    extra_field_names: &[&str],
) -> anyhow::Result<()> {
    let mut workbook = Workbook::new();

    // 创建格式
    let header_format = Format::new()
        .set_bold()
        .set_background_color(rust_xlsxwriter::Color::RGB(0xD9E1F2));

    let score_format = Format::new().set_num_format("0.0");

    // 检查是否有真实学号
    let has_student_id = has_real_student_ids(classes);

    // 工作表1: 分班结果详情
    let sheet = workbook.add_worksheet();
    sheet.set_name("分班结果")?;

    // 写入表头
    let mut headers = vec!["班级"];
    if has_student_id {
        headers.push("学号");
    }
    headers.push("姓名");
    headers.push("性别");
    headers.extend(extra_field_names.iter().copied());
    headers.extend(subjects.iter().copied());
    headers.push("总分");

    for (col, header) in headers.iter().enumerate() {
        sheet.write_with_format(0, col as u16, *header, &header_format)?;
    }

    // 写入学生数据
    let mut row = 1u32;
    for class in classes {
        for student in &class.students {
            let mut col = 0u16;
            sheet.write(row, col, (class.id + 1) as f64)?;
            col += 1;

            // 学号（仅当有真实学号时）
            if has_student_id {
                let student_id = student.student_id.as_deref().unwrap_or("");
                sheet.write_string(row, col, student_id)?;
                col += 1;
            }

            // 姓名
            sheet.write_string(row, col, &student.name)?;
            col += 1;

            // 性别
            sheet.write_string(
                row,
                col,
                if student.gender == Gender::Male {
                    "男"
                } else {
                    "女"
                },
            )?;
            col += 1;

            // 额外字段
            for field_name in extra_field_names {
                let value = student
                    .extra_fields
                    .get(*field_name)
                    .map(|s| s.as_str())
                    .unwrap_or("");
                sheet.write_string(row, col, value)?;
                col += 1;
            }

            // 科目成绩
            for subject in subjects {
                let score = student.scores.get(*subject).copied().unwrap_or(0.0);
                sheet.write_with_format(row, col, score, &score_format)?;
                col += 1;
            }

            // 总分
            sheet.write_with_format(row, col, student.total_score, &score_format)?;
            row += 1;
        }
    }

    // 工作表2: 班级统计
    let stats_sheet = workbook.add_worksheet();
    stats_sheet.set_name("班级统计")?;

    // 写入表头
    let mut stat_headers: Vec<String> = vec![
        "班级".to_string(),
        "人数".to_string(),
        "男生".to_string(),
        "女生".to_string(),
        "男生比例".to_string(),
    ];
    stat_headers.extend(subjects.iter().map(|s| format!("{}_平均", s)));
    stat_headers.push("总分平均".to_string());

    for (col, header) in stat_headers.iter().enumerate() {
        stats_sheet.write_with_format(0, col as u16, header.as_str(), &header_format)?;
    }

    // 写入统计数据
    for (idx, class) in classes.iter().enumerate() {
        let row = (idx + 1) as u32;
        stats_sheet.write(row, 0, (class.id + 1) as f64)?;
        stats_sheet.write(row, 1, class.students.len() as f64)?;
        stats_sheet.write(row, 2, class.male_count() as f64)?;
        stats_sheet.write(row, 3, class.female_count() as f64)?;

        let ratio = class.gender_ratio() * 100.0;
        let ratio_format = Format::new().set_num_format("0.0\"%\"");
        stats_sheet.write_with_format(row, 4, ratio, &ratio_format)?;

        let mut col = 5u16;
        for subject in subjects {
            let avg = class.avg_subject_score(subject);
            stats_sheet.write_with_format(row, col, avg, &score_format)?;
            col += 1;
        }
        stats_sheet.write_with_format(row, col, class.avg_total_score(), &score_format)?;
    }

    workbook.save(file_path)?;
    Ok(())
}

/// 导出分班结果到 Excel（简化版，保持向后兼容）
pub fn export_classes_to_excel(
    classes: &[Class],
    file_path: &str,
    subjects: &[&str],
) -> anyhow::Result<()> {
    export_classes_to_excel_with_extras(classes, file_path, subjects, &[])
}

/// 从 CSV 读取学生数据（使用列配置）
pub fn read_students_from_csv_with_config(
    file_path: &str,
    config: &ExcelColumnConfig,
) -> anyhow::Result<Vec<Student>> {
    let file = File::open(file_path)?;
    let mut rdr = CsvReader::from_reader(file);

    // 读取表头（跳过）
    let _headers = rdr.headers()?;

    // 并行处理学生数据
    let records: Vec<_> = rdr.records().collect::<Result<Vec<_>, _>>()?;

    let students: Vec<Student> = records
        .par_iter()
        .enumerate()
        .filter_map(|(row_idx, record)| {
            // 读取姓名
            let name = record.get(config.name_column)?.trim().to_string();
            if name.is_empty() {
                return None;
            }

            // 读取学号（如果没有学号列，使用行号）
            let student_id = if let Some(col) = config.student_id_column {
                record.get(col).map(|s| s.trim().to_string())
            } else {
                Some(format!("R{}", row_idx + 1))
            };

            // 读取性别
            let gender_str = record.get(config.gender_column)?.trim();
            let gender = gender_str.parse::<Gender>().ok()?;

            // 读取科目成绩
            let mut scores = HashMap::with_capacity(config.subject_columns.len());
            for (subject, &col_idx) in &config.subject_columns {
                let score = record
                    .get(col_idx)
                    .and_then(|s| s.trim().parse::<f64>().ok())
                    .unwrap_or(0.0);
                scores.insert(subject.clone(), score);
            }

            // 读取或计算总分
            let total_score = if let Some(col) = config.total_score_column {
                record
                    .get(col)
                    .and_then(|s| s.trim().parse::<f64>().ok())
                    .unwrap_or(0.0)
            } else {
                scores.values().sum()
            };

            // 读取额外字段
            let mut extra_fields = HashMap::with_capacity(config.extra_columns.len());
            for (field_name, &col_idx) in &config.extra_columns {
                if let Some(value) = record.get(col_idx) {
                    extra_fields.insert(field_name.clone(), value.trim().to_string());
                }
            }

            Some(Student {
                name,
                student_id,
                gender,
                scores,
                total_score,
                extra_fields,
            })
        })
        .collect();

    if students.is_empty() {
        anyhow::bail!("未读取到任何学生数据");
    }

    Ok(students)
}

/// 导出分班结果到 CSV（带额外字段）
pub fn export_classes_to_csv_with_extras(
    classes: &[Class],
    file_path: &str,
    subjects: &[&str],
    extra_field_names: &[&str],
) -> anyhow::Result<()> {
    let file = File::create(file_path)?;
    let mut wtr = CsvWriter::from_writer(file);

    // 检查是否有真实学号
    let has_student_id = has_real_student_ids(classes);

    // 写入表头
    let mut headers = vec!["班级"];
    if has_student_id {
        headers.push("学号");
    }
    headers.push("姓名");
    headers.push("性别");
    headers.extend(extra_field_names.iter().copied());
    headers.extend(subjects.iter().copied());
    headers.push("总分");

    wtr.write_record(&headers)?;

    // 写入学生数据
    for class in classes {
        for student in &class.students {
            let mut record = Vec::new();

            // 班级（从 1 开始）
            record.push((class.id + 1).to_string());

            // 学号（仅当有真实学号时）
            if has_student_id {
                let student_id = student.student_id.as_deref().unwrap_or("");
                record.push(student_id.to_string());
            }

            // 姓名
            record.push(student.name.clone());

            // 性别
            record.push(if student.gender == Gender::Male {
                "男".to_string()
            } else {
                "女".to_string()
            });

            // 额外字段
            for field_name in extra_field_names {
                let value = student
                    .extra_fields
                    .get(*field_name)
                    .map(|s| s.as_str())
                    .unwrap_or("");
                record.push(value.to_string());
            }

            // 科目成绩
            for subject in subjects {
                let score = student.scores.get(*subject).copied().unwrap_or(0.0);
                record.push(format!("{:.1}", score));
            }

            // 总分
            record.push(format!("{:.1}", student.total_score));

            wtr.write_record(&record)?;
        }
    }

    wtr.flush()?;
    Ok(())
}

/// 导出分班结果到 CSV（简化版）
pub fn export_classes_to_csv(
    classes: &[Class],
    file_path: &str,
    subjects: &[&str],
) -> anyhow::Result<()> {
    export_classes_to_csv_with_extras(classes, file_path, subjects, &[])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_score() {
        use calamine::Data;

        let float_data = Data::Float(95.5);
        assert_eq!(parse_score(&float_data), 95.5);

        let int_data = Data::Int(90);
        assert_eq!(parse_score(&int_data), 90.0);

        let string_data = Data::String("85.5".to_string());
        assert_eq!(parse_score(&string_data), 85.5);

        let empty_data = Data::Empty;
        assert_eq!(parse_score(&empty_data), 0.0);
    }
}

use super::types::ColumnType;
use calamine::{Data, Reader, Xls, Xlsx, open_workbook};

#[allow(clippy::type_complexity)]
pub fn read_excel_all_data(
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

pub fn cell_to_string(cell: &Data) -> String {
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

pub fn infer_column_type(name: &str) -> ColumnType {
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

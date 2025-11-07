use crate::model::{Gender, Student};
use calamine::{open_workbook, DataType, Reader, Xlsx};
use rayon::prelude::*;
use std::collections::HashMap;
use std::error::Error;

/// 从 Excel 读取学生数据
pub fn read_students_from_excel(
    file_path: &str,
) -> Result<(Vec<Student>, Vec<String>), Box<dyn Error>> {
    let mut workbook: Xlsx<_> = open_workbook(file_path)?;
    let sheet_name = workbook.sheet_names()[0].clone();
    let range = workbook.worksheet_range(&sheet_name)?;

    let rows: Vec<_> = range.rows().collect();
    if rows.is_empty() {
        return Err("Excel 文件为空".into());
    }

    // 读取表头
    let subjects: Vec<String> = rows[0]
        .iter()
        .skip(2)
        .filter_map(|cell| cell.get_string().map(|s| s.to_string()))
        .collect();

    if subjects.is_empty() {
        return Err("未找到科目列".into());
    }

    // 并行处理学生数据
    let students: Vec<Student> = rows
        .par_iter()
        .skip(1)
        .filter_map(|row| {
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
                gender,
                scores,
                total_score: total,
            })
        })
        .collect();

    if students.is_empty() {
        return Err("未读取到任何学生数据".into());
    }

    Ok((students, subjects))
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

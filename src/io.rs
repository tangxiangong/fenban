use crate::model::{Class, Gender, Student};
use calamine::{DataType, Reader, Xlsx, open_workbook};
use rayon::prelude::*;
use rust_xlsxwriter::{Format, Workbook};
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

/// 导出分班结果到 Excel
pub fn export_classes_to_excel(
    classes: &[Class],
    file_path: &str,
    subjects: &[&str],
) -> Result<(), Box<dyn Error>> {
    let mut workbook = Workbook::new();

    // 创建格式
    let header_format = Format::new()
        .set_bold()
        .set_background_color(rust_xlsxwriter::Color::RGB(0xD9E1F2));

    let score_format = Format::new().set_num_format("0.0");

    // 工作表1: 分班结果详情
    let sheet = workbook.add_worksheet();
    sheet.set_name("分班结果")?;

    // 写入表头
    let mut headers = vec!["班级", "姓名", "性别"];
    headers.extend(subjects.iter().copied());
    headers.push("总分");

    for (col, header) in headers.iter().enumerate() {
        sheet.write_with_format(0, col as u16, *header, &header_format)?;
    }

    // 写入学生数据
    let mut row = 1u32;
    for class in classes {
        for student in &class.students {
            sheet.write(row, 0, class.id as f64)?;
            sheet.write_string(row, 1, &student.name)?;
            sheet.write_string(
                row,
                2,
                if student.gender == Gender::Male {
                    "男"
                } else {
                    "女"
                },
            )?;

            let mut col = 3u16;
            for subject in subjects {
                let score = student.scores.get(*subject).copied().unwrap_or(0.0);
                sheet.write_with_format(row, col, score, &score_format)?;
                col += 1;
            }
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
        stats_sheet.write(row, 0, class.id as f64)?;
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

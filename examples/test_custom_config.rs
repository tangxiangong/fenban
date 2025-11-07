use fenban::{
    DivideConfig, ExcelColumnConfig, divide_students, export_classes_to_excel_with_extras,
    read_students_from_excel_with_config,
};
use rust_xlsxwriter::Workbook;

fn main() {
    println!("=== 测试自定义列配置完整流程 ===\n");

    // 步骤1: 创建测试 Excel 文件
    println!("1. 创建测试 Excel 文件...");
    create_test_excel("测试数据_自定义列.xlsx").expect("创建测试文件失败");
    println!("   ✓ 已创建: 测试数据_自定义列.xlsx\n");

    // 步骤2: 配置列映射
    println!("2. 配置列映射...");
    let config = ExcelColumnConfig::builder()
        .student_id_column(0) // 学号
        .name_column(2) // 姓名
        .gender_column(3) // 性别
        .add_subject("语文".to_string(), 4)
        .add_subject("数学".to_string(), 5)
        .add_subject("英语".to_string(), 6)
        .add_extra_column("原班级".to_string(), 1)
        .build()
        .expect("配置构建失败");
    println!("   ✓ 配置完成\n");

    // 步骤3: 读取学生数据
    println!("3. 读取学生数据...");
    let students =
        read_students_from_excel_with_config("测试数据_自定义列.xlsx", &config).expect("读取失败");
    println!("   ✓ 成功读取 {} 名学生\n", students.len());

    // 显示前3个学生的信息
    println!("   前3名学生信息：");
    for student in students.iter().take(3) {
        println!(
            "     - {} (学号: {}, 性别: {:?}, 原班级: {}, 总分: {:.1})",
            student.name,
            student.student_id.as_deref().unwrap_or("无"),
            student.gender,
            student
                .extra_fields
                .get("原班级")
                .unwrap_or(&"".to_string()),
            student.total_score
        );
    }
    println!();

    // 步骤4: 执行分班
    println!("4. 执行分班...");
    let num_classes = 5;
    let divide_config = DivideConfig::new(num_classes);
    let classes = divide_students(&students, divide_config);
    println!("   ✓ 分成 {} 个班级\n", classes.len());

    // 显示分班统计
    println!("   分班统计：");
    for class in &classes {
        println!(
            "     班级{}: {} 人 (男 {} 女 {})",
            class.id,
            class.students.len(),
            class.male_count(),
            class.female_count()
        );
    }
    println!();

    // 步骤5: 导出结果
    println!("5. 导出结果到 Excel...");
    let subjects = vec!["语文", "数学", "英语"];
    let extra_fields = vec!["原班级"];

    export_classes_to_excel_with_extras(
        &classes,
        "分班结果_自定义列.xlsx",
        &subjects,
        &extra_fields,
    )
    .expect("导出失败");
    println!("   ✓ 成功导出到: 分班结果_自定义列.xlsx\n");

    println!("✓ 完整流程测试成功！");
}

fn create_test_excel(filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();

    // 写入表头
    sheet.write_string(0, 0, "学号")?;
    sheet.write_string(0, 1, "原班级")?;
    sheet.write_string(0, 2, "姓名")?;
    sheet.write_string(0, 3, "性别")?;
    sheet.write_string(0, 4, "语文")?;
    sheet.write_string(0, 5, "数学")?;
    sheet.write_string(0, 6, "英语")?;

    // 写入测试数据（100名学生）
    for i in 1..=100 {
        let student_id = format!("2024{:03}", i);
        let class_name = format!("高一{}班", (i - 1) % 10 + 1);
        let name = format!("学生{}", i);
        let gender = if i % 3 == 0 { "女" } else { "男" };
        let score_chinese = 90.0 + (i as f64 % 30.0);
        let score_math = 85.0 + (i as f64 % 35.0);
        let score_english = 80.0 + (i as f64 % 40.0);

        sheet.write_string(i as u32, 0, &student_id)?;
        sheet.write_string(i as u32, 1, &class_name)?;
        sheet.write_string(i as u32, 2, &name)?;
        sheet.write_string(i as u32, 3, gender)?;
        sheet.write_number(i as u32, 4, score_chinese)?;
        sheet.write_number(i as u32, 5, score_math)?;
        sheet.write_number(i as u32, 6, score_english)?;
    }

    workbook.save(filename)?;
    Ok(())
}

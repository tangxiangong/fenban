use fenban::core::{
    DivideConfig, ExcelColumnConfig, divide_students, export_classes_to_excel_with_extras,
    read_students_from_excel_with_config,
};

fn main() {
    let config = ExcelColumnConfig::builder()
        .name_column(0) // 姓名
        .gender_column(7) // 性别
        .add_subject("语文".to_string(), 1)
        .add_subject("数学".to_string(), 2)
        .add_subject("英语".to_string(), 3)
        .add_subject("政治".to_string(), 4)
        .add_subject("历史".to_string(), 5)
        .add_subject("地理".to_string(), 6)
        .add_extra_column("原班级".to_string(), 8)
        .build()
        .expect("配置构建失败");

    println!("读取学生数据...");
    let students = read_students_from_excel_with_config("成绩.xls", &config).expect("读取失败");
    println!("   ✓ 成功读取 {} 名学生\n", students.len());

    println!("执行分班...");
    let num_classes = 2;
    let divide_config = DivideConfig::new(num_classes);
    let classes = divide_students(&students, divide_config);
    println!("   ✓ 分成 {} 个班级\n", classes.len());

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
    println!("导出结果到 Excel...");
    let subjects = vec!["语文", "数学", "英语", "政治", "历史", "地理"];
    let extra_fields = vec!["原班级"];

    export_classes_to_excel_with_extras(&classes, "分班结果.xlsx", &subjects, &extra_fields)
        .expect("导出失败");
    println!("   ✓ 成功导出到: 分班结果.xlsx\n");

    println!("✓ 完整流程测试成功！");
}

use fenban::ExcelColumnConfig;

fn main() {
    println!("=== 自定义列配置示例 ===\n");

    // 方式1: 使用构建器自定义列配置
    // 假设 Excel 格式为：学号 | 班级 | 姓名 | 性别 | 语文 | 数学 | 英语 | 总分
    let config = ExcelColumnConfig::builder()
        .student_id_column(0) // 第1列：学号
        .name_column(2) // 第3列：姓名
        .gender_column(3) // 第4列：性别
        .add_subject("语文".to_string(), 4) // 第5列：语文
        .add_subject("数学".to_string(), 5) // 第6列：数学
        .add_subject("英语".to_string(), 6) // 第7列：英语
        .total_score_column(7) // 第8列：总分（可选，如不指定则自动计算）
        .add_extra_column("原班级".to_string(), 1) // 第2列：原班级（保留到输出）
        .build()
        .expect("配置构建失败");

    // 读取学生数据
    println!("读取配置：");
    println!("  学号列: {:?}", config.student_id_column);
    println!("  姓名列: {}", config.name_column);
    println!("  性别列: {}", config.gender_column);
    println!("  科目列: {:?}", config.subject_columns);
    println!("  额外列: {:?}", config.extra_columns);
    println!();

    // 注意：这个示例需要一个匹配格式的 Excel 文件
    // 如果要运行，请准备一个符合上述格式的 Excel 文件
    println!("注意：请准备一个符合以下格式的 Excel 文件：");
    println!("  列1: 学号");
    println!("  列2: 原班级");
    println!("  列3: 姓名");
    println!("  列4: 性别（男/女）");
    println!("  列5: 语文成绩");
    println!("  列6: 数学成绩");
    println!("  列7: 英语成绩");
    println!("  列8: 总分（可选）");
    println!();

    // 如果有实际文件，取消下面的注释来运行
    /*
    let students = match read_students_from_excel_with_config("学生数据.xlsx", &config) {
        Ok(students) => students,
        Err(e) => {
            eprintln!("读取失败: {}", e);
            return;
        }
    };

    println!("成功读取 {} 名学生\n", students.len());

    // 执行分班
    let num_classes = 10;
    let divide_config = DivideConfig::new(num_classes);
    let classes = divide_students(&students, divide_config);

    // 导出结果（包含额外字段）
    let subjects = vec!["语文", "数学", "英语"];
    let extra_fields = vec!["原班级"];

    match export_classes_to_excel_with_extras(&classes, "分班结果_自定义.xlsx", &subjects, &extra_fields) {
        Ok(_) => println!("✓ 成功导出到 分班结果_自定义.xlsx"),
        Err(e) => eprintln!("✗ 导出失败: {}", e),
    }
    */

    println!("示例配置已完成！");
}

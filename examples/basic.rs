use fenban::{
    calculate_detailed_statistics, calculate_statistics, divide_students, read_students_from_excel,
    validate_constraints, DivideConfig, Gender, Student,
};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 分班示例程序 ===\n");

    // 示例 1: 从 Excel 读取并分班
    example_from_excel()?;

    // 示例 2: 手动创建数据并分班
    example_manual_data()?;

    Ok(())
}

fn example_from_excel() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- 示例 1: 从 Excel 读取 ---");

    // 读取 Excel 文件
    let excel_path = "students.xlsx";
    match read_students_from_excel(excel_path) {
        Ok((students, subjects)) => {
            println!("✓ 成功读取 {} 名学生", students.len());
            println!("✓ 科目: {:?}", subjects);

            // 配置分班参数（分班数量由用户输入）
            let num_classes = 3; // 可以从用户输入获取
            let config = DivideConfig::new(num_classes).with_iterations(100000);

            println!("\n开始分班到 {} 个班级...", num_classes);
            let classes = divide_students(&students, config);

            // 输出结果
            print_results(&classes, &subjects);
        }
        Err(e) => {
            println!("✗ 读取 Excel 失败: {}", e);
            println!("  提示: 请确保 students.xlsx 存在且格式正确");
        }
    }

    println!();
    Ok(())
}

fn example_manual_data() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- 示例 2: 手动创建测试数据 ---");

    // 创建测试学生数据
    let students = create_test_students(120);
    println!("✓ 创建 {} 名测试学生", students.len());

    // 配置分班（分班数量由用户输入）
    let num_classes = 4; // 可以从用户输入获取
    let config = DivideConfig::new(num_classes).with_iterations(100000);

    println!("开始分班到 {} 个班级...", num_classes);
    let classes = divide_students(&students, config);

    // 获取科目列表
    let subjects = if !students.is_empty() {
        students[0].scores.keys().cloned().collect()
    } else {
        vec![]
    };

    // 输出结果
    print_results(&classes, &subjects);

    Ok(())
}

fn print_results(classes: &[fenban::Class], subjects: &[String]) {
    println!("\n=== 分班结果 ===");

    // 约束验证（重点：检查是否满足硬约束）
    let validation = validate_constraints(classes);
    println!("\n【约束验证】");
    println!(
        "  总分约束 (差值≤1): {}",
        if validation.score_constraints_met {
            "✓ 满足"
        } else {
            "✗ 不满足"
        }
    );
    println!("    最大差值: {:.2} 分", validation.max_score_diff);
    println!(
        "  性别约束 (比例差≤0.2): {}",
        if validation.gender_constraints_met {
            "✓ 满足"
        } else {
            "✗ 不满足"
        }
    );
    println!("    最大比例差: {:.2}", validation.max_gender_ratio_diff);
    println!("  各科约束 (差值≤1):");
    for (subject, diff) in &validation.subject_max_diffs {
        let met = *diff <= 1.0;
        println!(
            "    {} {}: 差值 {:.2} 分",
            if met { "✓" } else { "✗" },
            subject,
            diff
        );
    }

    // 基础统计
    let stats = calculate_statistics(classes);
    println!("\n【整体统计】");
    println!("  平均总分: {:.2}", stats.mean_score);
    println!("  标准差: {:.2}", stats.std_dev);
    println!("  方差: {:.2}", stats.variance);
    println!(
        "  分数范围: {:.2} - {:.2}",
        stats.min_score, stats.max_score
    );

    // 各科统计
    println!("\n【各科成绩统计】");
    for subject_stat in &stats.subject_stats {
        println!(
            "  {}: 均值={:.2}, 标准差={:.2}",
            subject_stat.subject_name, subject_stat.mean_score, subject_stat.std_dev
        );
    }

    // 详细统计
    let detailed = calculate_detailed_statistics(classes);
    println!("\n【性别平衡统计】");
    println!("  男生方差: {:.2}", detailed.gender_balance.male_variance);
    println!("  女生方差: {:.2}", detailed.gender_balance.female_variance);
    println!("  比例方差: {:.4}", detailed.gender_balance.ratio_variance);

    // 各班详情
    println!("\n【各班详情】");
    for (idx, class) in classes.iter().enumerate() {
        println!("\n班级 {}:", idx + 1);
        println!("  总人数: {}", class.students.len());
        println!(
            "  男女比例: {} 男 / {} 女",
            class.male_count(),
            class.female_count()
        );
        println!("  平均总分: {:.2}", class.avg_total_score());

        // 计算男女比例
        let total = class.students.len() as f64;
        let male_ratio = if total > 0.0 {
            class.male_count() as f64 / total
        } else {
            0.0
        };
        println!("  男生比例: {:.2}", male_ratio);

        // 各科平均分
        print!("  各科均分: ");
        for subject in subjects {
            print!("{}={:.1} ", subject, class.avg_subject_score(subject));
        }
        println!();

        // 显示前 3 名学生
        println!("  学生示例:");
        for (i, student) in class.students.iter().take(3).enumerate() {
            println!(
                "    {}. {} ({:?}) - 总分: {:.0}",
                i + 1,
                student.name,
                student.gender,
                student.total_score
            );
        }
        if class.students.len() > 3 {
            println!("    ... 还有 {} 名学生", class.students.len() - 3);
        }
    }

    println!("\n=== 分班完成 ===\n");
}

fn create_test_students(count: usize) -> Vec<Student> {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    (0..count)
        .map(|i| {
            let gender = if i % 2 == 0 {
                Gender::Male
            } else {
                Gender::Female
            };

            let mut scores = HashMap::new();
            scores.insert("语文".to_string(), rng.gen_range(90.0..140.0));
            scores.insert("数学".to_string(), rng.gen_range(90.0..150.0));
            scores.insert("英语".to_string(), rng.gen_range(90.0..140.0));
            scores.insert("物理".to_string(), rng.gen_range(70.0..100.0));
            scores.insert("化学".to_string(), rng.gen_range(70.0..100.0));

            let total: f64 = scores.values().sum();

            Student {
                name: format!("学生{:03}", i + 1),
                gender,
                scores,
                total_score: total,
            }
        })
        .collect()
}

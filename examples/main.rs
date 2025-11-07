use fenban::algorithm::{DivideConfig, divide_students, validate_constraints_with_params};
use fenban::export_classes_to_excel;
use fenban::model::{Gender, Student};
use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Normal};
use std::collections::HashMap;

fn main() {
    println!("=== fenban 高中生均衡分班系统 ===\n");

    let students = generate_students(2000);
    let num_classes = 40;

    println!("  学生总数: {}", students.len());
    println!("  班级数量: {}", num_classes);
    println!("  约束要求:");
    println!("    - 平均分差值 ≤ 1.0 分");
    println!("    - 性别比例差 ≤ 0.1 (10%)");
    println!("    - 班级人数差 ≤ 5 人");

    let config = DivideConfig::new(num_classes);

    let classes = divide_students(&students, config.clone());

    let validation = validate_constraints_with_params(&classes, &config.optimization_params);

    println!("\n  结果:");
    println!(
        "    总分约束: 最大差值: {:.2} 分",
        validation.max_score_diff
    );
    println!(
        "    性别约束: 最大差值: {:.1}%",
        validation.max_gender_ratio_diff * 100.0
    );

    // // 显示每个班级的统计
    // println!("\n  班级详情:");
    let all_subjects = vec![
        "语文", "数学", "外语", "物理", "化学", "生物", "政治", "历史", "地理",
    ];

    // for class in &classes {
    //     println!(
    //         "    班级{}: {} 人 (男 {} 女 {}), 总分 {:.1}, 男生比例 {:.0}%",
    //         class.id,
    //         class.students.len(),
    //         class.male_count(),
    //         class.female_count(),
    //         class.avg_total_score(),
    //         class.gender_ratio() * 100.0
    //     );

    //     // 输出单科平均分
    //     print!("      单科平均: ");
    //     for (i, subject) in all_subjects.iter().enumerate() {
    //         let avg = class.avg_subject_score(subject);
    //         print!("{} {:.1}", subject, avg);
    //         if i < all_subjects.len() - 1 {
    //             print!(", ");
    //         }
    //     }
    //     println!();
    // }

    // 导出结果到 Excel
    println!("\n  导出结果到 Excel...");
    let export_result = export_classes_to_excel(&classes, "分班结果.xlsx", &all_subjects);
    match export_result {
        Ok(_) => println!("    ✓ 成功导出到 分班结果.xlsx"),
        Err(e) => println!("    ✗ 导出失败: {}", e),
    }
}

/// 生成测试学生数据（正态分布）
fn generate_students(count: usize) -> Vec<Student> {
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);

    // 主科：语文、数学、外语（满分 150）
    let main_subjects = vec!["语文", "数学", "外语"];
    // 副科：物理、化学、生物、政治、历史、地理（满分 100）
    let other_subjects = vec!["物理", "化学", "生物", "政治", "历史", "地理"];

    // 主科正态分布（均值 80.5，标准差 15）
    let normal_main = Normal::new(80.5, 15.0).unwrap();
    // 副科正态分布（均值 65，标准差 10）
    let normal_sub = Normal::new(65.0, 10.0).unwrap();

    (0..count)
        .map(|i| {
            let gender = if rng.random_bool(0.6) {
                Gender::Male
            } else {
                Gender::Female
            };

            let mut scores = HashMap::new();
            // 主科分数（满分 150）
            for subject in &main_subjects {
                let score: f64 = normal_main.sample(&mut rng);
                let score = score.clamp(0.0, 150.0);
                scores.insert(subject.to_string(), score);
            }
            // 副科分数（满分 100）
            for subject in &other_subjects {
                let score: f64 = normal_sub.sample(&mut rng);
                let score = score.clamp(0.0, 100.0);
                scores.insert(subject.to_string(), score);
            }

            Student::new(format!("学生{}", i + 1), gender, scores)
        })
        .collect()
}

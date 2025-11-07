use fenban::{divide_students, validate_constraints, DivideConfig, Gender, Student};
use rand::SeedableRng;
use rand_distr::{Distribution, Normal};
use std::collections::HashMap;
use std::time::Instant;

/// 生成测试数据（使用正态分布）
fn generate_test_students(count: usize) -> Vec<Student> {
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);

    // 正态分布：语数外 均值100±15，其他科目 均值75±12
    let normal_main = Normal::new(100.0, 15.0).unwrap();
    let normal_other = Normal::new(75.0, 12.0).unwrap();

    (0..count)
        .map(|i| {
            let mut scores = HashMap::new();

            // 语文、数学、外语 (150分制)
            let yuwen: f64 = normal_main.sample(&mut rng);
            scores.insert("语文".to_string(), yuwen.clamp(0.0, 150.0));

            let shuxue: f64 = normal_main.sample(&mut rng);
            scores.insert("数学".to_string(), shuxue.clamp(0.0, 150.0));

            let waiyu: f64 = normal_main.sample(&mut rng);
            scores.insert("外语".to_string(), waiyu.clamp(0.0, 150.0));

            // 物理、化学、生物、政治、历史、地理 (100分制)
            let wuli: f64 = normal_other.sample(&mut rng);
            scores.insert("物理".to_string(), wuli.clamp(0.0, 100.0));

            let huaxue: f64 = normal_other.sample(&mut rng);
            scores.insert("化学".to_string(), huaxue.clamp(0.0, 100.0));

            let shengwu: f64 = normal_other.sample(&mut rng);
            scores.insert("生物".to_string(), shengwu.clamp(0.0, 100.0));

            let zhengzhi: f64 = normal_other.sample(&mut rng);
            scores.insert("政治".to_string(), zhengzhi.clamp(0.0, 100.0));

            let lishi: f64 = normal_other.sample(&mut rng);
            scores.insert("历史".to_string(), lishi.clamp(0.0, 100.0));

            let dili: f64 = normal_other.sample(&mut rng);
            scores.insert("地理".to_string(), dili.clamp(0.0, 100.0));

            Student::new(
                format!("学生{:04}", i + 1),
                if i % 2 == 0 {
                    Gender::Male
                } else {
                    Gender::Female
                },
                scores,
            )
        })
        .collect()
}

fn print_validation_results(classes: &[fenban::Class]) {
    let validation = validate_constraints(classes);

    println!("\n约束验证:");
    println!(
        "  总分约束: {} (差值 {:.2} 分)",
        if validation.score_constraints_met {
            "✅"
        } else {
            "❌"
        },
        validation.max_score_diff
    );
    println!(
        "  性别约束: {} (差值 {:.2})",
        if validation.gender_constraints_met {
            "✅"
        } else {
            "❌"
        },
        validation.max_gender_ratio_diff
    );

    let mut met_count = 0;
    for (subject, diff) in &validation.subject_max_diffs {
        if *diff <= 1.0 {
            met_count += 1;
        }
    }
    println!(
        "  科目约束: {}/{} 满足",
        met_count,
        validation.subject_max_diffs.len()
    );

    if validation.score_constraints_met && validation.gender_constraints_met && met_count == 9 {
        println!("\n✅ 所有约束均满足");
    } else {
        println!("\n⚠️  部分约束未满足，详细:");
        for (subject, diff) in &validation.subject_max_diffs {
            if *diff > 1.0 {
                println!("    {} 差值: {:.2} 分", subject, diff);
            }
        }
    }
}

fn benchmark(students: &[Student], num_classes: usize, iterations: usize, label: &str) {
    println!("\n{}", "=".repeat(70));
    println!("{} - {} 学生 / {} 班级", label, students.len(), num_classes);
    println!("{}", "=".repeat(70));

    let config = DivideConfig::new(num_classes).with_iterations(iterations);
    let start = Instant::now();
    let classes = divide_students(students, config);
    let duration = start.elapsed();

    println!("耗时: {:.3} 秒", duration.as_secs_f64());

    // 打印各班统计
    for (i, class) in classes.iter().enumerate() {
        println!(
            "  班级{:2}: {:3}人 (男{:2}/女{:2}) 平均{:.1}分",
            i + 1,
            class.students.len(),
            class.male_count(),
            class.female_count(),
            class.avg_total_score()
        );
    }

    print_validation_results(&classes);
}

fn main() {
    println!("\n{}", "=".repeat(70));
    println!("fenban 性能测试 - 正态分布数据");
    println!("{}", "=".repeat(70));

    // 测试 1: 100 学生
    let students_100 = generate_test_students(100);
    benchmark(&students_100, 4, 300000, "小规模测试 (100学生)");

    // 测试 2: 500 学生
    let students_500 = generate_test_students(500);
    benchmark(&students_500, 10, 400000, "中等规模测试 (500学生)");

    // 测试 3: 1000 学生
    let students_1000 = generate_test_students(1000);
    benchmark(&students_1000, 20, 400000, "大规模测试 (1000学生)");

    // 测试 4: 5000 学生（目标测试）
    println!("\n{}", "=".repeat(70));
    println!("🎯 目标测试: 5000 学生在 5 分钟内完成");
    println!("{}", "=".repeat(70));

    let students_5000 = generate_test_students(5000);
    benchmark(&students_5000, 50, 500000, "超大规模测试 (5000学生)");

    println!("\n{}", "=".repeat(70));
    println!("测试完成");
    println!("{}", "=".repeat(70));
}

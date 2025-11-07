use fenban::algorithm::{
    DivideConfig, OptimizationParams, divide_students, validate_constraints_with_params,
};
use fenban::model::{Gender, Student};
use rand::SeedableRng;
use rand_distr::{Distribution, Normal};
use std::collections::HashMap;
use std::time::Instant;

fn main() {
    println!("=== fenban 高中生均衡分班系统 ===\n");

    // ========== 示例 1: 基本使用（默认参数） ==========
    println!("【示例 1】基本使用 - 默认参数");
    basic_usage();

    println!("\n{}\n", "=".repeat(60));

    // ========== 示例 2: 参数配置 ==========
    println!("【示例 2】参数配置 - 使用不同预设");
    parameter_configuration();

    println!("\n{}\n", "=".repeat(60));

    // ========== 示例 3: 性能测试 ==========
    println!("【示例 3】性能测试 - 不同规模数据");
    performance_test();

    println!("\n{}\n", "=".repeat(60));

    // ========== 总结 ==========
    println!("【总结】");
    println!("1. 默认参数适合大多数场景，能够满足所有约束");
    println!("2. 使用自适应参数可以根据数据规模自动调整（推荐）");
    println!("3. 迭代次数建议设置为 1,000,000 以上以确保收敛");
    println!("4. 必须使用 --release 模式编译，性能提升 300+ 倍");
    println!("\n运行 Excel 示例：cargo run --release --example basic");
}

/// 示例 1: 基本使用
fn basic_usage() {
    let students = generate_students(300);
    let num_classes = 6;

    println!("  学生总数: {}", students.len());
    println!("  班级数量: {}", num_classes);
    println!("  约束要求:");
    println!("    - 平均分差值 ≤ 1.0 分");
    println!("    - 性别比例差 ≤ 0.1 (10%)");
    println!("    - 班级人数差 ≤ 5 人");

    // 使用默认配置
    let config = DivideConfig::new(num_classes).with_iterations(1_000_000);

    let start = Instant::now();
    let classes = divide_students(&students, config.clone());
    let duration = start.elapsed();

    let validation = validate_constraints_with_params(&classes, &config.optimization_params);

    println!("\n  结果:");
    println!("    耗时: {:.2} 秒", duration.as_secs_f64());
    println!(
        "    总分约束: {} (最大差值: {:.2} 分)",
        if validation.score_constraints_met {
            "✓ 满足"
        } else {
            "✗ 不满足"
        },
        validation.max_score_diff
    );
    println!(
        "    性别约束: {} (最大差值: {:.1}%)",
        if validation.gender_constraints_met {
            "✓ 满足"
        } else {
            "✗ 不满足"
        },
        validation.max_gender_ratio_diff * 100.0
    );

    // 显示每个班级的统计
    println!("\n  班级详情:");
    for class in &classes {
        println!(
            "    班级{}: {} 人 (男 {} 女 {}), 平均分 {:.1}, 男生比例 {:.0}%",
            class.id,
            class.students.len(),
            class.male_count(),
            class.female_count(),
            class.avg_total_score(),
            class.gender_ratio() * 100.0
        );
    }
}

/// 示例 2: 参数配置
fn parameter_configuration() {
    let students = generate_students(300);
    let num_classes = 6;

    println!(
        "  学生总数: {}, 班级数量: {}\n",
        students.len(),
        num_classes
    );

    // 测试不同的参数配置
    let configs = vec![
        ("默认参数", OptimizationParams::default(), 1_000_000),
        ("宽松参数", OptimizationParams::relaxed(), 500_000),
        ("严格参数", OptimizationParams::strict(), 1_500_000),
        (
            "自适应参数",
            OptimizationParams::adaptive(students.len()),
            1_000_000,
        ),
    ];

    for (name, params, iterations) in configs {
        let config = DivideConfig::new(num_classes)
            .with_iterations(iterations)
            .with_optimization_params(params.clone());

        let start = Instant::now();
        let classes = divide_students(&students, config);
        let duration = start.elapsed();

        let validation = validate_constraints_with_params(&classes, &params);

        println!("  {} ({} 次迭代):", name, iterations);
        println!("    耗时: {:.2} 秒", duration.as_secs_f64());
        println!(
            "    总分约束 {}: {:.2} 分 ≤ {:.2} 分",
            if validation.score_constraints_met {
                "✓"
            } else {
                "✗"
            },
            validation.max_score_diff,
            params.max_score_diff
        );
        println!(
            "    性别约束 {}: {:.1}% ≤ {:.1}%",
            if validation.gender_constraints_met {
                "✓"
            } else {
                "✗"
            },
            validation.max_gender_ratio_diff * 100.0,
            params.max_gender_ratio_diff * 100.0
        );
        println!();
    }
}

/// 示例 3: 性能测试
fn performance_test() {
    let test_cases = vec![
        (100, 4, 500_000),
        (300, 6, 1_000_000),
        (500, 10, 1_500_000),
        (1000, 20, 2_000_000),
    ];

    println!("  规模测试（使用自适应参数）:\n");
    println!(
        "  {:<10} {:<10} {:<15} {:<10} {:<12} {:<12}",
        "学生数", "班级数", "迭代次数", "耗时", "分数约束", "性别约束"
    );
    println!("  {}", "-".repeat(75));

    for (student_count, num_classes, iterations) in test_cases {
        let students = generate_students(student_count);
        let params = OptimizationParams::adaptive(student_count);
        let config = DivideConfig::new(num_classes)
            .with_iterations(iterations)
            .with_optimization_params(params.clone());

        let start = Instant::now();
        let classes = divide_students(&students, config);
        let duration = start.elapsed();

        let validation = validate_constraints_with_params(&classes, &params);

        println!(
            "  {:<10} {:<10} {:<15} {:<10.2}s {:<12} {:<12}",
            student_count,
            num_classes,
            iterations,
            duration.as_secs_f64(),
            if validation.score_constraints_met {
                "✓ 满足"
            } else {
                "✗ 不满足"
            },
            if validation.gender_constraints_met {
                "✓ 满足"
            } else {
                "✗ 不满足"
            }
        );
    }

    println!("\n  注意：实际性能取决于硬件配置和数据分布");
}

/// 生成测试学生数据（正态分布）
fn generate_students(count: usize) -> Vec<Student> {
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);

    // 9 个科目的正态分布（均值 75，标准差 12）
    let subjects = vec![
        "语文", "数学", "外语", "物理", "化学", "生物", "政治", "历史", "地理",
    ];

    let normal = Normal::new(75.0, 12.0).unwrap();

    (0..count)
        .map(|i| {
            let gender = if i % 2 == 0 {
                Gender::Male
            } else {
                Gender::Female
            };

            let mut scores = HashMap::new();
            for subject in &subjects {
                let score: f64 = normal.sample(&mut rng);
                let score = score.clamp(0.0, 100.0);
                scores.insert(subject.to_string(), score);
            }

            Student::new(format!("学生{}", i + 1), gender, scores)
        })
        .collect()
}

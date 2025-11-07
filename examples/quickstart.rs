//! 快速入门示例
//!
//! 运行方式：
//! ```bash
//! cargo run --release --example quickstart
//! ```

use fenban::{divide_students, validate_constraints, DivideConfig, Gender, Student};
use rand::SeedableRng;
use rand_distr::{Distribution, Normal};
use std::collections::HashMap;

fn main() {
    println!("\n{}", "=".repeat(70));
    println!("fenban 快速入门");
    println!("{}\n", "=".repeat(70));

    println!("步骤 1: 准备学生数据...");

    let students = generate_sample_students(200);

    println!("✅ 成功生成 {} 名学生数据", students.len());
    println!("   科目: 语文、数学、外语、物理、化学、生物、政治、历史、地理");

    // 统计性别分布
    let male_count = students.iter().filter(|s| s.gender == Gender::Male).count();
    let female_count = students.len() - male_count;
    println!(
        "   性别分布: 男生 {} 人 ({:.1}%), 女生 {} 人 ({:.1}%)",
        male_count,
        male_count as f64 / students.len() as f64 * 100.0,
        female_count,
        female_count as f64 / students.len() as f64 * 100.0
    );

    println!("\n{}", "=".repeat(70));
    println!("步骤 2: 执行分班...");

    let config = DivideConfig::new(6).with_iterations(300000);
    let start = std::time::Instant::now();
    let classes = divide_students(&students, config);
    let duration = start.elapsed();

    println!("完成！耗时: {:.3} 秒", duration.as_secs_f64());

    println!("\n{}", "=".repeat(70));
    println!("步骤 3: 验证约束...");

    let validation = validate_constraints(&classes);

    println!("\n📊 约束验证结果:");
    println!(
        "   总分约束: {} (最大差值: {:.2} 分)",
        if validation.score_constraints_met {
            "✅ 满足"
        } else {
            "❌ 不满足"
        },
        validation.max_score_diff
    );

    println!(
        "   性别约束: {} (最大比例差: {:.2})",
        if validation.gender_constraints_met {
            "✅ 满足"
        } else {
            "❌ 不满足"
        },
        validation.max_gender_ratio_diff
    );

    // 检查各科约束
    let mut all_subjects_met = true;
    let mut subjects_met_count = 0;

    println!("   科目约束:");
    for (subject, diff) in &validation.subject_max_diffs {
        let met = *diff <= 1.0;
        if met {
            subjects_met_count += 1;
        } else {
            all_subjects_met = false;
        }
        println!(
            "     • {} {} (差值: {:.2} 分)",
            subject,
            if met { "✅" } else { "❌" },
            diff
        );
    }

    // 综合评价
    println!("\n🎯 综合评价:");
    if validation.score_constraints_met && validation.gender_constraints_met && all_subjects_met {
        println!("   ✅ 所有约束均满足！分班效果优秀！");
    } else if validation.max_score_diff < 2.0
        && validation.max_gender_ratio_diff < 0.3
        && subjects_met_count >= 7
    {
        println!("   ⚠️  接近满足所有约束，分班效果良好");
        println!("   💡 提示: 可以尝试增加迭代次数或多次运行选最优");
    } else {
        println!("   ❌ 部分约束未满足");
        println!("   💡 建议: 增加迭代次数或检查数据分布");
    }

    println!("\n{}", "=".repeat(70));
    println!("步骤 4: 各班详情\n");

    for (idx, class) in classes.iter().enumerate() {
        let male_count = class.male_count();
        let female_count = class.female_count();
        let total = class.students.len();
        let avg_score = class.avg_total_score();

        println!("班级 {:2}:", idx + 1);
        println!(
            "  人数: {:3} 人 (男 {:2} / 女 {:2}) 男生比例: {:.1}%",
            total,
            male_count,
            female_count,
            male_count as f64 / total as f64 * 100.0
        );
        println!("  平均总分: {:.1}", avg_score);

        // 打印各科平均分
        print!("  各科平均: ");
        let subjects = [
            "语文", "数学", "外语", "物理", "化学", "生物", "政治", "历史", "地理",
        ];
        for subject in &subjects {
            let avg = class.avg_subject_score(subject);
            print!("{} {:.0} ", subject, avg);
        }
        println!("\n");
    }

    println!("\n{}", "=".repeat(70));
    println!("完成！");
    println!("{}\n", "=".repeat(70));
}

/// 生成示例学生数据（使用正态分布）
fn generate_sample_students(count: usize) -> Vec<Student> {
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let normal_main = Normal::new(100.0, 15.0).unwrap();
    let normal_other = Normal::new(75.0, 12.0).unwrap();

    (0..count)
        .map(|i| {
            let mut scores = HashMap::new();

            let yuwen: f64 = normal_main.sample(&mut rng);
            scores.insert("语文".to_string(), yuwen.clamp(0.0, 150.0));

            let shuxue: f64 = normal_main.sample(&mut rng);
            scores.insert("数学".to_string(), shuxue.clamp(0.0, 150.0));

            let waiyu: f64 = normal_main.sample(&mut rng);
            scores.insert("外语".to_string(), waiyu.clamp(0.0, 150.0));

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
                format!("学生{:03}", i + 1),
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

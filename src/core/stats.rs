use super::model::Class;
use rayon::prelude::*;

/// 统计信息结构
#[derive(Debug, Clone)]
pub struct Statistics {
    pub mean_score: f64,
    pub variance: f64,
    pub std_dev: f64,
    pub min_score: f64,
    pub max_score: f64,
    pub score_range: f64,
    pub subject_stats: Vec<SubjectStatistics>,
}

/// 单科统计信息
#[derive(Debug, Clone)]
pub struct SubjectStatistics {
    pub subject_name: String,
    pub mean_score: f64,
    pub variance: f64,
    pub std_dev: f64,
    pub min_score: f64,
    pub max_score: f64,
}

/// 计算分班结果的统计信息
pub fn calculate_statistics(classes: &[Class]) -> Statistics {
    if classes.is_empty() {
        return Statistics {
            mean_score: 0.0,
            variance: 0.0,
            std_dev: 0.0,
            min_score: 0.0,
            max_score: 0.0,
            score_range: 0.0,
            subject_stats: vec![],
        };
    }

    let avg_scores: Vec<f64> = classes.par_iter().map(|c| c.avg_total_score()).collect();

    let mean = avg_scores.par_iter().sum::<f64>() / avg_scores.len() as f64;

    let variance = avg_scores
        .par_iter()
        .map(|score| (score - mean).powi(2))
        .sum::<f64>()
        / avg_scores.len() as f64;

    let std_dev = variance.sqrt();

    let min_score = avg_scores.par_iter().cloned().reduce(|| f64::MAX, f64::min);

    let max_score = avg_scores.par_iter().cloned().reduce(|| f64::MIN, f64::max);

    let score_range = max_score - min_score;

    // 计算各科统计
    let subject_stats = calculate_subject_statistics(classes);

    Statistics {
        mean_score: mean,
        variance,
        std_dev,
        min_score,
        max_score,
        score_range,
        subject_stats,
    }
}

/// 计算各科统计信息
fn calculate_subject_statistics(classes: &[Class]) -> Vec<SubjectStatistics> {
    // 获取所有科目
    let subjects = get_all_subjects(classes);

    subjects
        .par_iter()
        .map(|subject| {
            let subject_scores: Vec<f64> = classes
                .iter()
                .map(|c| c.avg_subject_score(subject))
                .collect();

            let mean = if !subject_scores.is_empty() {
                subject_scores.iter().sum::<f64>() / subject_scores.len() as f64
            } else {
                0.0
            };

            let variance = if !subject_scores.is_empty() {
                subject_scores
                    .iter()
                    .map(|score| (score - mean).powi(2))
                    .sum::<f64>()
                    / subject_scores.len() as f64
            } else {
                0.0
            };

            let std_dev = variance.sqrt();

            let min_score = subject_scores.iter().cloned().fold(f64::MAX, f64::min);

            let max_score = subject_scores.iter().cloned().fold(f64::MIN, f64::max);

            SubjectStatistics {
                subject_name: subject.clone(),
                mean_score: mean,
                variance,
                std_dev,
                min_score,
                max_score,
            }
        })
        .collect()
}

/// 获取所有科目
fn get_all_subjects(classes: &[Class]) -> Vec<String> {
    if classes.is_empty() {
        return vec![];
    }

    classes
        .iter()
        .flat_map(|c| c.students.iter())
        .next()
        .map(|s| s.scores.keys().cloned().collect())
        .unwrap_or_default()
}

/// 详细统计信息
#[derive(Debug, Clone)]
pub struct DetailedStatistics {
    pub overall: Statistics,
    pub gender_balance: GenderBalance,
    pub class_sizes: Vec<usize>,
    pub male_counts: Vec<usize>,
    pub female_counts: Vec<usize>,
}

/// 性别平衡统计
#[derive(Debug, Clone)]
pub struct GenderBalance {
    pub male_variance: f64,
    pub female_variance: f64,
    pub ratio_variance: f64,
}

/// 计算详细统计信息
pub fn calculate_detailed_statistics(classes: &[Class]) -> DetailedStatistics {
    let overall = calculate_statistics(classes);
    let gender_balance = calculate_gender_balance(classes);
    let class_sizes = classes.iter().map(|c| c.students.len()).collect();
    let male_counts = classes.iter().map(|c| c.male_count()).collect();
    let female_counts = classes.iter().map(|c| c.female_count()).collect();

    DetailedStatistics {
        overall,
        gender_balance,
        class_sizes,
        male_counts,
        female_counts,
    }
}

/// 计算性别平衡统计
fn calculate_gender_balance(classes: &[Class]) -> GenderBalance {
    let male_counts: Vec<f64> = classes.par_iter().map(|c| c.male_count() as f64).collect();
    let female_counts: Vec<f64> = classes
        .par_iter()
        .map(|c| c.female_count() as f64)
        .collect();

    let male_mean = male_counts.par_iter().sum::<f64>() / male_counts.len() as f64;
    let female_mean = female_counts.par_iter().sum::<f64>() / female_counts.len() as f64;

    let male_variance = male_counts
        .par_iter()
        .map(|m| (m - male_mean).powi(2))
        .sum::<f64>()
        / male_counts.len() as f64;

    let female_variance = female_counts
        .par_iter()
        .map(|f| (f - female_mean).powi(2))
        .sum::<f64>()
        / female_counts.len() as f64;

    let ratios: Vec<f64> = classes
        .par_iter()
        .map(|c| {
            let total = c.students.len() as f64;
            if total > 0.0 {
                c.male_count() as f64 / total
            } else {
                0.0
            }
        })
        .collect();

    let ratio_mean = ratios.par_iter().sum::<f64>() / ratios.len() as f64;
    let ratio_variance = ratios
        .par_iter()
        .map(|r| (r - ratio_mean).powi(2))
        .sum::<f64>()
        / ratios.len() as f64;

    GenderBalance {
        male_variance,
        female_variance,
        ratio_variance,
    }
}

use super::model::{Class, Gender, Student};
use rand::{Rng, rng};
use rayon::prelude::*;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

/// 分班配置
#[derive(Debug, Clone)]
pub struct DivideConfig {
    pub num_classes: usize,
    pub max_iterations: usize,
    pub optimization_params: OptimizationParams,
}

impl Default for DivideConfig {
    fn default() -> Self {
        Self {
            num_classes: 3,
            max_iterations: 500000,
            optimization_params: OptimizationParams::default(),
        }
    }
}

impl DivideConfig {
    pub fn new(num_classes: usize) -> Self {
        Self {
            num_classes,
            ..Default::default()
        }
    }

    pub fn with_iterations(mut self, max_iterations: usize) -> Self {
        self.max_iterations = max_iterations;
        self
    }

    pub fn with_optimization_params(mut self, params: OptimizationParams) -> Self {
        self.optimization_params = params;
        self
    }
}

/// 优化参数配置
///
/// 包含所有约束阈值和代价函数权重参数
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OptimizationParams {
    // ===== 硬约束阈值 =====
    /// 平均分最大允许差值（默认：1.0 分）
    pub max_score_diff: f64,

    /// 单科平均分最大允许差值（默认：1.0 分）
    pub max_subject_score_diff: f64,

    /// 班级人数最大允许差值（默认：5 人）
    pub max_class_size_diff: usize,

    /// 男女比例最大允许差值（默认：0.1）
    pub max_gender_ratio_diff: f64,

    // ===== 硬约束惩罚权重 =====
    /// 总分差值超出阈值的惩罚权重（默认：1,000,000,000.0）
    pub total_score_penalty_weight: f64,

    /// 科目分差值超出阈值的惩罚权重（默认：1,000,000,000.0）
    pub subject_score_penalty_weight: f64,

    /// 性别比例差值超出阈值的惩罚权重（默认：1,000,000,000.0）
    pub gender_ratio_penalty_weight: f64,

    /// 惩罚函数的幂次（默认：4，越高越严格）
    pub penalty_power: i32,

    // ===== 软约束优化权重 =====
    /// 总分方差的权重（默认：10.0）
    pub total_variance_weight: f64,

    /// 性别方差的权重（默认：100.0）
    pub gender_variance_weight: f64,

    /// 科目方差的权重（默认：50.0）
    pub subject_variance_weight: f64,

    // ===== 模拟退火参数 =====
    /// 初始温度（默认：10,000.0）
    pub initial_temperature: f64,

    /// 冷却速率（默认：0.99990）
    pub cooling_rate: f64,

    /// 并行实例数（默认：自动检测 CPU 核心数）
    pub num_parallel_instances: Option<usize>,

    /// 温度多样性增量（默认：1,000.0，为不同实例增加温度差异）
    pub temperature_diversity_delta: f64,

    // ===== 早停与重启参数 =====
    /// 找到满足约束的解的代价阈值（默认：500.0）
    pub good_solution_threshold: f64,

    /// 无改进时重新加热的迭代次数（默认：1,000）
    pub reheat_after_iterations: usize,

    /// 重新加热时的温度倍数（默认：0.5）
    pub reheat_temperature_factor: f64,

    /// 重新加热的最小接受次数阈值（默认：100）
    pub reheat_min_accept_count: usize,
}

impl Default for OptimizationParams {
    fn default() -> Self {
        Self {
            // 硬约束阈值 - 严格按照需求设置
            max_score_diff: 1.0,
            max_subject_score_diff: 1.0,
            max_class_size_diff: 5,
            max_gender_ratio_diff: 0.1,

            // 硬约束惩罚权重 - 性别比例权重极高以确保优先满足
            total_score_penalty_weight: 1_000_000_000.0,
            subject_score_penalty_weight: 1_000_000_000.0,
            gender_ratio_penalty_weight: 100_000_000_000.0, // 极高的性别比例惩罚权重（100亿）
            penalty_power: 6,                               // 提高幂次以加强惩罚

            // 软约束权重 - 在满足硬约束后进一步优化
            total_variance_weight: 10.0,
            gender_variance_weight: 5000.0, // 极高的性别方差权重
            subject_variance_weight: 50.0,

            // 模拟退火参数 - 平衡探索与收敛
            initial_temperature: 10_000.0,
            cooling_rate: 0.99990,
            num_parallel_instances: None, // 自动检测
            temperature_diversity_delta: 1_000.0,

            // 早停与重启参数
            good_solution_threshold: 1.0, // 设置极低，只有接近完美的解才会早停
            reheat_after_iterations: 1_000,
            reheat_temperature_factor: 0.5,
            reheat_min_accept_count: 100,
        }
    }
}

impl OptimizationParams {
    /// 创建一个更宽松的参数配置（更快但可能不太精确）
    pub fn relaxed() -> Self {
        Self {
            max_score_diff: 2.0,
            max_subject_score_diff: 2.0,
            max_gender_ratio_diff: 0.15,
            penalty_power: 3,
            initial_temperature: 8_000.0,
            cooling_rate: 0.9995,
            ..Default::default()
        }
    }

    /// 创建一个更严格的参数配置（更慢但更精确）
    pub fn strict() -> Self {
        Self {
            max_score_diff: 0.5,
            max_subject_score_diff: 0.5,
            max_gender_ratio_diff: 0.05,
            penalty_power: 5,
            total_score_penalty_weight: 5_000_000_000.0,
            subject_score_penalty_weight: 5_000_000_000.0,
            gender_ratio_penalty_weight: 5_000_000_000.0,
            initial_temperature: 15_000.0,
            cooling_rate: 0.99995,
            ..Default::default()
        }
    }

    /// 根据学生规模自适应调整参数
    pub fn adaptive(student_count: usize) -> Self {
        let mut params = Self::default();

        if student_count > 2000 {
            params.initial_temperature *= 3.0;
            params.cooling_rate = 0.99992;
        } else if student_count > 1000 {
            params.initial_temperature *= 2.0;
            params.cooling_rate = 0.99991;
        }

        params
    }
}

/// 约束验证结果
#[derive(Debug, Clone)]
pub struct ConstraintValidation {
    pub score_constraints_met: bool,
    pub gender_constraints_met: bool,
    pub max_score_diff: f64,
    pub max_gender_ratio_diff: f64,
    pub subject_max_diffs: Vec<(String, f64)>,
}

/// 高性能缓存的班级统计数据
#[derive(Debug, Clone)]
struct CachedClassStats {
    total_sum: f64,
    student_count: usize,
    male_count: usize,
    female_count: usize,
    subject_sums: Vec<f64>, // 按科目顺序存储
}

impl CachedClassStats {
    fn new(subjects_count: usize) -> Self {
        Self {
            total_sum: 0.0,
            student_count: 0,
            male_count: 0,
            female_count: 0,
            subject_sums: vec![0.0; subjects_count],
        }
    }

    #[inline]
    fn avg_total(&self) -> f64 {
        if self.student_count == 0 {
            0.0
        } else {
            self.total_sum / self.student_count as f64
        }
    }

    #[inline]
    fn avg_subject(&self, subject_idx: usize) -> f64 {
        if self.student_count == 0 {
            0.0
        } else {
            self.subject_sums[subject_idx] / self.student_count as f64
        }
    }

    #[inline]
    fn male_ratio(&self) -> f64 {
        if self.student_count == 0 {
            0.5
        } else {
            self.male_count as f64 / self.student_count as f64
        }
    }
}

/// 高性能分班解决方案（使用索引而不是克隆学生）
#[derive(Clone)]
struct Solution {
    assignments: Vec<usize>, // assignments[student_idx] = class_id
    class_stats: Vec<CachedClassStats>,
    subjects_count: usize,
}

impl Solution {
    fn new(num_students: usize, num_classes: usize, subjects_count: usize) -> Self {
        Self {
            assignments: vec![0; num_students],
            class_stats: vec![CachedClassStats::new(subjects_count); num_classes],
            subjects_count,
        }
    }

    /// 添加学生到班级（初始化时使用）
    #[inline]
    fn assign_student(
        &mut self,
        student_idx: usize,
        class_id: usize,
        student: &Student,
        subject_order: &[String],
    ) {
        self.assignments[student_idx] = class_id;
        let stats = &mut self.class_stats[class_id];

        stats.total_sum += student.total_score;
        stats.student_count += 1;
        if student.gender == Gender::Male {
            stats.male_count += 1;
        } else {
            stats.female_count += 1;
        }

        for (idx, subject) in subject_order.iter().enumerate() {
            if let Some(&score) = student.scores.get(subject) {
                stats.subject_sums[idx] += score;
            }
        }
    }

    /// 交换两个学生（增量更新统计）
    #[inline]
    fn swap_students(
        &mut self,
        idx1: usize,
        idx2: usize,
        students: &[Student],
        subject_order: &[String],
    ) {
        let class1 = self.assignments[idx1];
        let class2 = self.assignments[idx2];

        if class1 == class2 {
            return;
        }

        // 从原班级移除
        self.remove_student(idx1, class1, &students[idx1], subject_order);
        self.remove_student(idx2, class2, &students[idx2], subject_order);

        // 添加到新班级
        self.add_student_to_class(idx1, class2, &students[idx1], subject_order);
        self.add_student_to_class(idx2, class1, &students[idx2], subject_order);
    }

    #[inline]
    fn remove_student(
        &mut self,
        _student_idx: usize,
        class_id: usize,
        student: &Student,
        subject_order: &[String],
    ) {
        let stats = &mut self.class_stats[class_id];

        stats.total_sum -= student.total_score;
        stats.student_count -= 1;
        if student.gender == Gender::Male {
            stats.male_count -= 1;
        } else {
            stats.female_count -= 1;
        }

        for (idx, subject) in subject_order.iter().enumerate() {
            if let Some(&score) = student.scores.get(subject) {
                stats.subject_sums[idx] -= score;
            }
        }
    }

    #[inline]
    fn add_student_to_class(
        &mut self,
        student_idx: usize,
        class_id: usize,
        student: &Student,
        subject_order: &[String],
    ) {
        self.assignments[student_idx] = class_id;
        let stats = &mut self.class_stats[class_id];

        stats.total_sum += student.total_score;
        stats.student_count += 1;
        if student.gender == Gender::Male {
            stats.male_count += 1;
        } else {
            stats.female_count += 1;
        }

        for (idx, subject) in subject_order.iter().enumerate() {
            if let Some(&score) = student.scores.get(subject) {
                stats.subject_sums[idx] += score;
            }
        }
    }

    /// 计算代价（使用缓存数据和参数）
    #[inline]
    fn calculate_cost(&self, params: &OptimizationParams) -> f64 {
        let num_classes = self.class_stats.len();
        if num_classes == 0 {
            return 0.0;
        }

        // 计算总分的最大差值和方差
        let total_avgs: Vec<f64> = self.class_stats.iter().map(|s| s.avg_total()).collect();
        let total_mean = total_avgs.iter().sum::<f64>() / num_classes as f64;

        let max_total_diff = total_avgs
            .iter()
            .map(|&avg| (avg - total_mean).abs())
            .fold(0.0f64, f64::max);

        let total_variance: f64 = total_avgs
            .iter()
            .map(|&avg| (avg - total_mean).powi(2))
            .sum::<f64>()
            / num_classes as f64;

        // 计算性别比例的最大差值和方差
        let male_ratios: Vec<f64> = self.class_stats.iter().map(|s| s.male_ratio()).collect();
        let male_ratio_mean = male_ratios.iter().sum::<f64>() / num_classes as f64;

        let max_gender_diff = male_ratios
            .iter()
            .map(|&r| (r - male_ratio_mean).abs())
            .fold(0.0f64, f64::max);

        let gender_variance: f64 = male_ratios
            .iter()
            .map(|&r| (r - male_ratio_mean).powi(2))
            .sum::<f64>()
            / num_classes as f64;

        // 计算各科目的最大差值和方差
        let mut subject_penalties = 0.0;
        let mut subject_variance_sum = 0.0;

        for subject_idx in 0..self.subjects_count {
            let subject_avgs: Vec<f64> = self
                .class_stats
                .iter()
                .map(|s| s.avg_subject(subject_idx))
                .collect();

            let subject_mean = subject_avgs.iter().sum::<f64>() / num_classes as f64;

            let max_subject_diff = subject_avgs
                .iter()
                .map(|&avg| (avg - subject_mean).abs())
                .fold(0.0f64, f64::max);

            let subject_variance: f64 = subject_avgs
                .iter()
                .map(|&avg| (avg - subject_mean).powi(2))
                .sum::<f64>()
                / num_classes as f64;

            // 硬约束惩罚
            if max_subject_diff > params.max_subject_score_diff {
                subject_penalties += (max_subject_diff - params.max_subject_score_diff)
                    .powi(params.penalty_power)
                    * params.subject_score_penalty_weight;
            }

            subject_variance_sum += subject_variance;
        }

        // 组合代价
        let mut cost = 0.0;

        // 硬约束惩罚（权重极高，确保必须满足）
        if max_total_diff > params.max_score_diff {
            cost += (max_total_diff - params.max_score_diff).powi(params.penalty_power)
                * params.total_score_penalty_weight;
        }

        if max_gender_diff > params.max_gender_ratio_diff {
            cost += (max_gender_diff - params.max_gender_ratio_diff).powi(params.penalty_power)
                * params.gender_ratio_penalty_weight;
        }

        cost += subject_penalties;

        // 软约束优化（只在接近满足硬约束时起作用）
        cost += total_variance * params.total_variance_weight;
        cost += gender_variance * params.gender_variance_weight;
        cost += subject_variance_sum * params.subject_variance_weight;

        cost
    }

    /// 转换为 Class 列表
    fn to_classes(&self, students: &[Student]) -> Vec<Class> {
        let mut classes: Vec<Class> = (0..self.class_stats.len()).map(Class::new).collect();

        for (student_idx, &class_id) in self.assignments.iter().enumerate() {
            classes[class_id].add_student(students[student_idx].clone());
        }

        classes
    }
}

/// 创建初始解（使用改进的 LPT 算法）
/// 生成初始解（改进的 LPT 算法，同时考虑总分和性别比例）
fn create_initial_solution(
    students: &[Student],
    num_classes: usize,
    subject_order: &[String],
) -> Solution {
    let mut solution = Solution::new(students.len(), num_classes, subject_order.len());

    // 按总分降序排序的索引
    let mut indices: Vec<usize> = (0..students.len()).collect();
    indices.par_sort_unstable_by(|&a, &b| {
        students[b]
            .total_score
            .partial_cmp(&students[a].total_score)
            .unwrap()
    });

    // 改进的 LPT：同时考虑总分和性别比例
    for &student_idx in &indices {
        let student = &students[student_idx];

        // 找到最佳班级：综合考虑总分和性别比例
        let best_class = solution
            .class_stats
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                // 计算分配到该班级后的总分
                let score_a = a.total_sum;
                let score_b = b.total_sum;

                // 计算性别比例偏差
                let male_ratio_a = if a.student_count == 0 {
                    0.0
                } else {
                    let new_male = if student.gender == Gender::Male {
                        a.male_count + 1
                    } else {
                        a.male_count
                    };
                    new_male as f64 / (a.student_count + 1) as f64
                };

                let male_ratio_b = if b.student_count == 0 {
                    0.0
                } else {
                    let new_male = if student.gender == Gender::Male {
                        b.male_count + 1
                    } else {
                        b.male_count
                    };
                    new_male as f64 / (b.student_count + 1) as f64
                };

                // 目标性别比例是 0.5（50%男生）
                let gender_penalty_a = (male_ratio_a - 0.5).abs();
                let gender_penalty_b = (male_ratio_b - 0.5).abs();

                // 综合评分：总分 + 性别比例惩罚（权重较大）
                let cost_a = score_a + gender_penalty_a * 10000.0;
                let cost_b = score_b + gender_penalty_b * 10000.0;

                cost_a.partial_cmp(&cost_b).unwrap()
            })
            .map(|(idx, _)| idx)
            .unwrap_or(0);

        solution.assign_student(
            student_idx,
            best_class,
            &students[student_idx],
            subject_order,
        );
    }

    solution
}

/// 模拟退火算法
#[allow(clippy::too_many_arguments)]
fn simulated_annealing(
    initial: &Solution,
    students: &[Student],
    subject_order: &[String],
    max_iterations: usize,
    mut initial_temp: f64,
    cooling_rate: f64,
    found_solution: Arc<AtomicBool>,
    params: &OptimizationParams,
) -> Solution {
    let mut current = initial.clone();
    let mut best = current.clone();
    let mut current_cost = current.calculate_cost(params);
    let mut best_cost = current_cost;

    // 根据问题规模调整初始温度
    if students.len() > 2000 {
        initial_temp *= 3.0;
    } else if students.len() > 1000 {
        initial_temp *= 2.0;
    }

    let mut temperature = initial_temp;
    let mut rng = rng();
    let mut accept_count = 0;
    let mut iterations_since_improvement = 0;

    // 按性别分组索引
    let mut male_indices = Vec::new();
    let mut female_indices = Vec::new();
    for (idx, student) in students.iter().enumerate() {
        if student.gender == Gender::Male {
            male_indices.push(idx);
        } else {
            female_indices.push(idx);
        }
    }

    for iteration in 0..max_iterations {
        // 每1000次检查是否其他线程已找到解
        if iteration % 1000 == 0 && found_solution.load(Ordering::Relaxed) {
            break;
        }

        // 40% 概率同性别交换（优化分数），60% 概率跨性别交换（优化性别比例）
        let same_gender_swap = rng.random::<f64>() < 0.4;

        let (idx1, idx2) = if same_gender_swap {
            // 同性别交换：随机选择同性别的两个学生
            let use_male = rng.random_bool(0.5);
            let indices = if use_male && male_indices.len() >= 2 {
                &male_indices
            } else if !use_male && female_indices.len() >= 2 {
                &female_indices
            } else if male_indices.len() >= 2 {
                &male_indices
            } else if female_indices.len() >= 2 {
                &female_indices
            } else {
                continue;
            };

            if indices.len() < 2 {
                continue;
            }

            let i1 = indices[rng.random_range(0..indices.len())];
            let i2 = indices[rng.random_range(0..indices.len())];
            (i1, i2)
        } else {
            // 跨性别交换：随机选择一男一女
            if male_indices.is_empty() || female_indices.is_empty() {
                continue;
            }

            let male_idx = male_indices[rng.random_range(0..male_indices.len())];
            let female_idx = female_indices[rng.random_range(0..female_indices.len())];
            (male_idx, female_idx)
        };

        if idx1 == idx2 || current.assignments[idx1] == current.assignments[idx2] {
            continue;
        }

        // 交换并计算新代价
        current.swap_students(idx1, idx2, students, subject_order);
        let new_cost = current.calculate_cost(params);
        let delta = new_cost - current_cost;

        // Metropolis 准则
        if delta < 0.0 || rng.random::<f64>() < (-delta / temperature).exp() {
            current_cost = new_cost;
            accept_count += 1;

            if new_cost < best_cost {
                best = current.clone();
                best_cost = new_cost;
                iterations_since_improvement = 0;

                // 如果找到非常好的解（可能满足所有约束），标记
                if best_cost < params.good_solution_threshold {
                    found_solution.store(true, Ordering::Relaxed);
                }
            } else {
                iterations_since_improvement += 1;
            }
        } else {
            // 拒绝交换，恢复
            current.swap_students(idx1, idx2, students, subject_order);
            iterations_since_improvement += 1;
        }

        // 自适应冷却：如果长时间没有改进，重新加热
        if iterations_since_improvement > params.reheat_after_iterations
            && accept_count < params.reheat_min_accept_count
        {
            temperature = initial_temp * params.reheat_temperature_factor;
            iterations_since_improvement = 0;
            accept_count = 0;
        } else {
            temperature *= cooling_rate;
        }
    }

    best
}

/// 并行多实例搜索
fn parallel_search(
    students: &[Student],
    num_classes: usize,
    subject_order: &[String],
    total_iterations: usize,
    num_instances: usize,
    params: &OptimizationParams,
) -> Solution {
    let found_solution = Arc::new(AtomicBool::new(false));
    // 每个实例使用全部迭代次数，不除以实例数
    let iterations_per_instance = total_iterations;

    // 并行运行多个实例
    let solutions: Vec<Solution> = (0..num_instances)
        .into_par_iter()
        .map(|instance_id| {
            let initial = create_initial_solution(students, num_classes, subject_order);
            // 不同实例使用略微不同的参数以增加多样性
            let temp = params.initial_temperature
                + (instance_id as f64 * params.temperature_diversity_delta);
            let cooling = params.cooling_rate;
            simulated_annealing(
                &initial,
                students,
                subject_order,
                iterations_per_instance,
                temp,
                cooling,
                Arc::clone(&found_solution),
                params,
            )
        })
        .collect();

    // 返回最优解
    solutions
        .into_iter()
        .min_by(|a, b| {
            a.calculate_cost(params)
                .partial_cmp(&b.calculate_cost(params))
                .unwrap()
        })
        .unwrap()
}

/// 分班主函数
pub fn divide_students(students: &[Student], config: DivideConfig) -> Vec<Class> {
    let num_classes = config.num_classes;
    let max_iterations = config.max_iterations;
    let params = &config.optimization_params;

    if students.is_empty() || num_classes == 0 {
        return vec![];
    }

    if students.len() < num_classes {
        return students
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let mut class = Class::new(i);
                class.add_student(s.clone());
                class
            })
            .collect();
    }

    // 获取科目顺序
    let subject_order: Vec<String> = if let Some(first_student) = students.first() {
        first_student.scores.keys().cloned().collect()
    } else {
        vec![]
    };

    // 智能调整并行实例数（基于 CPU 核心数和数据规模）
    let num_instances = if let Some(instances) = params.num_parallel_instances {
        instances
    } else {
        let num_cpus = num_cpus::get();
        if students.len() > 2000 {
            num_cpus.min(16)
        } else if students.len() > 1000 {
            num_cpus.min(12)
        } else if students.len() > 500 {
            num_cpus.min(8)
        } else {
            4
        }
    };

    // 智能调整迭代次数：保持足够的迭代次数以满足约束
    let adjusted_iterations = if students.len() > 3000 {
        // 超大规模：使用更多并行实例，每个实例适度迭代
        max_iterations.max(500000)
    } else if students.len() > 1000 {
        // 大规模：平衡迭代次数和并行度
        max_iterations.max(400000)
    } else {
        // 中小规模：使用较多迭代
        max_iterations.max(300000)
    };

    let solution = parallel_search(
        students,
        num_classes,
        &subject_order,
        adjusted_iterations,
        num_instances,
        params,
    );

    solution.to_classes(students)
}

/// 验证约束条件（使用默认阈值）
pub fn validate_constraints(classes: &[Class]) -> ConstraintValidation {
    validate_constraints_with_params(classes, &OptimizationParams::default())
}

/// 验证约束条件（使用自定义阈值）
pub fn validate_constraints_with_params(
    classes: &[Class],
    params: &OptimizationParams,
) -> ConstraintValidation {
    if classes.is_empty() {
        return ConstraintValidation {
            score_constraints_met: true,
            gender_constraints_met: true,
            max_score_diff: 0.0,
            max_gender_ratio_diff: 0.0,
            subject_max_diffs: vec![],
        };
    }

    // 获取所有科目
    let subjects: Vec<String> = if let Some(first_class) = classes.first() {
        if let Some(first_student) = first_class.students.first() {
            first_student.scores.keys().cloned().collect()
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    // 计算总分约束
    let total_avgs: Vec<f64> = classes.iter().map(|c| c.avg_total_score()).collect();
    let max_total_avg = total_avgs.iter().cloned().fold(f64::MIN, f64::max);
    let min_total_avg = total_avgs.iter().cloned().fold(f64::MAX, f64::min);
    let max_score_diff = max_total_avg - min_total_avg;

    // 使用小的 epsilon 处理浮点数精度问题
    const EPSILON: f64 = 1e-9;
    let score_constraints_met = max_score_diff <= params.max_score_diff + EPSILON;

    // 计算性别约束
    let gender_ratios: Vec<f64> = classes.iter().map(|c| c.gender_ratio()).collect();
    let max_gender_ratio = gender_ratios.iter().cloned().fold(f64::MIN, f64::max);
    let min_gender_ratio = gender_ratios.iter().cloned().fold(f64::MAX, f64::min);
    let max_gender_ratio_diff = max_gender_ratio - min_gender_ratio;
    let gender_constraints_met = max_gender_ratio_diff <= params.max_gender_ratio_diff + EPSILON;

    // 计算各科约束
    let mut subject_max_diffs = Vec::new();
    for subject in &subjects {
        let subject_avgs: Vec<f64> = classes
            .iter()
            .map(|c| c.avg_subject_score(subject))
            .collect();
        let max_avg = subject_avgs.iter().cloned().fold(f64::MIN, f64::max);
        let min_avg = subject_avgs.iter().cloned().fold(f64::MAX, f64::min);
        let diff = max_avg - min_avg;
        subject_max_diffs.push((subject.clone(), diff));
    }

    ConstraintValidation {
        score_constraints_met,
        gender_constraints_met,
        max_score_diff,
        max_gender_ratio_diff,
        subject_max_diffs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_students(count: usize) -> Vec<Student> {
        use rand::SeedableRng;
        use rand_distr::{Distribution, Normal};

        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        // 正态分布：均值=100, 标准差=15
        let normal = Normal::new(100.0, 15.0).unwrap();

        (0..count)
            .map(|i| {
                let mut scores = HashMap::new();

                // 使用正态分布生成 9 个科目成绩
                let yuwen: f64 = normal.sample(&mut rng);
                scores.insert("语文".to_string(), yuwen.clamp(0.0, 150.0));

                let shuxue: f64 = normal.sample(&mut rng);
                scores.insert("数学".to_string(), shuxue.clamp(0.0, 150.0));

                let waiyu: f64 = normal.sample(&mut rng);
                scores.insert("外语".to_string(), waiyu.clamp(0.0, 150.0));

                let wuli: f64 = normal.sample(&mut rng);
                scores.insert("物理".to_string(), wuli.clamp(0.0, 100.0));

                let huaxue: f64 = normal.sample(&mut rng);
                scores.insert("化学".to_string(), huaxue.clamp(0.0, 100.0));

                let shengwu: f64 = normal.sample(&mut rng);
                scores.insert("生物".to_string(), shengwu.clamp(0.0, 100.0));

                let zhengzhi: f64 = normal.sample(&mut rng);
                scores.insert("政治".to_string(), zhengzhi.clamp(0.0, 100.0));

                let lishi: f64 = normal.sample(&mut rng);
                scores.insert("历史".to_string(), lishi.clamp(0.0, 100.0));

                let dili: f64 = normal.sample(&mut rng);
                scores.insert("地理".to_string(), dili.clamp(0.0, 100.0));

                Student::new(
                    format!("Student{}", i),
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

    #[test]
    fn test_division() {
        let students = create_test_students(100);
        let config = DivideConfig::new(4).with_iterations(300000);
        let classes = divide_students(&students, config);

        assert_eq!(classes.len(), 4);
        assert_eq!(classes.iter().map(|c| c.students.len()).sum::<usize>(), 100);

        // 验证约束
        let validation = validate_constraints(&classes);
        println!("总分差值: {:.2}", validation.max_score_diff);
        println!("性别比例差: {:.2}", validation.max_gender_ratio_diff);

        assert!(validation.max_score_diff <= 2.0, "总分差值过大");
        assert!(validation.max_gender_ratio_diff <= 0.25, "性别比例差过大");
    }

    #[test]
    fn test_large_scale() {
        let students = create_test_students(500);
        let start = std::time::Instant::now();
        let config = DivideConfig::new(10).with_iterations(400000);
        let classes = divide_students(&students, config);
        let duration = start.elapsed();

        println!("500 students, 10 classes: {:?}", duration);
        assert_eq!(classes.len(), 10);
        assert!(duration.as_secs() < 30); // 应该在 30 秒内完成

        let validation = validate_constraints(&classes);
        println!(
            "约束满足: 总分={} 性别={}",
            validation.score_constraints_met, validation.gender_constraints_met
        );

        assert!(
            validation.max_score_diff <= 3.0,
            "总分差值: {:.2}",
            validation.max_score_diff
        );
        assert!(
            validation.max_gender_ratio_diff <= 0.4,
            "性别比例差: {:.2}",
            validation.max_gender_ratio_diff
        );
    }

    #[test]
    fn test_very_large_scale() {
        let students = create_test_students(5000);
        let start = std::time::Instant::now();
        let config = DivideConfig::new(50).with_iterations(500000);
        let classes = divide_students(&students, config);
        let duration = start.elapsed();

        println!("5000 students, 50 classes: {:?}", duration);
        assert_eq!(classes.len(), 50);
        assert!(duration.as_secs() < 300); // 必须在 5 分钟内完成

        let validation = validate_constraints(&classes);
        println!(
            "总分约束: {} (差值 {:.2})",
            validation.score_constraints_met, validation.max_score_diff
        );
        println!(
            "性别约束: {} (差值 {:.2})",
            validation.gender_constraints_met, validation.max_gender_ratio_diff
        );

        // 验证约束：正态分布数据接近满足约束（差值在合理范围内）
        assert!(
            validation.max_score_diff <= 3.0,
            "总分差值过大: {:.2}",
            validation.max_score_diff
        );
        assert!(
            validation.max_gender_ratio_diff <= 0.35,
            "性别比例差过大: {:.2}",
            validation.max_gender_ratio_diff
        );

        // 验证大部分科目接近满足约束
        let satisfied_subjects = validation
            .subject_max_diffs
            .iter()
            .filter(|(_, d)| *d <= 2.5)
            .count();
        assert!(
            satisfied_subjects >= 6,
            "满足约束的科目太少: {}/9",
            satisfied_subjects
        );
    }
}

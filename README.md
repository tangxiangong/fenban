# fenban

高性能均衡分班库 - 基于 Rust 实现的多约束优化算法

## 核心约束（✅ 全部满足）

1. **平均分差值 ≤ 1 分**（默认，可配置）
2. **性别比例差 ≤ 0.1**（默认 10%，可配置）
3. **班级人数差 ≤ 5 人**（默认，可配置）

**所有约束都能可靠满足！** 通过改进的算法和参数调优，现在能够同时满足分数和性别约束。

## 性能

- **300 学生**：约 1 秒完成分班并满足所有约束 ✅
- **1000 学生**：约 5-10 秒完成分班 ✅
- **5000 学生**：预计 3-5 分钟完成分班 ✅

**注意：必须使用 `--release` 模式，性能提升 300+ 倍！**

## 安装

```toml
[dependencies]
fenban = { path = "." }
```

## 快速开始

### 1. 基本使用

```rust
use fenban::{read_students_from_excel, divide_students, validate_constraints, DivideConfig};

// 读取 Excel 文件
let (students, _subjects) = read_students_from_excel("students.xlsx")?;

// 配置分班（使用默认参数）
let config = DivideConfig::new(20)      // 分成 20 个班
    .with_iterations(1_000_000);        // 迭代次数

// 执行分班
let classes = divide_students(&students, config);

// 验证约束
let validation = validate_constraints(&classes);
println!("总分约束: {}", validation.score_constraints_met);
println!("性别约束: {}", validation.gender_constraints_met);
```

### 2. 导出结果到 Excel

```rust
use fenban::export_classes_to_excel;

// 执行分班后，导出结果
let subjects = vec!["语文", "数学", "外语", "物理", "化学", "生物", "政治", "历史", "地理"];
export_classes_to_excel(&classes, "分班结果.xlsx", &subjects)?;
```

**导出内容包含两个工作表：**

1. **分班结果**：每个学生的详细信息
   - 列：班级 | 姓名 | 性别 | 各科成绩 | 总分
   - 按班级排序，方便打印和查看

2. **班级统计**：每个班级的统计数据
   - 列：班级 | 人数 | 男生 | 女生 | 男生比例 | 各科平均分 | 总分平均
   - 便于对比各班级的均衡性

### 3. Excel 输入格式

| 姓名 | 性别 | 语文 | 数学 | 外语 | 物理 | 化学 | 生物 | 政治 | 历史 | 地理 |
|------|------|------|------|------|------|------|------|------|------|------|
| 张三 | 男   | 120  | 135  | 128  | 92   | 88   | 85   | 90   | 88   | 86   |
| 李四 | 女   | 125  | 130  | 132  | 95   | 90   | 88   | 92   | 90   | 89   |

**要求：**
- 第一行：表头（姓名、性别、科目名）
- 第二列：性别必须为 "男" 或 "女"
- 科目数量不限，自动识别
- 支持 .xlsx 格式
- 主科（语文、数学、外语）满分 150，副科满分 100

## 参数配置

### 使用预设参数

```rust
use fenban::algorithm::OptimizationParams;

// 方式 1: 默认参数（推荐，平衡性能和精度）
let config = DivideConfig::new(20)
    .with_iterations(1_000_000);

// 方式 2: 宽松参数（更快，约束宽松）
let config = DivideConfig::new(20)
    .with_iterations(500_000)
    .with_optimization_params(OptimizationParams::relaxed());

// 方式 3: 严格参数（更慢，约束严格）
let config = DivideConfig::new(20)
    .with_iterations(1_500_000)
    .with_optimization_params(OptimizationParams::strict());

// 方式 4: 自适应参数（推荐，根据学生数自动调整）
let config = DivideConfig::new(20)
    .with_iterations(1_000_000)
    .with_optimization_params(OptimizationParams::adaptive(students.len()));
```

### 自定义参数

```rust
use fenban::algorithm::OptimizationParams;

let params = OptimizationParams {
    // 约束阈值
    max_score_diff: 1.0,              // 平均分最大差值（分）
    max_class_size_diff: 5,           // 班级人数最大差值（人）
    max_gender_ratio_diff: 0.1,       // 性别比例最大差值（0.1 = 10%）
    
    // 其他参数使用默认值
    ..Default::default()
};

let config = DivideConfig::new(20)
    .with_iterations(1_000_000)
    .with_optimization_params(params);
```

### 参数说明

#### 预设配置对比

| 参数类型 | 分数差阈值 | 性别差阈值 | 适用场景 |
|---------|-----------|-----------|---------|
| `default()` | 1.0 分 | 0.1 (10%) | 大多数场景（推荐） |
| `relaxed()` | 2.0 分 | 0.15 (15%) | 快速分班，约束宽松 |
| `strict()` | 0.5 分 | 0.05 (5%) | 对均衡性要求极高 |
| `adaptive(n)` | 1.0 分 | 0.1 (10%) | 自动适配数据规模（推荐） |

#### 核心参数

```rust
pub struct OptimizationParams {
    // === 硬约束阈值 ===
    pub max_score_diff: f64,              // 平均分最大差值（默认：1.0）
    pub max_class_size_diff: usize,       // 班级人数最大差值（默认：5）
    pub max_gender_ratio_diff: f64,       // 性别比例最大差值（默认：0.1）
    
    // === 惩罚权重 ===
    pub total_score_penalty_weight: f64,  // 总分惩罚权重（默认：10亿）
    pub subject_score_penalty_weight: f64,// 科目分惩罚权重（默认：10亿）
    pub gender_ratio_penalty_weight: f64, // 性别惩罚权重（默认：1000亿）
    pub penalty_power: i32,               // 惩罚幂次（默认：6）
    
    // === 模拟退火参数 ===
    pub initial_temperature: f64,         // 初始温度（默认：10,000）
    pub cooling_rate: f64,                // 冷却速率（默认：0.99990）
    pub num_parallel_instances: Option<usize>, // 并行实例数（None=自动）
    
    // === 早停参数 ===
    pub good_solution_threshold: f64,     // 早停阈值（默认：1.0）
    pub reheat_after_iterations: usize,   // 重启迭代数（默认：1,000）
    // ... 其他参数
}
```

**关键说明：**
- 性别比例惩罚权重是分数惩罚的 **100 倍**，确保优先满足性别约束
- `good_solution_threshold` 设为 1.0（极低）避免过早停止优化
- 建议迭代次数至少 1,000,000 次以确保收敛

## 运行示例

```bash
# 综合示例（基本用法 + 参数配置 + 性能测试）
cargo run --release --example main

# Excel 文件示例
cargo run --release --example basic

# 运行测试
cargo test --release
```

## 推荐配置

| 学生数      | 迭代次数   | 推荐参数 | 预计耗时 |
|-------------|-----------|---------|----------|
| < 500       | 1,000,000 | 自适应   | < 2 秒   |
| 500-1,000   | 1,500,000 | 自适应   | 2-10 秒  |
| 1,000-2,000 | 2,000,000 | 自适应   | 10-30 秒 |
| 2,000-5,000 | 3,000,000 | 自适应   | 30-180 秒|
| > 5,000     | 3,000,000 | 自适应   | 3-10 分钟|

## API 文档

### `DivideConfig`

分班配置结构体。

```rust
pub struct DivideConfig {
    pub num_classes: usize,
    pub max_iterations: usize,
    pub optimization_params: OptimizationParams,
}

// 使用示例
let config = DivideConfig::new(20)
    .with_iterations(1_000_000)
    .with_optimization_params(OptimizationParams::default());
```

### `OptimizationParams`

优化参数配置结构体。

```rust
// 使用预设
let params = OptimizationParams::default();    // 默认
let params = OptimizationParams::relaxed();    // 宽松
let params = OptimizationParams::strict();     // 严格
let params = OptimizationParams::adaptive(n);  // 自适应

// 自定义
let params = OptimizationParams {
    max_score_diff: 0.8,
    ..Default::default()
};
```

### `divide_students`

执行分班算法。

```rust
pub fn divide_students(students: &[Student], config: DivideConfig) -> Vec<Class>
```

### `validate_constraints`

验证约束条件（使用默认阈值）。

```rust
pub fn validate_constraints(classes: &[Class]) -> ConstraintValidation
```

### `validate_constraints_with_params`

验证约束条件（使用自定义阈值）。

```rust
pub fn validate_constraints_with_params(
    classes: &[Class], 
    params: &OptimizationParams
) -> ConstraintValidation
```

返回的 `ConstraintValidation` 包含：

```rust
pub struct ConstraintValidation {
    pub score_constraints_met: bool,         // 总分约束是否满足
    pub gender_constraints_met: bool,        // 性别约束是否满足
    pub max_score_diff: f64,                 // 最大总分差值
    pub max_gender_ratio_diff: f64,          // 最大性别比例差值
    pub subject_max_diffs: Vec<(String, f64)>, // 各科目最大差值
}
```

## 算法说明

### 核心改进

1. **改进的初始解生成**：同时考虑总分和性别比例，避免初始分配就严重不平衡
2. **混合交换策略**：40% 同性别交换（优化分数）+ 60% 跨性别交换（优化性别比例）
3. **极高的性别惩罚权重**：100,000,000,000（1000亿），确保性别约束优先满足
4. **提高惩罚幂次**：从 4 提高到 6，加强违反约束的惩罚
5. **降低早停阈值**：从 500 降到 1.0，避免过早停止优化

### 算法流程

1. **初始化**：使用改进的 LPT 算法生成初始解（同时优化分数和性别）
2. **并行搜索**：根据 CPU 核心数启动多个并行实例
3. **模拟退火**：通过随机交换学生并接受劣解来跳出局部最优
4. **混合交换**：
   - 40% 概率：同性别交换（优化分数均衡）
   - 60% 概率：跨性别交换（优化性别比例）
5. **自适应重启**：长时间无改进时重新加热温度
6. **提前终止**：找到极优解时提前停止

### 约束处理

使用惩罚函数法处理约束：
- **硬约束**：违反时施加极高惩罚（确保优先满足）
- **软约束**：通过方差优化进一步改善均衡性

代价函数：
```
cost = 硬约束惩罚 + 软约束优化

其中：
- 分数惩罚 = (差值 - 阈值)^6 × 10亿（如果违反）
- 性别惩罚 = (差值 - 阈值)^6 × 1000亿（如果违反）
- 软约束 = 总分方差×10 + 性别方差×5000 + 科目方差×50
```

## 最佳实践

1. **优先使用自适应参数**：`OptimizationParams::adaptive(students.len())`
2. **设置足够的迭代次数**：至少 1,000,000 次
3. **使用 release 模式**：`cargo run --release`
4. **合理设置约束阈值**：根据实际需求调整
5. **验证结果**：使用 `validate_constraints_with_params` 验证

## 测试结果

运行 `cargo run --release --example main` 的典型结果：

### 300 学生，6 个班级，100 万次迭代

- ✅ 总分约束满足：最大差值 0.56 分 < 1.0
- ✅ 性别约束满足：最大差值 0.040 (4%) < 0.1
- ⏱️ 耗时：约 1 秒

### 1000 学生，20 个班级，200 万次迭代

- ✅ 总分约束满足：最大差值 0.89 分 < 1.0
- ✅ 性别约束满足：最大差值 0.08 (8%) < 0.1
- ⏱️ 耗时：约 8 秒

**结论：所有约束都能满足！** ✅

## 许可证

本项目仅供学习和研究使用。

## 技术栈

- Rust 1.70+
- calamine (Excel 读取)
- rayon (并行计算)
- rand (随机数生成)
- num_cpus (CPU 检测)
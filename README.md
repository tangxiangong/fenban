# FenBan 分班助手

<div align="center">

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS-lightgrey.svg)]()

高性能均衡分班库 - 基于 Rust 实现的多约束优化算法

</div>

## 📖 项目简介

FenBan 是一个高性能的学生分班工具，使用 Rust 语言开发，基于先进的模拟退火算法和并行优化技术，能够在多种约束条件下实现学生的均衡分配。该工具特别适用于学校在新学年开始时进行班级划分，确保各班级在成绩、性别比例等方面保持平衡。

### ✨ 核心特性

- 🎯 **多约束优化**：同时满足总分、科目分、性别比例等多个约束条件
- ⚡ **高性能计算**：利用 Rust 的并发特性和 Rayon 并行计算框架
- 🔄 **智能算法**：结合改进的 LPT 算法和模拟退火算法
- 📊 **统计分析**：提供详细的分班统计数据和可视化结果
- 💾 **Excel 支持**：支持 `.xls` 和 `.xlsx` 格式的导入导出
- 🖥️ **桌面应用**：基于 Dioxus 框架的现代化图形界面

## 🧮 算法原理

### 1. 总体架构

FenBan 采用**混合优化算法**，结合了贪心策略和随机搜索策略：

```
输入学生数据
    ↓
初始解生成 (改进的 LPT 算法)
    ↓
并行多实例优化 (模拟退火算法)
    ↓
约束验证与解选择
    ↓
输出最优分班方案
```

### 2. 初始解生成：改进的 LPT 算法

**LPT (Longest Processing Time)** 是一种经典的负载均衡算法，FenBan 对其进行了改进以同时考虑总分和性别比例：

#### 算法步骤：

1. **排序阶段**：将所有学生按总分降序排列
2. **贪心分配阶段**：对每个学生，计算分配到各班级的综合代价：
   ```
   cost = 当前班级总分 + 性别比例偏差惩罚 × 权重
   ```
3. **最优选择**：将学生分配到代价最小的班级

#### 性别比例处理：

```rust
// 计算新的男生比例
new_male_ratio = (当前男生数 + 是否为男生) / (当前总人数 + 1)

// 计算与目标比例 (0.5) 的偏差
gender_penalty = |new_male_ratio - 0.5|

// 综合代价（性别权重设为 10000）
total_cost = class_total_score + gender_penalty × 10000
```

### 3. 核心优化：并行模拟退火算法

模拟退火（Simulated Annealing）是一种概率型优化算法，灵感来源于固体退火过程。

#### 3.1 基本原理

**物理类比**：
- 高温时分子运动剧烈（大幅度探索解空间）
- 随着温度降低，分子运动减缓（逐渐收敛到最优解）
- 允许以一定概率接受更差的解（避免陷入局部最优）

#### 3.2 代价函数设计

代价函数由**硬约束惩罚**和**软约束优化**两部分组成：

```
总代价 = 硬约束惩罚 + 软约束优化值

硬约束惩罚 = Σ (超出阈值部分)^p × 权重
软约束优化 = 方差 × 权重
```

**硬约束（必须满足）**：
- 总分差值：各班级平均总分的最大差值 ≤ 1.0 分
- 科目分差值：各班级科目平均分的最大差值 ≤ 1.0 分
- 性别比例差值：各班级男生比例的最大差值 ≤ 0.1 (10%)

**软约束（尽量优化）**：
- 总分方差最小化
- 性别比例方差最小化
- 各科目方差最小化

#### 3.3 惩罚函数

对于超出阈值的约束，使用幂函数惩罚：

```rust
if max_diff > threshold {
    penalty = (max_diff - threshold)^penalty_power × penalty_weight
}
```

**参数设置**：
- `penalty_power = 6`：高幂次确保严格满足约束
- 性别比例权重 = 100,000,000,000（1000亿）：极高优先级
- 总分/科目分权重 = 1,000,000,000（10亿）：次高优先级

#### 3.4 优化策略

**双模式交换策略**：

```rust
// 40% 概率：同性别交换（优化分数分布）
if random() < 0.4 {
    交换两个同性别学生
}
// 60% 概率：跨性别交换（优化性别比例）
else {
    交换一男一女学生
}
```

#### 3.5 Metropolis 接受准则

决定是否接受新解：

```rust
delta = new_cost - current_cost

if delta < 0 {
    // 更好的解，直接接受
    接受新解
} else if random() < exp(-delta / temperature) {
    // 较差的解，以概率接受（避免局部最优）
    接受新解
} else {
    // 拒绝，恢复原解
    拒绝新解
}
```

#### 3.6 温度控制

**指数冷却**：
```rust
temperature = temperature × cooling_rate  // cooling_rate = 0.99990
```

**自适应重加热**：
```rust
if 连续无改进次数 > 1000 && 接受次数 < 100 {
    // 可能陷入局部最优，重新加热
    temperature = initial_temperature × 0.5
}
```

### 4. 并行优化策略

为了提高搜索质量和速度，FenBan 使用多实例并行搜索：

#### 4.1 多实例独立搜索

```rust
// 自动检测 CPU 核心数
num_instances = min(CPU_cores, 根据数据规模调整)

// 每个实例使用不同的初始温度
temperature_i = base_temperature + i × diversity_delta
```

#### 4.2 早停机制

```rust
// 全局共享标志
shared_flag: AtomicBool

// 任一实例找到满足约束的优质解时
if cost < good_solution_threshold {
    shared_flag.store(true)
    // 其他实例检测到标志后提前终止
}
```

#### 4.3 实例数量自适应

根据数据规模自动调整：

| 学生数量 | 并行实例数 | 迭代次数 |
|---------|-----------|---------|
| < 500   | 4         | 300,000 |
| 500-1000| 8         | 400,000 |
| 1000-2000| 12       | 400,000 |
| > 2000  | 16        | 500,000 |

### 5. 高性能实现技术

#### 5.1 增量更新

使用缓存统计数据，避免重复计算：

```rust
struct CachedClassStats {
    total_sum: f64,           // 总分和
    student_count: usize,     // 学生数
    male_count: usize,        // 男生数
    female_count: usize,      // 女生数
    subject_sums: Vec<f64>,   // 各科总分
}
```

交换学生时增量更新：
```rust
// O(1) 复杂度更新统计
remove_student_from_class(A, student1)
remove_student_from_class(B, student2)
add_student_to_class(A, student2)
add_student_to_class(B, student1)
```

#### 5.2 并行计算

使用 Rayon 数据并行：

```rust
// 并行计算班级统计
classes.par_iter().map(|c| c.calculate_stats())

// 并行读取 Excel 数据
rows.par_iter().map(|row| parse_student(row))

// 并行运行多个优化实例
(0..num_instances).into_par_iter().map(|i| optimize(i))
```

## 📊 算法参数配置

### 默认参数

```rust
OptimizationParams {
    // 硬约束阈值
    max_score_diff: 1.0,              // 分数最大差值
    max_class_size_diff: 5,            // 人数最大差值
    max_gender_ratio_diff: 0.1,        // 性别比例最大差值
    
    // 硬约束惩罚权重
    total_score_penalty_weight: 1_000_000_000.0,
    subject_score_penalty_weight: 1_000_000_000.0,
    gender_ratio_penalty_weight: 100_000_000_000.0,
    penalty_power: 6,
    
    // 软约束优化权重
    total_variance_weight: 10.0,
    gender_variance_weight: 5000.0,
    subject_variance_weight: 50.0,
    
    // 模拟退火参数
    initial_temperature: 10_000.0,
    cooling_rate: 0.99990,
    
    // 其他参数
    good_solution_threshold: 1.0,
    reheat_after_iterations: 1_000,
}
```

### 预设配置

#### 宽松模式（更快，精度稍低）

```rust
let config = DivideConfig::new(num_classes)
    .with_optimization_params(OptimizationParams::relaxed());
```

#### 严格模式（较慢，精度更高）

```rust
let config = DivideConfig::new(num_classes)
    .with_optimization_params(OptimizationParams::strict());
```

#### 自适应模式（根据规模调整）

```rust
let config = DivideConfig::new(num_classes)
    .with_optimization_params(OptimizationParams::adaptive(student_count));
```

## 🔧 编程接口

### 基本用法

```rust
use fenban::core::{
    model::Student,
    algorithm::{divide_students, DivideConfig},
    io::{read_students_from_excel, export_classes_to_excel},
};

// 读取学生数据
let (students, subjects) = read_students_from_excel("students.xlsx")?;

// 配置分班参数
let config = DivideConfig::new(6)  // 6 个班级
    .with_iterations(500_000);     // 50 万次迭代

// 执行分班
let classes = divide_students(&students, config);

// 导出结果
let subject_refs: Vec<&str> = subjects.iter().map(|s| s.as_str()).collect();
export_classes_to_excel(&classes, "result.xlsx", &subject_refs)?;
```

### 高级用法

```rust
use fenban::core::algorithm::{DivideConfig, OptimizationParams, validate_constraints};

// 自定义优化参数
let params = OptimizationParams {
    max_score_diff: 0.5,  // 更严格的分数约束
    max_gender_ratio_diff: 0.05,  // 更严格的性别约束
    initial_temperature: 15_000.0,
    cooling_rate: 0.99995,
    ..Default::default()
};

let config = DivideConfig::new(8)
    .with_iterations(1_000_000)
    .with_optimization_params(params);

let classes = divide_students(&students, config);

// 验证约束
let validation = validate_constraints(&classes);
println!("分数约束满足: {}", validation.score_constraints_met);
println!("性别约束满足: {}", validation.gender_constraints_met);
println!("最大分数差: {:.2}", validation.max_score_diff);
println!("最大性别比例差: {:.2}", validation.max_gender_ratio_diff);
```

## 🏗️ 项目结构

```
fenban/
├── src/
│   ├── core/              # 核心算法模块
│   │   ├── algorithm.rs   # 分班算法实现
│   │   ├── model.rs       # 数据模型定义
│   │   ├── io.rs          # Excel 读写
│   │   ├── stats.rs       # 统计分析
│   │   └── mod.rs
│   ├── ui/                # 用户界面
│   │   ├── components/    # UI 组件
│   │   ├── views/         # 视图页面
│   │   └── mod.rs
│   ├── updater/           # 自动更新模块
│   ├── lib.rs
│   └── main.rs
├── assets/                # 资源文件
├── icons/                 # 图标资源
├── examples/              # 示例代码
├── Cargo.toml
└── README.md
```

## 🛠️ 技术栈

- **语言**: Rust 2024 Edition
- **GUI 框架**: [Dioxus](https://dioxuslabs.com/) 0.7
- **并行计算**: [Rayon](https://github.com/rayon-rs/rayon) 1.0
- **Excel 处理**: 
  - [calamine](https://github.com/tafia/calamine) 0.31 (读取)
  - [rust_xlsxwriter](https://github.com/jmcnamara/rust_xlsxwriter) 0.91 (写入)
- **随机数**: [rand](https://github.com/rust-random/rand) 0.9

## 🤝 贡献指南

欢迎贡献代码、报告问题或提出建议！

## 📝 许可证

本项目采用双许可证：

- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

您可以选择其中一个许可证使用本软件。

## 📚 参考文献

1. Kirkpatrick, S., Gelatt, C. D., & Vecchi, M. P. (1983). "Optimization by simulated annealing". *Science*, 220(4598), 671-680.
2. Graham, R. L. (1969). "Bounds on multiprocessing timing anomalies". *SIAM Journal on Applied Mathematics*, 17(2), 416-429.
3. Van Laarhoven, P. J., & Aarts, E. H. (1987). *Simulated annealing: Theory and applications*. Springer.

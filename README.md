# fenban

高性能均衡分班库 - 基于 Rust 实现的多约束优化算法

## 核心约束

1. **各科平均分差值 ≤ 1 分**（目标）
2. **总成绩平均分差值 ≤ 1 分**（目标）
3. **男女生比例差 ≤ 0.2**（目标）

## 性能

- 5000 学生在 **5 分钟内**完成分班 ✅
- 使用正态分布测试数据
- 接近满足所有约束（差值通常在 2-3 分以内）

**注意**：由于多路数分区问题是 NP-hard 问题，使用正态分布的真实数据时，算法会尽可能接近满足约束，但不保证严格满足 1 分差值。实际应用中效果良好。

## 安装

```toml
[dependencies]
fenban = { path = "." }
```

## 快速开始

```rust
use fenban::{read_students_from_excel, divide_students, validate_constraints, DivideConfig};

// 1. 读取 Excel
let (students, _subjects) = read_students_from_excel("students.xlsx")?;

// 2. 配置分班
let config = DivideConfig::new(20)  // 分成 20 个班
    .with_iterations(200000);       // 迭代次数

// 3. 执行分班
let classes = divide_students(&students, config);

// 4. 验证约束
let validation = validate_constraints(&classes);
println!("总分约束: {}", validation.score_constraints_met);
println!("性别约束: {}", validation.gender_constraints_met);
```

## Excel 格式

| 姓名 | 性别 | 语文 | 数学 | 外语 | 物理 | 化学 | 生物 | 政治 | 历史 | 地理 |
|------|------|------|------|------|------|------|------|------|------|------|
| 张三 | 男   | 120  | 135  | 128  | 92   | 88   | 85   | 90   | 88   | 86   |
| 李四 | 女   | 125  | 130  | 132  | 95   | 90   | 88   | 92   | 90   | 89   |

**要求：**
- 第一行：表头（姓名、性别、科目名）
- 第二列：性别必须为 "男" 或 "女"
- 科目数量不限，自动识别
- 支持 .xlsx 格式

## 推荐参数

| 学生数      | 迭代次数 | 预计耗时 |
|-------------|----------|----------|
| < 500       | 300,000  | < 1 秒   |
| 500-1,000   | 400,000  | 1-5 秒   |
| 1,000-2,000 | 400,000  | 5-15 秒  |
| 2,000-5,000 | 500,000  | 15-60 秒 |
| > 5,000     | 500,000  | 1-5 分钟 |

## 运行示例

```bash
# 快速入门
cargo run --release --example quickstart

# 性能测试
cargo run --release --example performance_test

# 运行测试
cargo test --release
```

**注意：必须使用 --release 模式，性能提升 300+ 倍！**

## API

### `DivideConfig`

```rust
let config = DivideConfig::new(num_classes)
    .with_iterations(200000);
```

### `divide_students`

```rust
pub fn divide_students(students: &[Student], config: DivideConfig) -> Vec<Class>
```

### `validate_constraints`

```rust
pub struct ConstraintValidation {
    pub score_constraints_met: bool,        // 总分约束
    pub gender_constraints_met: bool,       // 性别约束
    pub max_score_diff: f64,                // 总分最大差值
    pub max_gender_ratio_diff: f64,         // 性别比例最大差值
    pub subject_max_diffs: Vec<(String, f64)>, // 各科最大差值
}
```

## 算法

- **LPT 初始化**：按总分降序分配到总分最低的班级
- **并行模拟退火**：多实例并行优化，选择最优解
- **增量计算**：只重新计算受影响的班级
- **硬约束惩罚**：权重 1,000,000×，确保必须满足约束

## 许可证

MIT License
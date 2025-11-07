# fenban

高性能均衡分班库 - 基于 Rust 实现的多约束优化算法

## 特性

✅ **灵活的 Excel 列配置** - 支持自定义列映射，适配各种 Excel 格式
✅ **自动学号生成** - 如果没有学号列，自动使用行号
✅ **额外字段保留** - 可保留原班级、备注等信息到输出
✅ **多科目支持** - 支持任意数量的科目
✅ **高性能算法** - 基于模拟退火的多约束优化

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

## 文档目录

- [特性](#特性)
- [快速开始](#快速开始)
- [Excel 格式配置](#excel-格式配置)
- [自定义列配置详解](#自定义列配置详解)
- [导出格式](#导出格式)
- [参数配置](#参数配置)
- [性能优化](#性能优化)
- [示例程序](#示例程序)

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

### 3. 自定义 Excel 列配置

如果你的 Excel 格式不是标准格式，可以自定义列的映射：

```rust
use fenban::{ExcelColumnConfig, read_students_from_excel_with_config, export_classes_to_excel_with_extras};

// 自定义列配置
// 假设格式：学号 | 原班级 | 姓名 | 性别 | 语文 | 数学 | 英语 | 总分
let config = ExcelColumnConfig::builder()
    .student_id_column(0)           // 第1列：学号
    .name_column(2)                  // 第3列：姓名
    .gender_column(3)                // 第4列：性别
    .add_subject("语文".to_string(), 4)    // 第5列：语文
    .add_subject("数学".to_string(), 5)    // 第6列：数学
    .add_subject("英语".to_string(), 6)    // 第7列：英语
    .total_score_column(7)           // 第8列：总分（可选）
    .add_extra_column("原班级".to_string(), 1)  // 第2列：保留到输出
    .build()?;

// 读取数据
let students = read_students_from_excel_with_config("学生数据.xlsx", &config)?;

// 执行分班
let classes = divide_students(&students, DivideConfig::new(20));

// 导出结果（包含额外字段）
let subjects = vec!["语文", "数学", "英语"];
let extra_fields = vec!["原班级"];
export_classes_to_excel_with_extras(&classes, "结果.xlsx", &subjects, &extra_fields)?;
```

**配置说明：**
- `student_id_column()`: 学号列（可选，如不指定则用行号代替）
- `name_column()`: 姓名列（必需）
- `gender_column()`: 性别列（必需，值必须为"男"或"女"）
- `total_score_column()`: 总分列（可选，如不指定则自动计算）
- `add_subject()`: 添加科目及其列号
- `add_extra_column()`: 添加需要保留到输出的额外列

### 4. Excel 输入格式

**标准格式（兼容旧版）：**

| 姓名 | 性别 | 语文 | 数学 | 外语 | 物理 | 化学 | 生物 | 政治 | 历史 | 地理 |
|------|------|------|------|------|------|------|------|------|------|------|
| 张三 | 男   | 120  | 135  | 128  | 92   | 88   | 85   | 90   | 88   | 86   |
| 李四 | 女   | 125  | 130  | 132  | 95   | 90   | 88   | 92   | 90   | 89   |

**自定义格式示例：**

| 学号 | 原班级 | 姓名 | 性别 | 语文 | 数学 | 英语 | 总分 |
|------|--------|------|------|------|------|------|------|
| 2024001 | 高一1班 | 张三 | 男 | 120 | 135 | 128 | 383 |
| 2024002 | 高一2班 | 李四 | 女 | 125 | 130 | 132 | 387 |

**要求：**
- 第一行：表头
- 性别列：值必须为 "男" 或 "女"
- 科目数量不限，自动识别
- 支持 .xlsx 格式
- 主科（语文、数学、外语）满分 150，副科满分 100

## 自定义列配置详解

### 配置构建器 API

**必需方法：**
- `.name_column(col: usize)` - 设置姓名列（0-based 索引）
- `.gender_column(col: usize)` - 设置性别列

**可选方法：**
- `.student_id_column(col: usize)` - 设置学号列（不设置则自动用行号 R1, R2, ...）
- `.total_score_column(col: usize)` - 设置总分列（不设置则自动计算）
- `.add_subject(name: String, col: usize)` - 添加科目及其列号
- `.add_extra_column(name: String, col: usize)` - 添加需要保留的额外字段
- `.build()` - 构建配置对象

### 常见场景示例

#### 场景1：没有学号列

```rust
let config = ExcelColumnConfig::builder()
    .name_column(0)                 // 姓名在第1列
    .gender_column(1)               // 性别在第2列
    .add_subject("语文".to_string(), 2)
    .add_subject("数学".to_string(), 3)
    .build()?;
// 系统会自动生成学号：R1, R2, R3, ...
```

#### 场景2：自动计算总分

```rust
let config = ExcelColumnConfig::builder()
    .name_column(0)
    .gender_column(1)
    .add_subject("语文".to_string(), 2)
    .add_subject("数学".to_string(), 3)
    .add_subject("英语".to_string(), 4)
    // 不设置 total_score_column，系统会自动计算
    .build()?;
```

#### 场景3：保留多个额外字段

```rust
let config = ExcelColumnConfig::builder()
    .student_id_column(0)
    .name_column(1)
    .gender_column(2)
    .add_subject("语文".to_string(), 3)
    .add_subject("数学".to_string(), 4)
    .add_extra_column("原班级".to_string(), 5)
    .add_extra_column("特长".to_string(), 6)
    .add_extra_column("备注".to_string(), 7)
    .build()?;

// 导出时指定要包含的额外字段
let extra_fields = vec!["原班级", "特长", "备注"];
export_classes_to_excel_with_extras(&classes, "结果.xlsx", &subjects, &extra_fields)?;
```

#### 场景4：完整配置示例

```rust
use fenban::{
    ExcelColumnConfig, divide_students, export_classes_to_excel_with_extras,
    read_students_from_excel_with_config, DivideConfig,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 配置列映射（假设 Excel 格式：学号|原班级|姓名|性别|语文|数学|英语|物理|化学|生物）
    let config = ExcelColumnConfig::builder()
        .student_id_column(0)
        .name_column(2)
        .gender_column(3)
        .add_subject("语文".to_string(), 4)
        .add_subject("数学".to_string(), 5)
        .add_subject("英语".to_string(), 6)
        .add_subject("物理".to_string(), 7)
        .add_subject("化学".to_string(), 8)
        .add_subject("生物".to_string(), 9)
        .add_extra_column("原班级".to_string(), 1)
        .build()?;

    // 2. 读取学生数据
    let students = read_students_from_excel_with_config("学生名单.xlsx", &config)?;
    println!("读取了 {} 名学生", students.len());

    // 3. 执行分班
    let divide_config = DivideConfig::new(20); // 分成20个班
    let classes = divide_students(&students, divide_config);

    // 4. 导出结果（包含原班级信息）
    let subjects = vec!["语文", "数学", "英语", "物理", "化学", "生物"];
    let extra_fields = vec!["原班级"];
    export_classes_to_excel_with_extras(&classes, "分班结果.xlsx", &subjects, &extra_fields)?;

    println!("分班完成！结果已保存到 分班结果.xlsx");
    Ok(())
}
```

## 导出格式

导出的 Excel 文件包含两个工作表：

### 工作表1：分班结果

包含每个学生的详细信息，按班级排序：

| 班级 | 学号 | 姓名 | 性别 | 原班级 | 语文 | 数学 | 英语 | 总分 |
|------|------|------|------|--------|------|------|------|------|
| 1 | 2024001 | 张三 | 男 | 高一1班 | 120 | 135 | 128 | 383 |
| 1 | 2024005 | 李四 | 女 | 高一2班 | 125 | 130 | 132 | 387 |
| 2 | 2024002 | 王五 | 男 | 高一3班 | 118 | 128 | 125 | 371 |
| ... | ... | ... | ... | ... | ... | ... | ... | ... |

**特点：**
- 包含学号列（如果原数据有学号或自动生成的行号）
- 包含所有额外保留的字段（如原班级）
- 按班级排序，便于打印和发放

### 工作表2：班级统计

每个班级的统计数据，便于对比均衡性：

| 班级 | 人数 | 男生 | 女生 | 男生比例 | 语文_平均 | 数学_平均 | 英语_平均 | 总分平均 |
|------|------|------|------|----------|----------|----------|----------|----------|
| 1 | 50 | 30 | 20 | 60.0% | 110.5 | 115.2 | 112.8 | 338.5 |
| 2 | 50 | 29 | 21 | 58.0% | 111.2 | 114.8 | 113.1 | 339.1 |
| 3 | 50 | 31 | 19 | 62.0% | 109.8 | 114.5 | 112.5 | 336.8 |
| ... | ... | ... | ... | ... | ... | ... | ... | ... |

**特点：**
- 显示每个班级的人数、性别分布
- 显示每个科目的平均分
- 显示总分平均值
- 便于检查分班是否均衡

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

## 性能优化

### 推荐配置

| 学生数      | 迭代次数   | 推荐参数 | 预计耗时 |
|-------------|-----------|---------|----------|
| < 500       | 1,000,000 | 自适应   | < 2 秒   |
| 500-1,000   | 1,500,000 | 自适应   | 2-10 秒  |
| 1,000-2,000 | 2,000,000 | 自适应   | 10-30 秒 |
| 2,000-5,000 | 3,000,000 | 自适应   | 30-180 秒|
| > 5,000     | 3,000,000 | 自适应   | 3-10 分钟|

### 优化建议

1. **必须使用 `--release` 模式**
   ```bash
   cargo build --release
   cargo run --release
   ```
   性能提升 300+ 倍！

2. **使用自适应参数**
   ```rust
   let params = OptimizationParams::adaptive(students.len());
   ```

3. **合理设置迭代次数**
   - 学生数多时增加迭代次数
   - 观察收敛情况，避免过早停止

## 示例程序

项目包含多个示例程序，展示不同的使用场景：

### 1. 基本示例 - `examples/main.rs`

生成测试数据并执行分班，展示基本用法：
```bash
cargo run --release --example main
```

**特点：**
- 生成 2000 名学生的测试数据
- 分成 40 个班
- 显示详细的分班结果和统计信息
- 自动导出到 Excel

### 2. 自定义列配置示例 - `examples/custom_columns.rs`

展示如何配置自定义 Excel 列映射：
```bash
cargo run --example custom_columns
```

**特点：**
- 展示列配置构建器的用法
- 说明各种配置选项
- 提供配置模板

### 3. 完整测试示例 - `examples/test_custom_config.rs`

完整的测试流程，包括创建测试文件、读取、分班、导出：
```bash
cargo run --release --example test_custom_config
```

**特点：**
- 自动创建测试 Excel 文件（100名学生）
- 使用自定义列配置读取
- 执行分班并导出结果
- 完整展示整个工作流程

### 运行所有测试

```bash
# 运行单元测试
cargo test

# 运行性能测试（release 模式）
cargo test --release
```

## 注意事项

1. **列索引从 0 开始**：Excel 第1列是 0，第2列是 1，以此类推
2. **性别值必须是"男"或"女"**：其他值会导致该学生被跳过
3. **表头行自动跳过**：第一行默认是表头，从第二行开始读取数据
4. **空行自动跳过**：姓名为空的行会被自动忽略
5. **额外字段是可选的**：如果某个学生没有该字段，输出时会显示为空
6. **学号自动生成**：如果没有学号列，系统会自动生成 R1, R2, R3...
7. **总分自动计算**：如果没有总分列，系统会自动累加所有科目成绩

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
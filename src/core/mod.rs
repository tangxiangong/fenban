pub mod algorithm;
pub mod history;
pub mod io;
pub mod model;
pub mod stats;

// 导出核心功能
pub use algorithm::{ConstraintValidation, DivideConfig, divide, validate_constraints};
pub use io::{ColumnConfig, ExcelColumnConfigBuilder, export_to_excel, read_from_excel};
pub use model::{Class, Gender, Student};
pub use stats::{
    DetailedStatistics, GenderBalance, Statistics, SubjectStatistics,
    calculate_detailed_statistics, calculate_statistics,
};

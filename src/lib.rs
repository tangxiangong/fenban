pub mod algorithm;
pub mod io;
pub mod model;
pub mod stats;

// 导出核心功能
pub use algorithm::{ConstraintValidation, DivideConfig, divide_students, validate_constraints};
pub use io::{export_classes_to_excel, read_students_from_excel};
pub use model::{Class, Gender, Student};
pub use stats::{
    DetailedStatistics, GenderBalance, Statistics, SubjectStatistics,
    calculate_detailed_statistics, calculate_statistics,
};

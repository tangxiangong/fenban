pub mod algorithm;
pub mod history;
pub mod io;
pub mod model;
pub mod stats;

// 导出核心功能
pub use algorithm::{ConstraintValidation, DivideConfig, divide_students, validate_constraints};
pub use io::{
    ExcelColumnConfig, ExcelColumnConfigBuilder, export_classes_to_excel,
    export_classes_to_excel_with_extras, read_students_from_excel,
    read_students_from_excel_with_config,
};
pub use model::{Class, Gender, Student};
pub use stats::{
    DetailedStatistics, GenderBalance, Statistics, SubjectStatistics,
    calculate_detailed_statistics, calculate_statistics,
};

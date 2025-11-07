#[derive(Clone, Debug, PartialEq)]
pub enum AppStep {
    SelectFile,
    PreviewData,
    ConfigureColumns,
    ConfigureDivision,
    Processing,
    Results,
}

#[derive(Clone, Debug)]
pub struct ColumnMapping {
    pub name: String,
    pub index: usize,
    pub column_type: ColumnType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ColumnType {
    Name,
    Gender,
    StudentId,
    TotalScore,
    Subject,
    Extra,
    Ignore,
}

impl ColumnType {
    pub fn to_string(&self) -> &'static str {
        match self {
            ColumnType::Name => "name",
            ColumnType::Gender => "gender",
            ColumnType::StudentId => "student_id",
            ColumnType::TotalScore => "total",
            ColumnType::Subject => "subject",
            ColumnType::Extra => "extra",
            ColumnType::Ignore => "ignore",
        }
    }

    pub fn from_string(s: &str) -> ColumnType {
        match s {
            "name" => ColumnType::Name,
            "gender" => ColumnType::Gender,
            "student_id" => ColumnType::StudentId,
            "total" => ColumnType::TotalScore,
            "subject" => ColumnType::Subject,
            "extra" => ColumnType::Extra,
            _ => ColumnType::Ignore,
        }
    }
}

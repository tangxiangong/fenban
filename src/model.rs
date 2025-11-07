use rayon::prelude::*;
use std::collections::HashMap;
use std::str::FromStr;

/// 性别枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gender {
    Male,
    Female,
}

impl FromStr for Gender {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "男" => Ok(Gender::Male),
            "女" => Ok(Gender::Female),
            _ => Err(format!("无效的性别: {}", s)),
        }
    }
}

/// 学生数据结构
#[derive(Debug, Clone)]
pub struct Student {
    pub name: String,
    pub gender: Gender,
    pub scores: HashMap<String, f64>,
    pub total_score: f64,
}

impl Student {
    pub fn new(name: String, gender: Gender, scores: HashMap<String, f64>) -> Self {
        let total_score = scores.values().sum();
        Self {
            name,
            gender,
            scores,
            total_score,
        }
    }
}

/// 班级数据结构
#[derive(Debug, Clone)]
pub struct Class {
    pub id: usize,
    pub students: Vec<Student>,
}

impl Class {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            students: Vec::with_capacity(64),
        }
    }

    #[inline]
    pub fn add_student(&mut self, student: Student) {
        self.students.push(student);
    }

    #[inline]
    pub fn male_count(&self) -> usize {
        self.students
            .par_iter()
            .filter(|s| s.gender == Gender::Male)
            .count()
    }

    #[inline]
    pub fn female_count(&self) -> usize {
        self.students
            .par_iter()
            .filter(|s| s.gender == Gender::Female)
            .count()
    }

    #[inline]
    pub fn avg_total_score(&self) -> f64 {
        if self.students.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.students.par_iter().map(|s| s.total_score).sum();
        sum / self.students.len() as f64
    }

    #[inline]
    pub fn avg_subject_score(&self, subject: &str) -> f64 {
        if self.students.is_empty() {
            return 0.0;
        }
        let sum: f64 = self
            .students
            .par_iter()
            .filter_map(|s| s.scores.get(subject).copied())
            .sum();
        sum / self.students.len() as f64
    }

    #[inline]
    pub fn variance(&self, mean: f64) -> f64 {
        if self.students.is_empty() {
            return 0.0;
        }
        let avg = self.avg_total_score();
        (avg - mean).powi(2)
    }

    /// 获取所有科目名称
    #[inline]
    pub fn all_subjects(&self) -> Vec<String> {
        if self.students.is_empty() {
            return vec![];
        }
        self.students[0].scores.keys().cloned().collect()
    }

    /// 计算各科平均分的方差
    #[inline]
    pub fn subject_variances(&self, subjects: &[String]) -> Vec<f64> {
        subjects
            .iter()
            .map(|subject| {
                let scores: Vec<f64> = self
                    .students
                    .iter()
                    .filter_map(|s| s.scores.get(subject).copied())
                    .collect();

                if scores.is_empty() {
                    return 0.0;
                }

                let mean = scores.iter().sum::<f64>() / scores.len() as f64;
                scores.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / scores.len() as f64
            })
            .collect()
    }

    /// 获取性别比例
    #[inline]
    pub fn gender_ratio(&self) -> f64 {
        let total = self.students.len() as f64;
        if total == 0.0 {
            return 0.0;
        }
        self.male_count() as f64 / total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gender_parsing() {
        assert_eq!("男".parse::<Gender>().unwrap(), Gender::Male);
        assert_eq!("女".parse::<Gender>().unwrap(), Gender::Female);
        assert!("unknown".parse::<Gender>().is_err());
    }

    #[test]
    fn test_student_creation() {
        let mut scores = HashMap::new();
        scores.insert("Math".to_string(), 90.0);
        scores.insert("English".to_string(), 85.0);

        let student = Student::new("Test".to_string(), Gender::Male, scores);
        assert_eq!(student.total_score, 175.0);
    }

    #[test]
    fn test_class_statistics() {
        let mut class = Class::new(0);

        let student1 = Student {
            name: "Student1".to_string(),
            gender: Gender::Male,
            scores: HashMap::new(),
            total_score: 600.0,
        };

        let student2 = Student {
            name: "Student2".to_string(),
            gender: Gender::Female,
            scores: HashMap::new(),
            total_score: 700.0,
        };

        class.add_student(student1);
        class.add_student(student2);

        assert_eq!(class.male_count(), 1);
        assert_eq!(class.female_count(), 1);
        assert_eq!(class.avg_total_score(), 650.0);
    }
}

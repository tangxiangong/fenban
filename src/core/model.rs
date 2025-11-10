use rayon::prelude::*;
use std::{collections::HashMap, str::FromStr};

/// 性别枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gender {
    Male,
    Female,
}

impl FromStr for Gender {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "男" => Ok(Gender::Male),
            "女" => Ok(Gender::Female),
            _ => Err(anyhow::anyhow!("无效的性别: {}", s)),
        }
    }
}

/// 学生数据结构
#[derive(Debug, Clone)]
pub struct Student {
    pub name: String,
    pub id: Option<String>,
    pub gender: Gender,
    pub scores: HashMap<String, f64>,
    pub total_score: f64,
    pub extra_fields: HashMap<String, String>,
}

impl Student {
    pub fn new(name: String, gender: Gender, scores: HashMap<String, f64>) -> Self {
        let total_score = scores.values().sum();
        Self {
            name,
            id: None,
            gender,
            scores,
            total_score,
            extra_fields: HashMap::new(),
        }
    }

    pub fn with_id(mut self, id: Option<String>) -> Self {
        self.id = id;
        self
    }

    pub fn with_extra_fields(mut self, extra_fields: HashMap<String, String>) -> Self {
        self.extra_fields = extra_fields;
        self
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

use crate::core::algorithm::OptimizationParams;
use anyhow::Result;
use fs_err as fs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryRecord {
    pub timestamp: String,
    pub input_path: String,
    pub output_path: Option<String>,
    pub num_classes: usize,
    pub num_students: usize,
    pub format: String, // "xlsx" or "csv"
    #[serde(default)]
    pub optimization_params: OptimizationParams,
}

impl HistoryRecord {
    pub fn new(
        input_path: String,
        output_path: Option<String>,
        num_classes: usize,
        num_students: usize,
        format: String,
        optimization_params: OptimizationParams,
    ) -> Self {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self {
            timestamp,
            input_path,
            output_path,
            num_classes,
            num_students,
            format,
            optimization_params,
        }
    }
}

pub struct HistoryManager {
    history_file: PathBuf,
}

impl HistoryManager {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("无法获取配置目录"))?
            .join("FenBan");

        fs::create_dir_all(&config_dir)?;

        Ok(Self {
            history_file: config_dir.join("history.json"),
        })
    }

    pub fn load(&self) -> Result<Vec<HistoryRecord>> {
        if !self.history_file.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.history_file)?;

        if content.trim().is_empty() {
            return Ok(Vec::new());
        }

        match serde_json::from_str(&content) {
            Ok(records) => Ok(records),
            Err(_) => {
                // 格式错误，清空文件
                let _ = fs::remove_file(&self.history_file);
                Ok(Vec::new())
            }
        }
    }

    fn save(&self, records: &[HistoryRecord]) -> Result<()> {
        let content = serde_json::to_string_pretty(records)?;
        fs::write(&self.history_file, content)?;
        Ok(())
    }

    pub fn add(&self, record: HistoryRecord) -> Result<()> {
        let mut records = self.load().unwrap_or_default();
        records.insert(0, record); // 最新的记录在前面

        // 只保留最近 50 条记录
        if records.len() > 50 {
            records.truncate(50);
        }

        self.save(&records)?;
        Ok(())
    }

    pub fn clear(&self) -> Result<()> {
        if self.history_file.exists() {
            fs::remove_file(&self.history_file)?;
        }
        Ok(())
    }

    pub fn delete(&self, timestamp: &str) -> Result<()> {
        let mut records = self.load()?;
        records.retain(|r| r.timestamp != timestamp);
        self.save(&records)?;
        Ok(())
    }
}

//! Cron 配置管理
//!
//! 从 ~/.alou/cron.json 加载和保存配置

use std::path::PathBuf;
use std::fs;
use std::io::Write;
use crate::cron::types::CronConfig;

/// Cron 配置管理器
pub struct CronConfigManager {
    config_path: PathBuf,
}

impl CronConfigManager {
    /// 创建新的配置管理器
    pub fn new() -> Result<Self, String> {
        let config_path = Self::get_config_path()?;
        Ok(Self { config_path })
    }

    /// 获取配置文件路径 (~/.alou/cron.json)
    pub fn get_config_path() -> Result<PathBuf, String> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| "无法获取 home 目录".to_string())?;

        let alou_dir = home_dir.join(".alou");

        // 确保目录存在
        if !alou_dir.exists() {
            fs::create_dir_all(&alou_dir)
                .map_err(|e| format!("创建 ~/.alou 目录失败：{}", e))?;
        }

        Ok(alou_dir.join("cron.json"))
    }

    /// 从文件加载配置
    pub fn load_config(&self) -> Result<CronConfig, String> {
        if !self.config_path.exists() {
            // 文件不存在，创建默认配置
            let default_config = CronConfig::default_with_template();
            self.save_config(&default_config)?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(&self.config_path)
            .map_err(|e| format!("读取配置文件失败：{}", e))?;

        let config: CronConfig = serde_json::from_str(&content)
            .map_err(|e| format!("解析配置文件失败：{}", e))?;

        Ok(config)
    }

    /// 保存配置到文件
    pub fn save_config(&self, config: &CronConfig) -> Result<(), String> {
        // 确保目录存在
        if let Some(parent) = self.config_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("创建配置目录失败：{}", e))?;
            }
        }

        let content = serde_json::to_string_pretty(config)
            .map_err(|e| format!("序列化配置失败：{}", e))?;

        // 使用临时文件确保原子写入
        let temp_path = self.config_path.with_extension("json.tmp");
        let mut temp_file = fs::File::create(&temp_path)
            .map_err(|e| format!("创建临时文件失败：{}", e))?;

        temp_file.write_all(content.as_bytes())
            .map_err(|e| format!("写入临时文件失败：{}", e))?;
        temp_file.sync_all()
            .map_err(|e| format!("同步临时文件失败：{}", e))?;

        // 原子替换
        fs::rename(&temp_path, &self.config_path)
            .map_err(|e| format!("替换配置文件失败：{}", e))?;

        println!("[CronConfig] 配置已保存到：{}", self.config_path.display());
        Ok(())
    }

    /// 检查配置文件是否存在
    pub fn config_exists(&self) -> bool {
        self.config_path.exists()
    }

    /// 创建默认配置文件
    pub fn create_default_config(&self) -> Result<CronConfig, String> {
        let default_config = CronConfig::default_with_template();
        self.save_config(&default_config)?;
        Ok(default_config)
    }

    /// 获取配置目录路径
    pub fn get_config_dir() -> Result<PathBuf, String> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| "无法获取 home 目录".to_string())?;

        let alou_dir = home_dir.join(".alou");

        // 确保目录存在
        if !alou_dir.exists() {
            fs::create_dir_all(&alou_dir)
                .map_err(|e| format!("创建 ~/.alou 目录失败：{}", e))?;
        }

        Ok(alou_dir)
    }
}

impl Default for CronConfigManager {
    fn default() -> Self {
        Self::new().expect("Failed to create CronConfigManager")
    }
}

/// 快速加载配置的辅助函数
pub fn load_cron_config() -> Result<CronConfig, String> {
    let manager = CronConfigManager::new()?;
    manager.load_config()
}

/// 快速保存配置的辅助函数
pub fn save_cron_config(config: &CronConfig) -> Result<(), String> {
    let manager = CronConfigManager::new()?;
    manager.save_config(config)
}

/// 获取配置文件路径的辅助函数
pub fn get_cron_config_path() -> Result<PathBuf, String> {
    CronConfigManager::get_config_path()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_config_path() {
        let path = CronConfigManager::get_config_path();
        assert!(path.is_ok());
        let path = path.unwrap();
        assert!(path.ends_with("cron.json"));
    }

    #[test]
    fn test_default_config() {
        let config = CronConfig::default_with_template();
        assert!(!config.jobs.is_empty());
        assert_eq!(config.check_interval_seconds, 60);
    }
}

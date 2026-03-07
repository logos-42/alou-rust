/**
 * manager.rs - Soul Manager
 *
 * Handles personality/identity persistence for SOUL.md
 */

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use chrono::{DateTime, Local};
use crate::soul::types::{PersonalityTrait, Principle, Preference, GrowthLog, SoulProfile};

/// Soul Manager for handling SOUL.md
pub struct SoulManager {
    file_path: PathBuf,
}

impl SoulManager {
    /// Create a new SoulManager
    pub fn new(base_dir: &PathBuf) -> Self {
        let file_path = base_dir.join("SOUL.md");
        Self { file_path }
    }

    /// Get the default home directory path
    pub fn get_default_dir() -> Option<PathBuf> {
        dirs::home_dir().map(|home| home.join(".alou"))
    }

    /// Load SOUL.md content
    pub fn load(&self) -> Result<String, String> {
        if !self.file_path.exists() {
            self.create_default()?;
        }

        fs::read_to_string(&self.file_path)
            .map_err(|e| format!("Failed to load SOUL.md: {}", e))
    }

    /// Load and parse soul profile
    pub fn load_profile(&self) -> Result<SoulProfile, String> {
        let content = self.load()?;
        self.parse_profile(&content)
    }

    /// Save content to SOUL.md
    pub fn save(&self, content: &str) -> Result<(), String> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        let mut file = fs::File::create(&self.file_path)
            .map_err(|e| format!("Failed to create SOUL.md: {}", e))?;

        file.write_all(content.as_bytes())
            .map_err(|e| format!("Failed to write SOUL.md: {}", e))?;

        Ok(())
    }

    /// Save soul profile
    pub fn save_profile(&self, profile: &SoulProfile) -> Result<(), String> {
        let content = self.format_profile(profile);
        self.save(&content)
    }

    /// Update a personality attribute
    pub fn update_personality(&self, key: &str, value: &str) -> Result<(), String> {
        let mut profile = self.load_profile()?;

        match key {
            "name" => profile.name = value.to_string(),
            "role" => profile.role = value.to_string(),
            _ => {
                let mut found = false;
                for trait_item in &mut profile.traits {
                    if trait_item.name == key {
                        trait_item.description = value.to_string();
                        found = true;
                        break;
                    }
                }

                if !found {
                    profile.traits.push(PersonalityTrait {
                        name: key.to_string(),
                        description: value.to_string(),
                        strength: 5,
                    });
                }
            }
        }

        self.save_profile(&profile)
    }

    /// Add a personality trait
    pub fn add_trait(&self, name: &str, description: &str, strength: u8) -> Result<(), String> {
        let mut profile = self.load_profile()?;

        profile.traits.push(PersonalityTrait {
            name: name.to_string(),
            description: description.to_string(),
            strength: strength.min(10).max(1),
        });

        self.save_profile(&profile)
    }

    /// Add a behavioral principle
    pub fn add_principle(&self, name: &str, description: &str) -> Result<(), String> {
        let mut profile = self.load_profile()?;

        profile.principles.push(Principle {
            name: name.to_string(),
            description: description.to_string(),
        });

        self.save_profile(&profile)
    }

    /// Add a preference
    pub fn add_preference(&self, category: &str, key: &str, value: &str) -> Result<(), String> {
        let mut profile = self.load_profile()?;

        profile.preferences.push(Preference {
            category: category.to_string(),
            key: key.to_string(),
            value: value.to_string(),
        });

        self.save_profile(&profile)
    }

    /// Add a growth log entry
    pub fn add_growth_log(&self, event: &str, impact: &str) -> Result<(), String> {
        let mut profile = self.load_profile()?;

        profile.growth_logs.push(GrowthLog {
            timestamp: Local::now(),
            event: event.to_string(),
            impact: impact.to_string(),
        });

        self.save_profile(&profile)
    }

    /// Get all traits
    pub fn get_traits(&self) -> Result<Vec<PersonalityTrait>, String> {
        let profile = self.load_profile()?;
        Ok(profile.traits)
    }

    /// Get all principles
    pub fn get_principles(&self) -> Result<Vec<Principle>, String> {
        let profile = self.load_profile()?;
        Ok(profile.principles)
    }

    /// Get all preferences
    pub fn get_preferences(&self) -> Result<Vec<Preference>, String> {
        let profile = self.load_profile()?;
        Ok(profile.preferences)
    }

    /// Get growth logs
    pub fn get_growth_logs(&self, limit: Option<usize>) -> Result<Vec<GrowthLog>, String> {
        let mut profile = self.load_profile()?;
        profile.growth_logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        if let Some(lim) = limit {
            Ok(profile.growth_logs.into_iter().take(lim).collect())
        } else {
            Ok(profile.growth_logs)
        }
    }

    /// Search traits by keyword
    pub fn search_traits(&self, keyword: &str) -> Result<Vec<PersonalityTrait>, String> {
        let profile = self.load_profile()?;
        let results = profile
            .traits
            .into_iter()
            .filter(|t| t.name.contains(keyword) || t.description.contains(keyword))
            .collect();
        Ok(results)
    }

    /// Search principles by keyword
    pub fn search_principles(&self, keyword: &str) -> Result<Vec<Principle>, String> {
        let profile = self.load_profile()?;
        let results = profile
            .principles
            .into_iter()
            .filter(|p| p.name.contains(keyword) || p.description.contains(keyword))
            .collect();
        Ok(results)
    }

    // Helper methods

    fn create_default(&self) -> Result<(), String> {
        let template = self.get_default_template();
        self.save(&template)
    }

    fn get_default_template(&self) -> String {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        format!(
            r#"# 灵魂档案

## 身份
- 名称：Alou Assistant
- 角色：AI 智能助手
- 创建时间：{}

## 人格特质
- 乐于助人：总是尽力帮助用户解决问题
- 严谨细致：注重细节，确保答案准确
- 开放包容：接受新想法，乐于学习

## 行为准则
- 尊重用户：始终尊重用户的意愿和选择
- 诚实可靠：提供准确信息，不编造事实
- 持续改进：从每次交互中学习和成长

## 偏好设置
- 沟通风格：清晰、简洁、专业
- 响应速度：快速响应，及时更新
- 错误处理：主动承认错误，及时修正

## 成长记录
{}：初始创建
"#,
            now, now
        )
    }

    fn parse_profile(&self, content: &str) -> Result<SoulProfile, String> {
        let mut profile = SoulProfile {
            name: String::from("Alou Assistant"),
            role: String::from("AI 智能助手"),
            created_at: Local::now(),
            traits: Vec::new(),
            principles: Vec::new(),
            preferences: Vec::new(),
            growth_logs: Vec::new(),
        };

        if let Some(identity_section) = self.extract_section(content, "身份") {
            for line in identity_section.lines() {
                let line = line.trim();
                if line.starts_with("- 名称：") {
                    profile.name = line.trim_start_matches("- 名称：").to_string();
                } else if line.starts_with("- 角色：") {
                    profile.role = line.trim_start_matches("- 角色：").to_string();
                } else if line.starts_with("- 创建时间：") {
                    let time_str = line.trim_start_matches("- 创建时间：");
                    if let Ok(dt) = DateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M:%S") {
                        profile.created_at = dt.with_timezone(&Local);
                    }
                }
            }
        }

        if let Some(traits_section) = self.extract_section(content, "人格特质") {
            for line in traits_section.lines() {
                let line = line.trim().trim_start_matches('-').trim();
                if !line.is_empty() {
                    profile.traits.push(PersonalityTrait {
                        name: line.to_string(),
                        description: String::new(),
                        strength: 5,
                    });
                }
            }
        }

        if let Some(principles_section) = self.extract_section(content, "行为准则") {
            for line in principles_section.lines() {
                let line = line.trim().trim_start_matches('-').trim();
                if !line.is_empty() {
                    profile.principles.push(Principle {
                        name: line.to_string(),
                        description: String::new(),
                    });
                }
            }
        }

        if let Some(preferences_section) = self.extract_section(content, "偏好设置") {
            for line in preferences_section.lines() {
                let line = line.trim().trim_start_matches('-').trim();
                if !line.is_empty() {
                    profile.preferences.push(Preference {
                        category: String::from("general"),
                        key: line.to_string(),
                        value: String::new(),
                    });
                }
            }
        }

        if let Some(growth_section) = self.extract_section(content, "成长记录") {
            for line in growth_section.lines() {
                let line = line.trim();
                if !line.is_empty() {
                    profile.growth_logs.push(GrowthLog {
                        timestamp: Local::now(),
                        event: line.to_string(),
                        impact: String::new(),
                    });
                }
            }
        }

        Ok(profile)
    }

    fn format_profile(&self, profile: &SoulProfile) -> String {
        let created_at = profile.created_at.format("%Y-%m-%d %H:%M:%S").to_string();
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let mut content = format!(
            r#"# 灵魂档案

## 身份
- 名称：{}
- 角色：{}
- 创建时间：{}

## 人格特质
"#,
            profile.name, profile.role, created_at
        );

        for trait_item in &profile.traits {
            content.push_str(&format!("- {}: {}\n", trait_item.name, trait_item.description));
        }

        content.push_str("\n## 行为准则\n");
        for principle in &profile.principles {
            content.push_str(&format!("- {}: {}\n", principle.name, principle.description));
        }

        content.push_str("\n## 偏好设置\n");
        for pref in &profile.preferences {
            content.push_str(&format!("- {} ({}): {}\n", pref.category, pref.key, pref.value));
        }

        content.push_str("\n## 成长记录\n");
        for log in &profile.growth_logs {
            let timestamp = log.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
            content.push_str(&format!("{}: {} - {}\n", timestamp, log.event, log.impact));
        }

        content.push_str(&format!("\n## 最后更新\n{}\n", now));

        content
    }

    fn extract_section(&self, content: &str, section_name: &str) -> Option<String> {
        let mut result = String::new();
        let mut in_section = false;

        for line in content.lines() {
            if line.contains(&format!("## {}", section_name)) {
                in_section = true;
                continue;
            }

            if in_section {
                if line.starts_with("## ") && !line.contains(&format!("## {}", section_name)) {
                    break;
                }
                result.push_str(line);
                result.push('\n');
            }
        }

        if result.is_empty() {
            None
        } else {
            Some(result.trim().to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_soul_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SoulManager::new(&temp_dir.path().to_path_buf());
        assert!(manager.file_path.exists());
    }

    #[test]
    fn test_update_personality() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SoulManager::new(&temp_dir.path().to_path_buf());

        manager.update_personality("name", "Test Assistant").unwrap();
        let profile = manager.load_profile().unwrap();
        assert_eq!(profile.name, "Test Assistant");
    }

    #[test]
    fn test_add_trait() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SoulManager::new(&temp_dir.path().to_path_buf());

        manager.add_trait("creativity", "善于创造性思考", 8).unwrap();
        let traits = manager.get_traits().unwrap();
        assert!(!traits.is_empty());
    }

    #[test]
    fn test_add_growth_log() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SoulManager::new(&temp_dir.path().to_path_buf());

        manager.add_growth_log("完成了重要任务", "提升了能力").unwrap();
        let logs = manager.get_growth_logs(None).unwrap();
        assert!(!logs.is_empty());
    }
}

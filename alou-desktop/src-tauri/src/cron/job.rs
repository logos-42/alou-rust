//! Cron Job 实现
//!
//! 实现 CronJob 的核心方法

use chrono::{DateTime, Utc, Timelike, Datelike};
use uuid::Uuid;
use crate::cron::types::CronJob;

/// 解析 Cron 表达式的结果
#[derive(Debug, Clone)]
pub struct CronSchedule {
    pub minute: Vec<u32>,
    pub hour: Vec<u32>,
    pub day_of_month: Vec<u32>,
    pub month: Vec<u32>,
    pub day_of_week: Vec<u32>,
}

impl CronJob {
    /// 判断在当前时间是否应该运行
    ///
    /// # Arguments
    /// * `current_time` - 当前时间
    ///
    /// # Returns
    /// * `bool` - 是否应该运行
    pub fn should_run(&self, current_time: DateTime<Utc>) -> bool {
        // 检查任务是否启用
        if !self.is_enabled() {
            return false;
        }

        // 解析 Cron 表达式
        let schedule = match self.parse_cron_expression(&self.schedule) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("[CronJob] 解析 Cron 表达式失败：{}", self.schedule);
                return false;
            }
        };

        // 检查是否匹配当前时间
        self.matches_schedule(current_time, &schedule)
    }

    /// 创建独立会话 ID
    ///
    /// # Returns
    /// * `String` - 新生成的 session_id
    pub fn create_isolated_session(&self) -> String {
        let timestamp = Utc::now().timestamp_nanos_opt().unwrap_or(0);
        format!("cron_{}_{}", self.name, timestamp)
    }

    /// 生成基于 UUID 的会话 ID
    ///
    /// # Returns
    /// * `String` - UUID 格式的 session_id
    pub fn create_isolated_session_uuid(&self) -> String {
        format!("cron_{}_{}", self.name, Uuid::new_v4())
    }

    /// 解析 Cron 表达式
    ///
    /// 格式：分 时 日 月 星期
    /// 示例："0 8 * * *" = 每天早上 8 点
    ///
    /// # Arguments
    /// * `expression` - Cron 表达式字符串
    ///
    /// # Returns
    /// * `Result<CronSchedule, String>` - 解析结果
    fn parse_cron_expression(&self, expression: &str) -> Result<CronSchedule, String> {
        let parts: Vec<&str> = expression.trim().split_whitespace().collect();

        if parts.len() != 5 {
            return Err(format!(
                "无效的 Cron 表达式：需要 5 个部分（分 时 日 月 星期），实际得到 {} 个",
                parts.len()
            ));
        }

        Ok(CronSchedule {
            minute: self.parse_field(parts[0], 0, 59)?,
            hour: self.parse_field(parts[1], 0, 23)?,
            day_of_month: self.parse_field(parts[2], 1, 31)?,
            month: self.parse_field(parts[3], 1, 12)?,
            day_of_week: self.parse_field(parts[4], 0, 6)?, // 0=Sunday, 6=Saturday
        })
    }

    /// 解析 Cron 字段
    ///
    /// # Arguments
    /// * `field` - 字段字符串
    /// * `min` - 最小值
    /// * `max` - 最大值
    ///
    /// # Returns
    /// * `Result<Vec<u32>, String>` - 解析后的值列表
    fn parse_field(&self, field: &str, min: u32, max: u32) -> Result<Vec<u32>, String> {
        // 处理通配符
        if field == "*" {
            return Ok((min..=max).collect());
        }

        // 处理步长（如 */5）
        if let Some(step_pos) = field.find('/') {
            let range = &field[..step_pos];
            let step: u32 = field[step_pos + 1..]
                .parse()
                .map_err(|_| format!("无效的步长值：{}", &field[step_pos + 1..]))?;

            let values = if range == "*" {
                (min..=max).collect()
            } else {
                self.parse_range(range, min, max)?
            };

            return Ok(values.into_iter().step_by(step as usize).collect());
        }

        // 处理范围（如 1-5）和列表（如 1,3,5）
        self.parse_range(field, min, max)
    }

    /// 解析范围或列表
    ///
    /// # Arguments
    /// * `range` - 范围字符串
    /// * `min` - 最小值
    /// * `max` - 最大值
    ///
    /// # Returns
    /// * `Result<Vec<u32>, String>` - 解析后的值列表
    fn parse_range(&self, range: &str, min: u32, max: u32) -> Result<Vec<u32>, String> {
        let mut values = Vec::new();

        for part in range.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            // 处理范围（如 1-5）
            if let Some(dash_pos) = part.find('-') {
                let start: u32 = part[..dash_pos]
                    .parse()
                    .map_err(|_| format!("无效的范围起始值：{}", &part[..dash_pos]))?;
                let end: u32 = part[dash_pos + 1..]
                    .parse()
                    .map_err(|_| format!("无效的范围结束值：{}", &part[dash_pos + 1..]))?;

                if start > end {
                    return Err(format!("范围起始值 {} 大于结束值 {}", start, end));
                }

                for i in start..=end {
                    if i >= min && i <= max {
                        values.push(i);
                    }
                }
            } else {
                // 单个值
                let value: u32 = part
                    .parse()
                    .map_err(|_| format!("无效的值：{}", part))?;

                if value < min || value > max {
                    return Err(format!("值 {} 超出范围 [{}, {}]", value, min, max));
                }

                values.push(value);
            }
        }

        if values.is_empty() {
            return Err(format!("字段 '{}' 未解析出任何有效值", range));
        }

        Ok(values)
    }

    /// 检查时间是否匹配调度
    ///
    /// # Arguments
    /// * `time` - 要检查的时间
    /// * `schedule` - 解析后的 Cron 调度
    ///
    /// # Returns
    /// * `bool` - 是否匹配
    fn matches_schedule(&self, time: DateTime<Utc>, schedule: &CronSchedule) -> bool {
        let naive = time.naive_utc();
        let minute = naive.minute() as u32;
        let hour = naive.hour() as u32;
        let day = naive.day() as u32;
        let month = naive.month() as u32;
        let day_of_week = naive.weekday().num_days_from_sunday();

        // 检查每个字段是否匹配
        schedule.minute.contains(&minute)
            && schedule.hour.contains(&hour)
            && schedule.day_of_month.contains(&day)
            && schedule.month.contains(&month)
            && schedule.day_of_week.contains(&day_of_week)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_run_morning_8am() {
        let job = CronJob::new(
            "test".to_string(),
            "0 8 * * *".to_string(),
            "test prompt".to_string(),
            true,
            None,
        );

        // 早上 8:00 应该运行
        let time_8am = Utc.with_ymd_and_hms(2024, 1, 15, 8, 0, 0).unwrap();
        assert!(job.should_run(time_8am));

        // 早上 8:30 不应该运行（只在 0 分运行）
        let time_830am = Utc.with_ymd_and_hms(2024, 1, 15, 8, 30, 0).unwrap();
        assert!(!job.should_run(time_830am));

        // 下午 2:00 不应该运行
        let time_2pm = Utc.with_ymd_and_hms(2024, 1, 15, 14, 0, 0).unwrap();
        assert!(!job.should_run(time_2pm));
    }

    #[test]
    fn test_create_isolated_session() {
        let job = CronJob::new(
            "daily_summary".to_string(),
            "0 8 * * *".to_string(),
            "test prompt".to_string(),
            true,
            None,
        );

        let session1 = job.create_isolated_session();
        let session2 = job.create_isolated_session();

        assert!(session1.starts_with("cron_daily_summary_"));
        assert!(session2.starts_with("cron_daily_summary_"));
        assert_ne!(session1, session2); // 每次生成的 session_id 应该不同
    }

    #[test]
    fn test_create_isolated_session_uuid() {
        let job = CronJob::new(
            "daily_summary".to_string(),
            "0 8 * * *".to_string(),
            "test prompt".to_string(),
            true,
            None,
        );

        let session = job.create_isolated_session_uuid();
        assert!(session.starts_with("cron_daily_summary_"));
        assert!(session.len() > 36); // UUID 长度 + 前缀
    }
}

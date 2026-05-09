/// Health Checks - Comprehensive health monitoring for Alou system
///
/// This module provides health check functionality for various system components
/// including cron jobs, tasks, network, models, and system resources.

use std::sync::Arc;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};

/// Health level indicating the severity of the status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HealthLevel {
    /// Everything is working normally
    Good,
    /// Some issues detected but system is functional
    Warning,
    /// Critical issues requiring immediate attention
    Critical,
}

impl std::fmt::Display for HealthLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthLevel::Good => write!(f, "good"),
            HealthLevel::Warning => write!(f, "warning"),
            HealthLevel::Critical => write!(f, "critical"),
        }
    }
}

/// Issue severity level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IssueSeverity {
    /// Minor issue, no immediate action needed
    Low,
    /// Moderate issue, should be addressed soon
    Medium,
    /// Serious issue, action recommended
    High,
    /// Critical issue, immediate action required
    Critical,
}

impl std::fmt::Display for IssueSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueSeverity::Low => write!(f, "low"),
            IssueSeverity::Medium => write!(f, "medium"),
            IssueSeverity::High => write!(f, "high"),
            IssueSeverity::Critical => write!(f, "critical"),
        }
    }
}

/// Result of a single health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// Name of the check
    pub name: String,
    /// Status of the check
    pub status: HealthLevel,
    /// Human-readable message
    pub message: String,
    /// Optional detailed information
    pub details: Option<String>,
}

impl CheckResult {
    pub fn new(name: String, status: HealthLevel, message: String) -> Self {
        Self {
            name,
            status,
            message,
            details: None,
        }
    }

    pub fn with_details(mut self, details: String) -> Self {
        self.details = Some(details);
        self
    }

    pub fn good(name: String, message: String) -> Self {
        Self::new(name, HealthLevel::Good, message)
    }

    pub fn warning(name: String, message: String) -> Self {
        Self::new(name, HealthLevel::Warning, message)
    }

    pub fn critical(name: String, message: String) -> Self {
        Self::new(name, HealthLevel::Critical, message)
    }
}

/// Represents a detected issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// Severity of the issue
    pub severity: IssueSeverity,
    /// Description of the issue
    pub description: String,
    /// Component affected by the issue
    pub affected_component: String,
    /// When the issue was detected
    pub detected_at: DateTime<Utc>,
}

impl Issue {
    pub fn new(
        severity: IssueSeverity,
        description: String,
        affected_component: String,
    ) -> Self {
        Self {
            severity,
            description,
            affected_component,
            detected_at: Utc::now(),
        }
    }

    pub fn critical(description: String, component: String) -> Self {
        Self::new(IssueSeverity::Critical, description, component)
    }

    pub fn high(description: String, component: String) -> Self {
        Self::new(IssueSeverity::High, description, component)
    }

    pub fn medium(description: String, component: String) -> Self {
        Self::new(IssueSeverity::Medium, description, component)
    }

    pub fn low(description: String, component: String) -> Self {
        Self::new(IssueSeverity::Low, description, component)
    }
}

/// Overall health status of the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Overall health level
    pub overall: HealthLevel,
    /// Individual check results
    pub checks: Vec<CheckResult>,
    /// Detected issues
    pub issues: Vec<Issue>,
    /// Recommendations for improvement
    pub recommendations: Vec<String>,
}

impl HealthStatus {
    pub fn new() -> Self {
        Self {
            overall: HealthLevel::Good,
            checks: Vec::new(),
            issues: Vec::new(),
            recommendations: Vec::new(),
        }
    }

    pub fn add_check(&mut self, check: CheckResult) {
        // Update overall status if this check is worse
        if self.is_worse(&check.status) {
            self.overall = check.status.clone();
        }
        self.checks.push(check);
    }

    pub fn add_issue(&mut self, issue: Issue) {
        self.issues.push(issue);
    }

    pub fn add_recommendation(&mut self, recommendation: String) {
        if !self.recommendations.contains(&recommendation) {
            self.recommendations.push(recommendation);
        }
    }

    fn is_worse(&self, status: &HealthLevel) -> bool {
        match (status, &self.overall) {
            (HealthLevel::Critical, _) => true,
            (HealthLevel::Warning, HealthLevel::Good) => true,
            _ => false,
        }
    }

    pub fn has_critical_issues(&self) -> bool {
        self.issues.iter().any(|i| i.severity == IssueSeverity::Critical)
    }

    pub fn has_issues(&self) -> bool {
        !self.issues.is_empty()
    }
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self::new()
    }
}

/// Cron job health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronHealthStatus {
    /// Overall status
    pub status: HealthLevel,
    /// Total number of jobs
    pub total_jobs: usize,
    /// Number of jobs that missed their scheduled run
    pub missed_jobs: usize,
    /// Number of failed jobs in last 24 hours
    pub failed_jobs: usize,
    /// Details about missed jobs
    pub missed_job_details: Vec<MissedJobInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissedJobInfo {
    pub job_name: String,
    pub last_run: Option<DateTime<Utc>>,
    pub should_have_run_at: DateTime<Utc>,
    pub hours_overdue: i64,
}

/// Task health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskHealthStatus {
    /// Overall status
    pub status: HealthLevel,
    /// Total number of tasks
    pub total_tasks: usize,
    /// Number of stuck tasks
    pub stuck_tasks: usize,
    /// Number of tasks in queue
    pub queued_tasks: usize,
    /// Details about stuck tasks
    pub stuck_task_details: Vec<StuckTaskInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StuckTaskInfo {
    pub task_id: String,
    pub task_name: String,
    pub stuck_since: DateTime<Utc>,
    pub hours_stuck: i64,
    pub current_status: String,
}

/// Network health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkHealthStatus {
    /// Overall status
    pub status: HealthLevel,
    /// API connection status
    pub api_connected: bool,
    /// IPFS node status
    pub ipfs_connected: bool,
    /// Last successful API call
    pub last_api_call: Option<DateTime<Utc>>,
    /// Last successful IPFS operation
    pub last_ipfs_operation: Option<DateTime<Utc>>,
    /// Connection latency in ms
    pub latency_ms: Option<u64>,
}

/// Model health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelHealthStatus {
    /// Overall status
    pub status: HealthLevel,
    /// Current model being used
    pub current_model: String,
    /// Error rate in last hour (0.0 - 1.0)
    pub error_rate: f64,
    /// Average response time in ms
    pub avg_response_time_ms: Option<u64>,
    /// Number of failed requests in last hour
    pub failed_requests: usize,
    /// Quality score (0.0 - 1.0)
    pub quality_score: Option<f64>,
}

/// System health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthStatus {
    /// Overall status
    pub status: HealthLevel,
    /// CPU usage percentage (0.0 - 100.0)
    pub cpu_usage_percent: f64,
    /// Memory usage percentage (0.0 - 100.0)
    pub memory_usage_percent: f64,
    /// Available disk space in GB
    pub available_disk_gb: Option<f64>,
    /// Disk usage percentage (0.0 - 100.0)
    pub disk_usage_percent: Option<f64>,
    /// Number of running processes
    pub process_count: usize,
}

/// Health Checker - performs comprehensive health checks
pub struct HealthChecker {
    /// Threshold for cron job overdue time (hours)
    cron_overdue_threshold_hours: i64,
    /// Threshold for task stuck time (hours)
    task_stuck_threshold_hours: i64,
    /// Threshold for high error rate (0.0 - 1.0)
    high_error_rate_threshold: f64,
    /// Threshold for CPU usage warning (percentage)
    cpu_warning_threshold: f64,
    /// Threshold for memory usage warning (percentage)
    memory_warning_threshold: f64,
    /// Threshold for disk space warning (GB)
    disk_warning_threshold_gb: f64,
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthChecker {
    /// Create a new HealthChecker with default thresholds
    pub fn new() -> Self {
        Self {
            cron_overdue_threshold_hours: 26, // More than a day
            task_stuck_threshold_hours: 24,
            high_error_rate_threshold: 0.3, // 30% error rate
            cpu_warning_threshold: 80.0,
            memory_warning_threshold: 85.0,
            disk_warning_threshold_gb: 5.0,
        }
    }

    /// Create with custom thresholds
    pub fn with_thresholds(
        cron_overdue_hours: i64,
        task_stuck_hours: i64,
        error_rate_threshold: f64,
        cpu_threshold: f64,
        memory_threshold: f64,
        disk_threshold_gb: f64,
    ) -> Self {
        Self {
            cron_overdue_threshold_hours: cron_overdue_hours,
            task_stuck_threshold_hours: task_stuck_hours,
            high_error_rate_threshold: error_rate_threshold,
            cpu_warning_threshold: cpu_threshold,
            memory_warning_threshold: memory_threshold,
            disk_warning_threshold_gb: disk_threshold_gb,
        }
    }

    /// Run all health checks and return comprehensive status
    pub async fn run_all_checks(&self) -> HealthStatus {
        let mut status = HealthStatus::new();

        // Run individual checks
        let cron_status = self.check_cron_health().await;
        let task_status = self.check_task_health().await;
        let network_status = self.check_network_health().await;
        let model_status = self.check_model_health().await;
        let system_status = self.check_system_health().await;

        // Add check results
        status.add_check(self.cron_to_check_result(&cron_status));
        status.add_check(self.task_to_check_result(&task_status));
        status.add_check(self.network_to_check_result(&network_status));
        status.add_check(self.model_to_check_result(&model_status));
        status.add_check(self.system_to_check_result(&system_status));

        // Convert statuses to issues
        self.add_cron_issues(&mut status, &cron_status);
        self.add_task_issues(&mut status, &task_status);
        self.add_network_issues(&mut status, &network_status);
        self.add_model_issues(&mut status, &model_status);
        self.add_system_issues(&mut status, &system_status);

        // Generate recommendations
        self.generate_recommendations(&mut status);

        status
    }

    /// Check cron job health
    pub async fn check_cron_health(&self) -> CronHealthStatus {
        println!("[HealthCheck] Checking cron job health...");

        let mut missed_jobs = Vec::new();
        let mut failed_jobs = 0;
        let total_jobs = 0; // Will be populated from cron manager

        // Try to get cron jobs from the system
        // In a real implementation, this would query the cron scheduler
        // For now, we'll check the cron configuration file

        let cron_config_path = dirs::home_dir()
            .map(|h| h.join(".alou/cron.json"))
            .and_then(|p| if p.exists() { Some(p) } else { None });

        if let Some(path) = cron_config_path {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(jobs) = config.get("jobs").and_then(|j| j.as_array()) {
                        let now = Utc::now();
                        let threshold = Duration::hours(self.cron_overdue_threshold_hours);

                        for job in jobs {
                            if let Some(enabled) = job.get("enabled").and_then(|e| e.as_bool()) {
                                if !enabled {
                                    continue;
                                }
                            }

                            // Parse cron schedule and check last run
                            // This is a simplified check - real implementation would use cron parser
                            if let Some(name) = job.get("name").and_then(|n| n.as_str()) {
                                // Check if job should have run but didn't
                                // For now, we'll just check if the job exists and is enabled
                                // Real implementation would track last run times

                                // Simulate checking last run time
                                let last_run: Option<DateTime<Utc>> = job
                                    .get("last_run")
                                    .and_then(|lr| lr.as_str())
                                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                                    .map(|dt| dt.with_timezone(&Utc));

                                if let Some(lr) = last_run {
                                    let elapsed = now.signed_duration_since(lr);
                                    if elapsed > threshold {
                                        missed_jobs.push(MissedJobInfo {
                                            job_name: name.to_string(),
                                            last_run: Some(lr),
                                            should_have_run_at: lr + Duration::hours(24),
                                            hours_overdue: (elapsed.num_seconds() / 3600) - 24,
                                        });
                                    }
                                } else {
                                    // No last run recorded - might be a new job or missed
                                    // Check if it's a daily job that should have run today
                                    if let Some(schedule) = job.get("schedule").and_then(|s| s.as_str()) {
                                        if schedule.contains("* * *") {
                                            // Daily job without last run - likely missed
                                            missed_jobs.push(MissedJobInfo {
                                                job_name: name.to_string(),
                                                last_run: None,
                                                should_have_run_at: now - Duration::hours(12),
                                                hours_overdue: self.cron_overdue_threshold_hours,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let status = if !missed_jobs.is_empty() {
            HealthLevel::Warning
        } else {
            HealthLevel::Good
        };

        CronHealthStatus {
            status,
            total_jobs: total_jobs.max(1), // At least 1 to avoid division by zero
            missed_jobs: missed_jobs.len(),
            failed_jobs,
            missed_job_details: missed_jobs,
        }
    }

    /// Check task health
    pub async fn check_task_health(&self) -> TaskHealthStatus {
        println!("[HealthCheck] Checking task health...");

        let mut stuck_tasks = Vec::new();
        let mut queued_tasks = 0;
        let total_tasks = 0;

        // Check task queue file
        let task_queue_path = dirs::home_dir()
            .map(|h| h.join(".alou/tasks"))
            .and_then(|p| if p.exists() { Some(p) } else { None });

        if let Some(path) = task_queue_path {
            // Check for task files
            if let Ok(entries) = std::fs::read_dir(&path) {
                let now = Utc::now();
                let threshold = Duration::hours(self.task_stuck_threshold_hours);

                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("json") {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(task) = serde_json::from_str::<serde_json::Value>(&content) {
                                if let Some(status) = task.get("status").and_then(|s| s.as_str()) {
                                    if status == "running" || status == "pending" {
                                        // Check last update time
                                        let last_update: Option<DateTime<Utc>> = task
                                            .get("updated_at")
                                            .or_else(|| task.get("created_at"))
                                            .and_then(|t| t.as_str())
                                            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                                            .map(|dt| dt.with_timezone(&Utc));

                                        if let Some(lu) = last_update {
                                            let elapsed = now.signed_duration_since(lu);
                                            if elapsed > threshold {
                                                stuck_tasks.push(StuckTaskInfo {
                                                    task_id: path.file_stem()
                                                        .and_then(|s| s.to_str())
                                                        .unwrap_or("unknown")
                                                        .to_string(),
                                                    task_name: task.get("name")
                                                        .and_then(|n| n.as_str())
                                                        .unwrap_or("Unknown Task")
                                                        .to_string(),
                                                    stuck_since: lu,
                                                    hours_stuck: elapsed.num_seconds() / 3600,
                                                    current_status: status.to_string(),
                                                });
                                            }
                                        }

                                        if status == "pending" {
                                            queued_tasks += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Also check TASKS.md file
        let tasks_md_path = dirs::current_dir()
            .map(|p| p.join("TASKS.md"))
            .or_else(|| dirs::home_dir().map(|h| h.join("TASKS.md")))
            .and_then(|p| if p.exists() { Some(p) } else { None });

        if let Some(path) = tasks_md_path {
            if let Ok(content) = std::fs::read_to_string(&path) {
                // Simple heuristic: check for tasks that look stuck
                // Look for "In Progress" tasks without recent updates
                if content.contains("In Progress") || content.contains("进行中") {
                    // This is a simplified check - real implementation would parse the markdown
                    // and check timestamps
                }
            }
        }

        let status = if !stuck_tasks.is_empty() {
            if stuck_tasks.len() > 3 {
                HealthLevel::Critical
            } else {
                HealthLevel::Warning
            }
        } else if queued_tasks > 10 {
            HealthLevel::Warning
        } else {
            HealthLevel::Good
        };

        TaskHealthStatus {
            status,
            total_tasks: total_tasks.max(1),
            stuck_tasks: stuck_tasks.len(),
            queued_tasks,
            stuck_task_details: stuck_tasks,
        }
    }

    /// Check network health
    pub async fn check_network_health(&self) -> NetworkHealthStatus {
        println!("[HealthCheck] Checking network health...");

        let mut api_connected = false;
        let mut ipfs_connected = false;
        let mut last_api_call: Option<DateTime<Utc>> = None;
        let mut last_ipfs_operation: Option<DateTime<Utc>> = None;
        let mut latency_ms: Option<u64> = None;

        // Check API connectivity
        // Try to read recent API activity from logs or state
        let alou_dir = dirs::home_dir().map(|h| h.join(".alou"));
        if let Some(dir) = alou_dir {
            // Check for API state file
            let api_state_path = dir.join("api_state.json");
            if api_state_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&api_state_path) {
                    if let Ok(state) = serde_json::from_str::<serde_json::Value>(&content) {
                        api_connected = state.get("connected").and_then(|c| c.as_bool()).unwrap_or(false);
                        last_api_call = state.get("last_call")
                            .and_then(|t| t.as_str())
                            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                            .map(|dt| dt.with_timezone(&Utc));
                        latency_ms = state.get("latency_ms").and_then(|l| l.as_u64());
                    }
                }
            }

            // Check IPFS state
            let ipfs_state_path = dir.join("ipfs_state.json");
            if ipfs_state_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&ipfs_state_path) {
                    if let Ok(state) = serde_json::from_str::<serde_json::Value>(&content) {
                        ipfs_connected = state.get("running").and_then(|r| r.as_bool()).unwrap_or(false);
                        last_ipfs_operation = state.get("last_operation")
                            .and_then(|t| t.as_str())
                            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                            .map(|dt| dt.with_timezone(&Utc));
                    }
                }
            }
        }

        // If no state files, try direct connectivity check
        if !api_connected {
            // Try a simple HTTP request to check connectivity
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build();

            if let Ok(client) = client {
                let start = std::time::Instant::now();
                match client.get("https://api.deepseek.com").send().await {
                    Ok(_) => {
                        api_connected = true;
                        latency_ms = Some(start.elapsed().as_millis() as u64);
                        last_api_call = Some(Utc::now());
                    }
                    Err(_) => {
                        // Try alternative endpoint
                        match client.get("https://httpbin.org/status/200").send().await {
                            Ok(_) => {
                                api_connected = true;
                                latency_ms = Some(start.elapsed().as_millis() as u64);
                            }
                            Err(_) => {
                                api_connected = false;
                            }
                        }
                    }
                }
            }
        }

        let status = if !api_connected && !ipfs_connected {
            HealthLevel::Critical
        } else if !api_connected || !ipfs_connected {
            HealthLevel::Warning
        } else {
            HealthLevel::Good
        };

        NetworkHealthStatus {
            status,
            api_connected,
            ipfs_connected,
            last_api_call,
            last_ipfs_operation,
            latency_ms,
        }
    }

    /// Check model health
    pub async fn check_model_health(&self) -> ModelHealthStatus {
        println!("[HealthCheck] Checking model health...");

        let mut current_model = String::from("deepseek-chat");
        let mut error_rate = 0.0;
        let mut avg_response_time_ms: Option<u64> = None;
        let mut failed_requests = 0;
        let mut quality_score: Option<f64> = None;

        // Read model state from logs or config
        let alou_dir = dirs::home_dir().map(|h| h.join(".alou"));
        if let Some(dir) = alou_dir {
            // Check heartbeat config for current model
            let hb_config_path = dir.join("heartbeat.json");
            if hb_config_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&hb_config_path) {
                    if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                        current_model = config.get("model")
                            .and_then(|m| m.as_str())
                            .unwrap_or("deepseek-chat")
                            .to_string();
                    }
                }
            }

            // Check for model usage stats
            let model_stats_path = dir.join("model_stats.json");
            if model_stats_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&model_stats_path) {
                    if let Ok(stats) = serde_json::from_str::<serde_json::Value>(&content) {
                        error_rate = stats.get("error_rate").and_then(|e| e.as_f64()).unwrap_or(0.0);
                        failed_requests = stats.get("failed_requests").and_then(|f| f.as_u64()).unwrap_or(0) as usize;
                        avg_response_time_ms = stats.get("avg_response_time_ms").and_then(|t| t.as_u64());
                        quality_score = stats.get("quality_score").and_then(|q| q.as_f64());
                    }
                }
            }
        }

        // Calculate status based on error rate
        let status = if error_rate > self.high_error_rate_threshold {
            HealthLevel::Critical
        } else if error_rate > 0.1 {
            HealthLevel::Warning
        } else {
            HealthLevel::Good
        };

        ModelHealthStatus {
            status,
            current_model,
            error_rate,
            avg_response_time_ms,
            failed_requests,
            quality_score,
        }
    }

    /// Check system health
    pub async fn check_system_health(&self) -> SystemHealthStatus {
        println!("[HealthCheck] Checking system health...");

        let mut cpu_usage = 0.0;
        let mut memory_usage = 0.0;
        let mut available_disk_gb: Option<f64> = None;
        let mut disk_usage_percent: Option<f64> = None;
        let mut process_count = 0;

        // Use sysinfo crate to get system stats
        use sysinfo::{ProcessRefreshKind, System};

        let mut sys = System::new();
        sys.refresh_cpu();
        sys.refresh_memory();
        sys.refresh_processes(ProcessRefreshKind::new());

        cpu_usage = sys.global_cpu_usage() as f64;
        memory_usage = (sys.used_memory() as f64 / sys.total_memory() as f64) * 100.0;
        process_count = sys.processes().len();

        // Get disk space
        if let Some(home_dir) = dirs::home_dir() {
            if let Ok(disk_usage) = get_disk_usage(&home_dir) {
                available_disk_gb = Some(disk_usage.available_gb);
                disk_usage_percent = Some(disk_usage.usage_percent);
            }
        }

        // Determine status
        let mut status = HealthLevel::Good;

        if cpu_usage > self.cpu_warning_threshold || memory_usage > self.memory_warning_threshold {
            status = HealthLevel::Warning;
        }

        if let Some(available) = available_disk_gb {
            if available < self.disk_warning_threshold_gb {
                status = HealthLevel::Critical;
            } else if available < self.disk_warning_threshold_gb * 2.0 {
                status = HealthLevel::Warning;
            }
        }

        if cpu_usage > 95.0 || memory_usage > 95.0 {
            status = HealthLevel::Critical;
        }

        SystemHealthStatus {
            status,
            cpu_usage_percent: cpu_usage,
            memory_usage_percent: memory_usage,
            available_disk_gb: available_disk_gb,
            disk_usage_percent: disk_usage_percent,
            process_count,
        }
    }

    // Helper methods to convert status to check results

    fn cron_to_check_result(&self, status: &CronHealthStatus) -> CheckResult {
        let message = match status.status {
            HealthLevel::Good => format!("All {} cron jobs healthy", status.total_jobs),
            HealthLevel::Warning => format!("{} cron jobs missed their scheduled run", status.missed_jobs),
            HealthLevel::Critical => format!("{} cron jobs failed, {} missed", status.failed_jobs, status.missed_jobs),
        };

        let mut check = CheckResult::new(
            "cron_health".to_string(),
            status.status.clone(),
            message,
        );

        if !status.missed_job_details.is_empty() {
            let details: Vec<String> = status.missed_job_details.iter()
                .map(|j| format!("{}: {} hours overdue", j.job_name, j.hours_overdue))
                .collect();
            check = check.with_details(details.join("; "));
        }

        check
    }

    fn task_to_check_result(&self, status: &TaskHealthStatus) -> CheckResult {
        let message = match status.status {
            HealthLevel::Good => format!("All {} tasks healthy", status.total_tasks),
            HealthLevel::Warning => format!("{} tasks stuck, {} queued", status.stuck_tasks, status.queued_tasks),
            HealthLevel::Critical => format!("{} tasks stuck - system may be overloaded", status.stuck_tasks),
        };

        let mut check = CheckResult::new(
            "task_health".to_string(),
            status.status.clone(),
            message,
        );

        if !status.stuck_task_details.is_empty() {
            let details: Vec<String> = status.stuck_task_details.iter()
                .map(|t| format!("{}: stuck for {} hours", t.task_name, t.hours_stuck))
                .collect();
            check = check.with_details(details.join("; "));
        }

        check
    }

    fn network_to_check_result(&self, status: &NetworkHealthStatus) -> CheckResult {
        let mut parts = Vec::new();

        if status.api_connected {
            parts.push("API connected");
            if let Some(latency) = status.latency_ms {
                parts.push(&format!("latency: {}ms", latency));
            }
        } else {
            parts.push("API disconnected");
        }

        if status.ipfs_connected {
            parts.push("IPFS connected");
        } else {
            parts.push("IPFS disconnected");
        }

        let message = parts.join(", ");

        CheckResult::new(
            "network_health".to_string(),
            status.status.clone(),
            message,
        )
    }

    fn model_to_check_result(&self, status: &ModelHealthStatus) -> CheckResult {
        let message = format!(
            "Model: {}, Error rate: {:.1}%, Failed requests: {}",
            status.current_model,
            status.error_rate * 100.0,
            status.failed_requests
        );

        let mut check = CheckResult::new(
            "model_health".to_string(),
            status.status.clone(),
            message,
        );

        if let Some(response_time) = status.avg_response_time_ms {
            check = check.with_details(format!("Avg response time: {}ms", response_time));
        }

        if let Some(quality) = status.quality_score {
            check = check.with_details(format!("Quality score: {:.2}", quality));
        }

        check
    }

    fn system_to_check_result(&self, status: &SystemHealthStatus) -> CheckResult {
        let mut parts = vec![
            format!("CPU: {:.1}%", status.cpu_usage_percent),
            format!("Memory: {:.1}%", status.memory_usage_percent),
        ];

        if let Some(disk) = status.available_disk_gb {
            parts.push(format!("Disk: {:.1}GB available", disk));
        }

        let message = parts.join(", ");

        CheckResult::new(
            "system_health".to_string(),
            status.status.clone(),
            message,
        )
    }

    // Methods to add issues based on status

    fn add_cron_issues(&self, status: &mut HealthStatus, cron: &CronHealthStatus) {
        for job in &cron.missed_job_details {
            status.add_issue(Issue::high(
                format!("Cron job '{}' is {} hours overdue", job.job_name, job.hours_overdue),
                "cron_scheduler".to_string(),
            ));
            status.add_recommendation(format!(
                "Consider re-running missed cron job: {}",
                job.job_name
            ));
        }
    }

    fn add_task_issues(&self, status: &mut HealthStatus, task: &TaskHealthStatus) {
        for stuck in &task.stuck_task_details {
            let severity = if stuck.hours_stuck > 48 {
                IssueSeverity::Critical
            } else {
                IssueSeverity::High
            };

            status.add_issue(Issue::new(
                severity,
                format!("Task '{}' has been stuck for {} hours", stuck.task_name, stuck.hours_stuck),
                "task_queue".to_string(),
            ));
            status.add_recommendation(format!(
                "Consider continuing or restarting stuck task: {}",
                stuck.task_name
            ));
        }

        if task.queued_tasks > 10 {
            status.add_issue(Issue::medium(
                format!("Task queue has {} pending tasks", task.queued_tasks),
                "task_queue".to_string(),
            ));
            status.add_recommendation("Consider processing queued tasks or increasing parallelism".to_string());
        }
    }

    fn add_network_issues(&self, status: &mut HealthStatus, network: &NetworkHealthStatus) {
        if !network.api_connected {
            status.add_issue(Issue::critical(
                "API connection lost".to_string(),
                "network".to_string(),
            ));
            status.add_recommendation("Check network connectivity and API credentials".to_string());
        }

        if !network.ipfs_connected {
            status.add_issue(Issue::high(
                "IPFS node is not running".to_string(),
                "ipfs".to_string(),
            ));
            status.add_recommendation("Restart IPFS daemon".to_string());
        }
    }

    fn add_model_issues(&self, status: &mut HealthStatus, model: &ModelHealthStatus) {
        if model.error_rate > self.high_error_rate_threshold {
            status.add_issue(Issue::critical(
                format!("Model error rate is critically high: {:.1}%", model.error_rate * 100.0),
                "model".to_string(),
            ));
            status.add_recommendation("Consider switching to a different model".to_string());
        } else if model.error_rate > 0.1 {
            status.add_issue(Issue::medium(
                format!("Model error rate is elevated: {:.1}%", model.error_rate * 100.0),
                "model".to_string(),
            ));
        }
    }

    fn add_system_issues(&self, status: &mut HealthStatus, system: &SystemHealthStatus) {
        if system.cpu_usage_percent > 95.0 {
            status.add_issue(Issue::critical(
                format!("CPU usage is critically high: {:.1}%", system.cpu_usage_percent),
                "system".to_string(),
            ));
            status.add_recommendation("Close other applications or reduce workload".to_string());
        } else if system.cpu_usage_percent > self.cpu_warning_threshold {
            status.add_issue(Issue::medium(
                format!("CPU usage is high: {:.1}%", system.cpu_usage_percent),
                "system".to_string(),
            ));
        }

        if system.memory_usage_percent > 95.0 {
            status.add_issue(Issue::critical(
                format!("Memory usage is critically high: {:.1}%", system.memory_usage_percent),
                "system".to_string(),
            ));
            status.add_recommendation("Close other applications or restart Alou".to_string());
        } else if system.memory_usage_percent > self.memory_warning_threshold {
            status.add_issue(Issue::medium(
                format!("Memory usage is high: {:.1}%", system.memory_usage_percent),
                "system".to_string(),
            ));
        }

        if let Some(available) = system.available_disk_gb {
            if available < 1.0 {
                status.add_issue(Issue::critical(
                    format!("Disk space critically low: {:.1}GB available", available),
                    "system".to_string(),
                ));
                status.add_recommendation("Clean up old logs and files immediately".to_string());
            } else if available < self.disk_warning_threshold_gb {
                status.add_issue(Issue::high(
                    format!("Disk space low: {:.1}GB available", available),
                    "system".to_string(),
                ));
                status.add_recommendation("Consider cleaning up old files".to_string());
            }
        }
    }

    fn generate_recommendations(&self, status: &mut HealthStatus) {
        // Add general recommendations based on overall status
        if status.overall == HealthLevel::Good {
            status.add_recommendation("System is healthy - continue regular monitoring".to_string());
        } else if status.overall == HealthLevel::Warning {
            status.add_recommendation("Review and address warning issues soon".to_string());
        } else {
            status.add_recommendation("Immediate attention required - critical issues detected".to_string());
        }
    }
}

/// Helper function to get disk usage information
fn get_disk_usage(path: &std::path::Path) -> Option<DiskUsage> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::FilesystemExt;

        if let Ok(metadata) = std::fs::metadata(path) {
            if let Ok(statvfs) = rustix::fs::statvfs(path) {
                let total_bytes = statvfs.blocks() * statvfs.block_size() as u64;
                let available_bytes = statvfs.blocks_available() * statvfs.block_size() as u64;
                let total_gb = total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                let available_gb = available_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                let usage_percent = if total_bytes > 0 {
                    ((total_bytes - available_bytes) as f64 / total_bytes as f64) * 100.0
                } else {
                    0.0
                };

                return Some(DiskUsage {
                    total_gb,
                    available_gb,
                    usage_percent,
                });
            }
        }
    }

    // Fallback for non-unix or if statvfs fails
    // Use a simple estimation based on home directory
    if let Ok(metadata) = std::fs::metadata(path) {
        // This is a rough estimation
        return Some(DiskUsage {
            total_gb: 512.0, // Assume 512GB total
            available_gb: 50.0, // Assume 50GB available as default
            usage_percent: 90.0,
        });
    }

    None
}

#[derive(Debug, Clone)]
struct DiskUsage {
    total_gb: f64,
    available_gb: f64,
    usage_percent: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_level_display() {
        assert_eq!(HealthLevel::Good.to_string(), "good");
        assert_eq!(HealthLevel::Warning.to_string(), "warning");
        assert_eq!(HealthLevel::Critical.to_string(), "critical");
    }

    #[test]
    fn test_issue_severity_display() {
        assert_eq!(IssueSeverity::Low.to_string(), "low");
        assert_eq!(IssueSeverity::Medium.to_string(), "medium");
        assert_eq!(IssueSeverity::High.to_string(), "high");
        assert_eq!(IssueSeverity::Critical.to_string(), "critical");
    }

    #[test]
    fn test_check_result_helpers() {
        let good = CheckResult::good("test".to_string(), "All good".to_string());
        assert_eq!(good.status, HealthLevel::Good);

        let warning = CheckResult::warning("test".to_string(), "Warning".to_string());
        assert_eq!(warning.status, HealthLevel::Warning);

        let critical = CheckResult::critical("test".to_string(), "Critical".to_string());
        assert_eq!(critical.status, HealthLevel::Critical);
    }

    #[test]
    fn test_health_status_aggregation() {
        let mut status = HealthStatus::new();

        // Add good check
        status.add_check(CheckResult::good("check1".to_string(), "OK".to_string()));
        assert_eq!(status.overall, HealthLevel::Good);

        // Add warning check - should upgrade overall status
        status.add_check(CheckResult::warning("check2".to_string(), "Warning".to_string()));
        assert_eq!(status.overall, HealthLevel::Warning);

        // Add critical check - should upgrade overall status
        status.add_check(CheckResult::critical("check3".to_string(), "Critical".to_string()));
        assert_eq!(status.overall, HealthLevel::Critical);
    }

    #[tokio::test]
    async fn test_health_checker_creation() {
        let checker = HealthChecker::new();
        assert_eq!(checker.cron_overdue_threshold_hours, 26);
        assert_eq!(checker.task_stuck_threshold_hours, 24);
    }
}

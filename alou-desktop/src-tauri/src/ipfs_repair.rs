//! IPFS/Kubo 节点修复模块
//!
//! 提供错误诊断和修复触发功能，支持 AI 自动修复 IPFS 节点问题

use serde::{Deserialize, Serialize};

/// IPFS 错误类型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IpfsErrorType {
    /// 二进制文件缺失
    BinaryMissing,
    /// 端口被占用
    PortConflict,
    /// 配置文件损坏
    ConfigCorrupted,
    /// 权限不足
    PermissionDenied,
    /// 网络问题
    NetworkError,
    /// 未知错误
    Unknown,
}

impl IpfsErrorType {
    /// 从错误消息推断错误类型
    pub fn from_error_message(error_msg: &str) -> Self {
        let msg_lower = error_msg.to_lowercase();

        if msg_lower.contains("binary not found")
            || msg_lower.contains("no such file")
            || msg_lower.contains("不存在")
        {
            IpfsErrorType::BinaryMissing
        } else if msg_lower.contains("port")
            || msg_lower.contains("5001")
            || msg_lower.contains("占用")
            || msg_lower.contains("already running")
            || msg_lower.contains("already running")
        {
            IpfsErrorType::PortConflict
        } else if msg_lower.contains("config")
            || msg_lower.contains("corrupt")
            || msg_lower.contains("损坏")
        {
            IpfsErrorType::ConfigCorrupted
        } else if msg_lower.contains("permission")
            || msg_lower.contains("denied")
            || msg_lower.contains("权限")
            || msg_lower.contains("access denied")
        {
            IpfsErrorType::PermissionDenied
        } else if msg_lower.contains("network")
            || msg_lower.contains("connection")
            || msg_lower.contains("proxy")
            || msg_lower.contains("网络")
            || msg_lower.contains("超时")
        {
            IpfsErrorType::NetworkError
        } else {
            IpfsErrorType::Unknown
        }
    }
}

/// IPFS 修复信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpfsRepairInfo {
    /// 错误类型
    pub error_type: IpfsErrorType,
    /// 原始错误消息
    pub error_message: String,
    /// 平台信息
    pub platform: String,
    /// 修复建议
    pub repair_hint: String,
    /// 是否可以使用 skill 修复
    pub can_auto_repair: bool,
}

impl IpfsRepairInfo {
    /// 创建修复信息
    pub fn new(error_type: IpfsErrorType, error_message: String, platform: String) -> Self {
        let repair_hint = Self::generate_hint(&error_type, &platform);
        let can_auto_repair = matches!(
            error_type,
            IpfsErrorType::BinaryMissing
                | IpfsErrorType::PortConflict
                | IpfsErrorType::ConfigCorrupted
                | IpfsErrorType::PermissionDenied
                | IpfsErrorType::NetworkError
        );

        Self {
            error_type,
            error_message,
            platform,
            repair_hint,
            can_auto_repair,
        }
    }

    /// 从启动错误创建修复信息
    pub fn from_launch_error(error_msg: &str) -> Self {
        let error_type = IpfsErrorType::from_error_message(error_msg);
        let platform = Self::detect_platform();

        Self::new(error_type, error_msg.to_string(), platform)
    }

    /// 检测当前平台
    fn detect_platform() -> String {
        #[cfg(target_os = "macos")]
        return "darwin".to_string();

        #[cfg(target_os = "linux")]
        return "linux".to_string();

        #[cfg(target_os = "windows")]
        return "win32".to_string();

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        return "unknown".to_string();
    }

    /// 生成修复提示
    fn generate_hint(error_type: &IpfsErrorType, platform: &str) -> String {
        match error_type {
            IpfsErrorType::BinaryMissing => {
                format!(
                    "请使用 kubo_repair skill 修复，参数: error_type=binary_missing, platform={}",
                    platform
                )
            }
            IpfsErrorType::PortConflict => {
                format!(
                    "请使用 kubo_repair skill 修复，参数: error_type=port_conflict, platform={}",
                    platform
                )
            }
            IpfsErrorType::ConfigCorrupted => {
                format!(
                    "请使用 kubo_repair skill 修复，参数: error_type=config_corrupted, platform={}",
                    platform
                )
            }
            IpfsErrorType::PermissionDenied => {
                format!(
                    "请使用 kubo_repair skill 修复，参数: error_type=permission_denied, platform={}",
                    platform
                )
            }
            IpfsErrorType::NetworkError => {
                format!(
                    "请使用 kubo_repair skill 修复，参数: error_type=network_error, platform={}",
                    platform
                )
            }
            IpfsErrorType::Unknown => "请使用 kubo_repair skill 进行诊断和修复".to_string(),
        }
    }
}

/// 检查 IPFS 状态并返回修复信息
pub fn check_and_get_repair_info(error_msg: &str) -> IpfsRepairInfo {
    IpfsRepairInfo::from_launch_error(error_msg)
}

use std::path::PathBuf;
use std::env;

/// 工作区上下文trait - 提供工作区目录信息
pub trait WorkspaceContext: std::fmt::Debug {
    fn get_directories(&self) -> Vec<PathBuf>;
    
    /// 获取主要工作目录（用于压缩传输）
    fn get_primary_directory(&self) -> PathBuf {
        self.get_directories().into_iter().next()
            .unwrap_or_else(|| PathBuf::from("."))
    }
    
    /// 获取压缩的上下文信息（仅包含关键信息）
    fn get_compressed_info(&self) -> String {
        let primary = self.get_primary_directory();
        primary.to_string_lossy().to_string()
    }
}

/// 基础工作区上下文 - 使用当前目录
#[derive(Debug)]
pub struct BasicWorkspaceContext;

impl BasicWorkspaceContext {
    pub fn new() -> Self { Self }
}

impl Default for BasicWorkspaceContext {
    fn default() -> Self { Self::new() }
}

impl WorkspaceContext for BasicWorkspaceContext {
    fn get_directories(&self) -> Vec<PathBuf> {
        vec![env::current_dir().unwrap_or_else(|_| PathBuf::from("."))]
    }
}

/// 自定义工作区上下文 - 使用指定目录
#[derive(Debug)]
pub struct CustomWorkspaceContext {
    directories: Vec<PathBuf>,
}

impl CustomWorkspaceContext {
    pub fn new(directories: Vec<PathBuf>) -> Self {
        Self { directories }
    }
}

impl WorkspaceContext for CustomWorkspaceContext {
    fn get_directories(&self) -> Vec<PathBuf> {
        self.directories.clone()
    }
}

/// 智能工作区上下文 - 自动检测项目根目录
#[derive(Debug)]
pub struct SmartWorkspaceContext {
    directories: Vec<PathBuf>,
}

impl SmartWorkspaceContext {
    pub fn new() -> Self {
        let mut directories = Vec::new();
        let mut current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        
        // 向上查找项目根目录
        for _ in 0..5 { // 限制搜索深度
            if Self::is_project_root(&current_dir) {
                directories.push(current_dir.clone());
            }
            if let Some(parent) = current_dir.parent() {
                current_dir = parent.to_path_buf();
            } else {
                break;
            }
        }
        
        // 如果没找到，使用当前目录
        if directories.is_empty() {
            directories.push(env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        }
        
        Self { directories }
    }

    fn is_project_root(path: &PathBuf) -> bool {
        ["Cargo.toml", "package.json", ".git", "pyproject.toml"]
            .iter()
            .any(|indicator| path.join(indicator).exists())
    }
}

impl Default for SmartWorkspaceContext {
    fn default() -> Self { Self::new() }
}

impl WorkspaceContext for SmartWorkspaceContext {
    fn get_directories(&self) -> Vec<PathBuf> {
        self.directories.clone()
    }
}

/// 工作区上下文工厂 - 简化创建接口
pub struct WorkspaceContextFactory;

impl WorkspaceContextFactory {
    pub fn create_basic() -> Box<dyn WorkspaceContext + Send + Sync> {
        Box::new(BasicWorkspaceContext::new())
    }

    pub fn create_custom(directories: Vec<PathBuf>) -> Box<dyn WorkspaceContext + Send + Sync> {
        Box::new(CustomWorkspaceContext::new(directories))
    }

    pub fn create_smart() -> Box<dyn WorkspaceContext + Send + Sync> {
        Box::new(SmartWorkspaceContext::new())
    }
}


use std::path::PathBuf;
use std::env;

/// 工作区上下文trait
/// 为MCP工具提供工作区上下文，允许工具了解当前工作环境，
/// 例如项目的根目录。
pub trait WorkspaceContext: std::fmt::Debug {
    /// 获取当前工作区的根目录
    /// 这些是包含项目文件的顶级目录
    /// 
    /// # Returns
    /// 工作区根目录的绝对路径数组
    fn get_directories(&self) -> Vec<PathBuf>;
}

/// 基础工作区上下文实现
/// 返回当前工作目录的默认实现
#[derive(Debug)]
pub struct BasicWorkspaceContext;

impl BasicWorkspaceContext {
    pub fn new() -> Self {
        Self
    }
}

impl Default for BasicWorkspaceContext {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceContext for BasicWorkspaceContext {
    fn get_directories(&self) -> Vec<PathBuf> {
        vec![env::current_dir().unwrap_or_else(|_| PathBuf::from("."))]
    }
}

/// 自定义工作区上下文实现
/// 允许设置自定义目录的工作区上下文
#[derive(Debug)]
pub struct CustomWorkspaceContext {
    directories: Vec<PathBuf>,
}

impl CustomWorkspaceContext {
    /// 创建新的自定义工作区上下文
    /// 
    /// # Arguments
    /// * `directories` - 工作区目录列表
    pub fn new(directories: Vec<PathBuf>) -> Self {
        Self { directories }
    }

    /// 从字符串路径创建自定义工作区上下文
    /// 
    /// # Arguments
    /// * `paths` - 路径字符串列表
    pub fn from_paths(paths: Vec<String>) -> Self {
        let directories = paths.into_iter().map(PathBuf::from).collect();
        Self::new(directories)
    }

    /// 添加目录到工作区
    /// 
    /// # Arguments
    /// * `directory` - 要添加的目录路径
    pub fn add_directory(&mut self, directory: PathBuf) {
        self.directories.push(directory);
    }

    /// 移除目录从工作区
    /// 
    /// # Arguments
    /// * `directory` - 要移除的目录路径
    pub fn remove_directory(&mut self, directory: &PathBuf) {
        self.directories.retain(|d| d != directory);
    }

    /// 获取目录数量
    pub fn len(&self) -> usize {
        self.directories.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.directories.is_empty()
    }
}

impl WorkspaceContext for CustomWorkspaceContext {
    fn get_directories(&self) -> Vec<PathBuf> {
        self.directories.clone()
    }
}

/// 智能工作区上下文
/// 自动检测项目根目录的智能实现
#[derive(Debug)]
pub struct SmartWorkspaceContext {
    directories: Vec<PathBuf>,
}

impl SmartWorkspaceContext {
    /// 创建新的智能工作区上下文
    /// 自动检测当前目录及其父目录中的项目根目录
    pub fn new() -> Self {
        let mut directories = Vec::new();
        
        // 从当前目录开始，向上查找项目根目录
        let mut current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        
        loop {
            // 检查是否为项目根目录（包含常见的项目标识文件）
            if Self::is_project_root(&current_dir) {
                directories.push(current_dir.clone());
            }
            
            // 移动到父目录
            if let Some(parent) = current_dir.parent() {
                current_dir = parent.to_path_buf();
            } else {
                break;
            }
            
            // 限制搜索深度，避免无限循环
            if directories.len() >= 5 {
                break;
            }
        }
        
        // 如果没有找到项目根目录，使用当前目录
        if directories.is_empty() {
            directories.push(env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        }
        
        Self { directories }
    }

    /// 检查目录是否为项目根目录
    /// 
    /// # Arguments
    /// * `path` - 要检查的路径
    fn is_project_root(path: &PathBuf) -> bool {
        let project_indicators = [
            "Cargo.toml",      // Rust项目
            "package.json",    // Node.js项目
            "pyproject.toml",  // Python项目
            ".git",           // Git仓库
            "Makefile",       // Make项目
            "CMakeLists.txt", // CMake项目
            "go.mod",         // Go项目
            "pom.xml",        // Maven项目
            "build.gradle",   // Gradle项目
        ];

        project_indicators.iter().any(|indicator| {
            path.join(indicator).exists()
        })
    }

    /// 添加自定义目录
    /// 
    /// # Arguments
    /// * `directory` - 要添加的目录路径
    pub fn add_directory(&mut self, directory: PathBuf) {
        if !self.directories.contains(&directory) {
            self.directories.push(directory);
        }
    }

    /// 移除目录
    /// 
    /// # Arguments
    /// * `directory` - 要移除的目录路径
    pub fn remove_directory(&mut self, directory: &PathBuf) {
        self.directories.retain(|d| d != directory);
    }

    /// 刷新目录列表
    /// 重新检测项目根目录
    pub fn refresh(&mut self) {
        *self = Self::new();
    }
}

impl Default for SmartWorkspaceContext {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceContext for SmartWorkspaceContext {
    fn get_directories(&self) -> Vec<PathBuf> {
        self.directories.clone()
    }
}

/// 工作区上下文构建器
#[derive(Debug)]
pub struct WorkspaceContextBuilder {
    directories: Vec<PathBuf>,
    use_smart_detection: bool,
}

impl WorkspaceContextBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            directories: Vec::new(),
            use_smart_detection: false,
        }
    }

    /// 添加目录
    /// 
    /// # Arguments
    /// * `directory` - 要添加的目录路径
    pub fn add_directory(mut self, directory: PathBuf) -> Self {
        self.directories.push(directory);
        self
    }

    /// 添加多个目录
    /// 
    /// # Arguments
    /// * `directories` - 要添加的目录路径列表
    pub fn add_directories(mut self, directories: Vec<PathBuf>) -> Self {
        self.directories.extend(directories);
        self
    }

    /// 启用智能检测
    pub fn with_smart_detection(mut self) -> Self {
        self.use_smart_detection = true;
        self
    }

    /// 构建工作区上下文
    pub fn build(self) -> Box<dyn WorkspaceContext + Send + Sync> {
        if self.use_smart_detection {
            let mut smart_context = SmartWorkspaceContext::new();
            for directory in self.directories {
                smart_context.add_directory(directory);
            }
            Box::new(smart_context)
        } else if self.directories.is_empty() {
            Box::new(BasicWorkspaceContext::new())
        } else {
            Box::new(CustomWorkspaceContext::new(self.directories))
        }
    }
}

impl Default for WorkspaceContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 工作区上下文工厂
#[derive(Debug)]
pub struct WorkspaceContextFactory;

impl WorkspaceContextFactory {
    /// 创建基础工作区上下文
    pub fn create_basic() -> Box<dyn WorkspaceContext + Send + Sync> {
        Box::new(BasicWorkspaceContext::new())
    }

    /// 创建自定义工作区上下文
    /// 
    /// # Arguments
    /// * `directories` - 目录列表
    pub fn create_custom(directories: Vec<PathBuf>) -> Box<dyn WorkspaceContext + Send + Sync> {
        Box::new(CustomWorkspaceContext::new(directories))
    }

    /// 创建智能工作区上下文
    pub fn create_smart() -> Box<dyn WorkspaceContext + Send + Sync> {
        Box::new(SmartWorkspaceContext::new())
    }

    /// 从当前工作目录创建工作区上下文
    pub fn from_current_dir() -> Box<dyn WorkspaceContext + Send + Sync> {
        let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Box::new(CustomWorkspaceContext::new(vec![current_dir]))
    }

    /// 从环境变量创建工作区上下文
    /// 检查ALOU_WORKSPACE_DIRS环境变量
    pub fn from_env() -> Box<dyn WorkspaceContext + Send + Sync> {
        if let Ok(workspace_dirs) = env::var("ALOU_WORKSPACE_DIRS") {
            let directories: Vec<PathBuf> = workspace_dirs
                .split(',')
                .map(|s| PathBuf::from(s.trim()))
                .filter(|p| p.exists())
                .collect();
            
            if !directories.is_empty() {
                return Box::new(CustomWorkspaceContext::new(directories));
            }
        }
        
        // 回退到智能检测
        Self::create_smart()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_basic_workspace_context() {
        let context = BasicWorkspaceContext::new();
        let directories = context.get_directories();
        assert!(!directories.is_empty());
    }

    #[test]
    fn test_custom_workspace_context() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();
        
        let context = CustomWorkspaceContext::new(vec![path.clone()]);
        let directories = context.get_directories();
        assert_eq!(directories.len(), 1);
        assert_eq!(directories[0], path);
    }

    #[test]
    fn test_smart_workspace_context() {
        let context = SmartWorkspaceContext::new();
        let directories = context.get_directories();
        assert!(!directories.is_empty());
    }

    #[test]
    fn test_workspace_context_builder() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();
        
        let context = WorkspaceContextBuilder::new()
            .add_directory(path.clone())
            .build();
        
        let directories = context.get_directories();
        assert_eq!(directories.len(), 1);
        assert_eq!(directories[0], path);
    }

    #[test]
    fn test_workspace_context_factory() {
        let basic_context = WorkspaceContextFactory::create_basic();
        assert!(!basic_context.get_directories().is_empty());
        
        let smart_context = WorkspaceContextFactory::create_smart();
        assert!(!smart_context.get_directories().is_empty());
    }
}

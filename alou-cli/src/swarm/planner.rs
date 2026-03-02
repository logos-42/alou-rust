//! 任务规划器 - 基于人月神话的任务分解与规划
//! 
//! 核心设计原则:
//! - 概念完整性: 统一的规划策略
//! - 渐进交付: 支持增量执行计划
//! - 减少沟通: 明确的任务边界

use std::collections::{HashMap, HashSet, VecDeque};
use crate::swarm::types::*;

/// 规划器配置
#[derive(Debug, Clone)]
pub struct PlannerConfig {
    /// 最大任务深度
    pub max_depth: u32,
    /// 默认超时时间
    pub default_timeout_ms: u64,
    /// 启用任务拆分
    pub enable_splitting: bool,
    /// 启用并行化
    pub enable_parallelization: bool,
    /// 启用依赖分析
    pub enable_dependency_analysis: bool,
}

impl Default for PlannerConfig {
    fn default() -> Self {
        Self {
            max_depth: 5,
            default_timeout_ms: 300_000,
            enable_splitting: true,
            enable_parallelization: true,
            enable_dependency_analysis: true,
        }
    }
}

/// 任务规划器
pub struct TaskPlanner {
    config: PlannerConfig,
}

/// 执行计划
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    /// 计划 ID
    pub id: String,
    /// 目标描述
    pub objective: String,
    /// 任务列表
    pub tasks: Vec<PlannedTask>,
    /// 执行策略
    pub strategy: ExecutionStrategy,
    /// 依赖图
    pub dependency_graph: DependencyGraph,
    /// 预估总时长
    pub estimated_duration_ms: i64,
}

/// 计划任务
#[derive(Debug, Clone)]
pub struct PlannedTask {
    /// 任务 ID
    pub id: String,
    /// 任务定义
    pub task: Task,
    /// 所属阶段
    pub stage: u32,
    /// 是否可并行
    pub parallelizable: bool,
    /// 预估时长
    pub estimated_duration_ms: i64,
}

/// 执行策略
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExecutionStrategy {
    /// 顺序执行
    Sequential,
    /// 并行执行
    Parallel,
    /// 流水线
    Pipeline,
    /// Map-Reduce
    MapReduce,
    /// Swarm 协作
    Swarm,
}

/// 任务模板
#[derive(Debug, Clone)]
pub struct TaskTemplate {
    /// 模板名称
    pub name: String,
    /// 任务类型
    pub task_type: String,
    /// 所需能力
    pub required_capabilities: Vec<String>,
    /// 默认步骤
    pub default_steps: Vec<TaskStep>,
    /// 超时时间
    pub timeout_ms: u64,
}

impl TaskPlanner {
    /// 创建新的规划器
    pub fn new(config: PlannerConfig) -> Self {
        Self { config }
    }
    
    /// 使用默认配置创建
    pub fn default() -> Self {
        Self::new(PlannerConfig::default())
    }
    
    /// 分析目标并生成执行计划
    pub fn plan(&self, objective: &str, context: &PlanningContext) -> Result<ExecutionPlan, String> {
        let plan_id = format!("plan-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap());
        
        // 1. 分析目标并分解为任务
        let tasks = self.decompose_objective(objective, context)?;
        
        // 2. 分析依赖关系
        let dependency_graph = if self.config.enable_dependency_analysis {
            self.analyze_dependencies(&tasks)?
        } else {
            DependencyGraph {
                nodes: tasks.iter().map(|t| t.id.clone()).collect(),
                edges: Vec::new(),
                critical_path: Vec::new(),
                parallel_groups: vec![tasks.iter().map(|t| t.id.clone()).collect()],
                estimated_duration: 0,
            }
        };
        
        // 3. 确定执行策略
        let strategy = self.determine_strategy(&tasks, &dependency_graph);
        
        // 4. 计算预估时长
        let estimated_duration = self.estimate_total_duration(
            &dependency_graph.nodes,
            &dependency_graph.edges,
            &tasks
        );
        
        // 5. 包装为 PlannedTask
        let planned_tasks: Vec<PlannedTask> = tasks.into_iter()
            .map(|task| {
                let parallelizable = self.is_parallelizable(&task);
                let stage = self.calculate_stage(&task, &dependency_graph);
                let estimated_duration_ms = self.estimate_task_duration(&task);
                
                PlannedTask {
                    id: task.id.clone(),
                    task,
                    stage,
                    parallelizable,
                    estimated_duration_ms,
                }
            })
            .collect();
        
        Ok(ExecutionPlan {
            id: plan_id,
            objective: objective.to_string(),
            tasks: planned_tasks,
            strategy,
            dependency_graph,
            estimated_duration_ms: estimated_duration,
        })
    }
    
    /// 分解目标为任务
    fn decompose_objective(&self, objective: &str, context: &PlanningContext) -> Result<Vec<Task>, String> {
        let mut tasks = Vec::new();
        let now = chrono::Utc::now().timestamp_millis();
        
        // 基于关键词分析目标类型
        let objective_lower = objective.to_lowercase();
        
        if objective_lower.contains("analyze") || objective_lower.contains("分析") {
            // 分析类目标
            tasks.push(self.create_analysis_task(objective, context, now)?);
        } else if objective_lower.contains("refactor") || objective_lower.contains("重构") {
            // 重构类目标 - 需要多个步骤
            tasks.extend(self.create_refactor_tasks(objective, context, now)?);
        } else if objective_lower.contains("generate") || objective_lower.contains("生成") {
            // 生成类目标
            tasks.push(self.create_generation_task(objective, context, now)?);
        } else if objective_lower.contains("test") || objective_lower.contains("测试") {
            // 测试类目标
            tasks.extend(self.create_test_tasks(objective, context, now)?);
        } else {
            // 通用目标
            tasks.push(self.create_generic_task(objective, context, now)?);
        }
        
        Ok(tasks)
    }
    
    /// 创建分析任务
    fn create_analysis_task(&self, objective: &str, _context: &PlanningContext, timestamp: i64) -> Result<Task, String> {
        let task_id = format!("task-analyze-{}", timestamp);
        
        let steps = vec![
            TaskStep {
                id: format!("{}-step-1", task_id),
                name: "收集信息".to_string(),
                description: Some("收集目标相关信息".to_string()),
                action: "file_read".to_string(),
                parameters: None,
                depends_on: None,
                condition: None,
                parallel: false,
                optional: false,
                retry_count: Some(1),
                status: None,
                result: None,
                error: None,
                started_at: None,
                completed_at: None,
            },
            TaskStep {
                id: format!("{}-step-2", task_id),
                name: "执行分析".to_string(),
                description: Some("分析收集到的信息".to_string()),
                action: "skill".to_string(),
                parameters: Some({
                    let mut params = HashMap::new();
                    params.insert("skill".to_string(), serde_json::json!("code-analyzer"));
                    params
                }),
                depends_on: Some(vec![format!("{}-step-1", task_id)]),
                condition: None,
                parallel: false,
                optional: false,
                retry_count: Some(2),
                status: None,
                result: None,
                error: None,
                started_at: None,
                completed_at: None,
            },
            TaskStep {
                id: format!("{}-step-3", task_id),
                name: "生成报告".to_string(),
                description: Some("生成分析报告".to_string()),
                action: "skill".to_string(),
                parameters: Some({
                    let mut params = HashMap::new();
                    params.insert("skill".to_string(), serde_json::json!("report-generator"));
                    params
                }),
                depends_on: Some(vec![format!("{}-step-2", task_id)]),
                condition: None,
                parallel: false,
                optional: true,
                retry_count: Some(1),
                status: None,
                result: None,
                error: None,
                started_at: None,
                completed_at: None,
            },
        ];
        
        Ok(Task {
            id: task_id,
            title: objective.to_string(),
            description: Some("分析任务".to_string()),
            priority: TaskPriority::High,
            status: TaskStatus::Pending,
            execution_mode: ExecutionMode::Sequential,
            max_agents: Some(1),
            timeout: Some(self.config.default_timeout_ms),
            retry_count: Some(2),
            retry_delay: Some(1000),
            steps,
            dependencies: None,
            sub_tasks: None,
            assignee_id: None,
            assignee_ids: None,
            required_capabilities: Some(vec![
                "analysis".to_string(),
                "code_reading".to_string(),
            ]),
            context: None,
            parameters: Some({
                let mut params = HashMap::new();
                params.insert("objective".to_string(), serde_json::json!(objective));
                params
            }),
            result: None,
            error: None,
            created_at: timestamp,
            updated_at: timestamp,
            assigned_at: None,
            started_at: None,
            completed_at: None,
            metadata: Some(TaskMetadata {
                source: Some("planner".to_string()),
                tags: Some(vec!["analysis".to_string()]),
                group: None,
                parent_task_id: None,
                related_tasks: None,
                created_by: Some("task_planner".to_string()),
                notes: None,
            }),
        })
    }
    
    /// 创建重构任务组
    fn create_refactor_tasks(&self, objective: &str, _context: &PlanningContext, timestamp: i64) -> Result<Vec<Task>, String> {
        let mut tasks = Vec::new();
        
        // 任务 1: 分析现有代码
        let analyze_task = Task {
            id: format!("task-refactor-analyze-{}", timestamp),
            title: "分析重构目标".to_string(),
            description: Some("分析需要重构的代码".to_string()),
            priority: TaskPriority::High,
            status: TaskStatus::Pending,
            execution_mode: ExecutionMode::Sequential,
            max_agents: Some(1),
            timeout: Some(300_000),
            retry_count: Some(1),
            retry_delay: Some(1000),
            steps: vec![TaskStep {
                id: format!("task-refactor-analyze-{}", timestamp),
                name: "代码分析".to_string(),
                description: None,
                action: "skill".to_string(),
                parameters: Some({
                    let mut params = HashMap::new();
                    params.insert("skill".to_string(), serde_json::json!("code-analyzer"));
                    params
                }),
                depends_on: None,
                condition: None,
                parallel: false,
                optional: false,
                retry_count: None,
                status: None,
                result: None,
                error: None,
                started_at: None,
                completed_at: None,
            }],
            dependencies: None,
            sub_tasks: None,
            assignee_id: None,
            assignee_ids: None,
            required_capabilities: Some(vec!["code_analysis".to_string()]),
            context: None,
            parameters: None,
            result: None,
            error: None,
            created_at: timestamp,
            updated_at: timestamp,
            assigned_at: None,
            started_at: None,
            completed_at: None,
            metadata: None,
        };
        tasks.push(analyze_task);
        
        // 任务 2: 设计重构方案（依赖任务 1）
        let design_task = Task {
            id: format!("task-refactor-design-{}", timestamp),
            title: "设计重构方案".to_string(),
            description: Some("基于分析结果设计重构方案".to_string()),
            priority: TaskPriority::High,
            status: TaskStatus::Pending,
            execution_mode: ExecutionMode::Sequential,
            max_agents: Some(1),
            timeout: Some(600_000),
            retry_count: Some(1),
            retry_delay: Some(1000),
            steps: vec![TaskStep {
                id: format!("task-refactor-design-{}", timestamp),
                name: "方案设计".to_string(),
                description: None,
                action: "skill".to_string(),
                parameters: Some({
                    let mut params = HashMap::new();
                    params.insert("skill".to_string(), serde_json::json!("architecture-designer"));
                    params
                }),
                depends_on: None,
                condition: None,
                parallel: false,
                optional: false,
                retry_count: None,
                status: None,
                result: None,
                error: None,
                started_at: None,
                completed_at: None,
            }],
            dependencies: Some(vec![format!("task-refactor-analyze-{}", timestamp)]),
            sub_tasks: None,
            assignee_id: None,
            assignee_ids: None,
            required_capabilities: Some(vec!["architecture".to_string(), "design".to_string()]),
            context: None,
            parameters: None,
            result: None,
            error: None,
            created_at: timestamp,
            updated_at: timestamp,
            assigned_at: None,
            started_at: None,
            completed_at: None,
            metadata: None,
        };
        tasks.push(design_task);
        
        // 任务 3: 执行重构（依赖任务 2）
        let implement_task = Task {
            id: format!("task-refactor-implement-{}", timestamp),
            title: "执行重构".to_string(),
            description: Some("按照设计方案执行重构".to_string()),
            priority: TaskPriority::Critical,
            status: TaskStatus::Pending,
            execution_mode: ExecutionMode::Sequential,
            max_agents: Some(1),
            timeout: Some(1_800_000),
            retry_count: Some(0),
            retry_delay: None,
            steps: vec![
                TaskStep {
                    id: format!("{}-backup", timestamp),
                    name: "备份原代码".to_string(),
                    description: None,
                    action: "bash".to_string(),
                    parameters: Some({
                        let mut params = HashMap::new();
                        params.insert("command".to_string(), serde_json::json!("git stash"));
                        params
                    }),
                    depends_on: None,
                    condition: None,
                    parallel: false,
                    optional: false,
                    retry_count: None,
                    status: None,
                    result: None,
                    error: None,
                    started_at: None,
                    completed_at: None,
                },
                TaskStep {
                    id: format!("{}-refactor", timestamp),
                    name: "执行重构".to_string(),
                    description: None,
                    action: "skill".to_string(),
                    parameters: Some({
                        let mut params = HashMap::new();
                        params.insert("skill".to_string(), serde_json::json!("code-refactor"));
                        params
                    }),
                    depends_on: Some(vec![format!("{}-backup", timestamp)]),
                    condition: None,
                    parallel: false,
                    optional: false,
                    retry_count: None,
                    status: None,
                    result: None,
                    error: None,
                    started_at: None,
                    completed_at: None,
                },
            ],
            dependencies: Some(vec![format!("task-refactor-design-{}", timestamp)]),
            sub_tasks: None,
            assignee_id: None,
            assignee_ids: None,
            required_capabilities: Some(vec!["code_refactoring".to_string()]),
            context: None,
            parameters: Some({
                let mut params = HashMap::new();
                params.insert("objective".to_string(), serde_json::json!(objective));
                params
            }),
            result: None,
            error: None,
            created_at: timestamp,
            updated_at: timestamp,
            assigned_at: None,
            started_at: None,
            completed_at: None,
            metadata: None,
        };
        tasks.push(implement_task);
        
        // 任务 4: 验证重构结果（依赖任务 3）
        let verify_task = Task {
            id: format!("task-refactor-verify-{}", timestamp),
            title: "验证重构结果".to_string(),
            description: Some("验证重构后的代码".to_string()),
            priority: TaskPriority::High,
            status: TaskStatus::Pending,
            execution_mode: ExecutionMode::Sequential,
            max_agents: Some(1),
            timeout: Some(300_000),
            retry_count: Some(1),
            retry_delay: Some(1000),
            steps: vec![
                TaskStep {
                    id: format!("{}-compile", timestamp),
                    name: "编译检查".to_string(),
                    description: None,
                    action: "bash".to_string(),
                    parameters: Some({
                        let mut params = HashMap::new();
                        params.insert("command".to_string(), serde_json::json!("cargo check"));
                        params
                    }),
                    depends_on: None,
                    condition: None,
                    parallel: false,
                    optional: false,
                    retry_count: None,
                    status: None,
                    result: None,
                    error: None,
                    started_at: None,
                    completed_at: None,
                },
                TaskStep {
                    id: format!("{}-test", timestamp),
                    name: "运行测试".to_string(),
                    description: None,
                    action: "bash".to_string(),
                    parameters: Some({
                        let mut params = HashMap::new();
                        params.insert("command".to_string(), serde_json::json!("cargo test"));
                        params
                    }),
                    depends_on: Some(vec![format!("{}-compile", timestamp)]),
                    condition: None,
                    parallel: false,
                    optional: false,
                    retry_count: None,
                    status: None,
                    result: None,
                    error: None,
                    started_at: None,
                    completed_at: None,
                },
            ],
            dependencies: Some(vec![format!("task-refactor-implement-{}", timestamp)]),
            sub_tasks: None,
            assignee_id: None,
            assignee_ids: None,
            required_capabilities: Some(vec!["testing".to_string()]),
            context: None,
            parameters: None,
            result: None,
            error: None,
            created_at: timestamp,
            updated_at: timestamp,
            assigned_at: None,
            started_at: None,
            completed_at: None,
            metadata: None,
        };
        tasks.push(verify_task);
        
        Ok(tasks)
    }
    
    /// 创建生成任务
    fn create_generation_task(&self, objective: &str, _context: &PlanningContext, timestamp: i64) -> Result<Task, String> {
        Ok(Task {
            id: format!("task-generate-{}", timestamp),
            title: objective.to_string(),
            description: Some("代码生成任务".to_string()),
            priority: TaskPriority::Medium,
            status: TaskStatus::Pending,
            execution_mode: ExecutionMode::Sequential,
            max_agents: Some(1),
            timeout: Some(600_000),
            retry_count: Some(2),
            retry_delay: Some(2000),
            steps: vec![TaskStep {
                id: format!("task-generate-{}", timestamp),
                name: "生成代码".to_string(),
                description: None,
                action: "skill".to_string(),
                parameters: Some({
                    let mut params = HashMap::new();
                    params.insert("skill".to_string(), serde_json::json!("code-generator"));
                    params
                }),
                depends_on: None,
                condition: None,
                parallel: false,
                optional: false,
                retry_count: None,
                status: None,
                result: None,
                error: None,
                started_at: None,
                completed_at: None,
            }],
            dependencies: None,
            sub_tasks: None,
            assignee_id: None,
            assignee_ids: None,
            required_capabilities: Some(vec!["code_generation".to_string()]),
            context: None,
            parameters: None,
            result: None,
            error: None,
            created_at: timestamp,
            updated_at: timestamp,
            assigned_at: None,
            started_at: None,
            completed_at: None,
            metadata: None,
        })
    }
    
    /// 创建测试任务组
    fn create_test_tasks(&self, objective: &str, _context: &PlanningContext, timestamp: i64) -> Result<Vec<Task>, String> {
        let mut tasks = Vec::new();
        
        // 并行测试任务 1: 单元测试
        tasks.push(Task {
            id: format!("task-test-unit-{}", timestamp),
            title: "运行单元测试".to_string(),
            description: Some("执行单元测试套件".to_string()),
            priority: TaskPriority::High,
            status: TaskStatus::Pending,
            execution_mode: ExecutionMode::Parallel,
            max_agents: Some(2),
            timeout: Some(300_000),
            retry_count: Some(1),
            retry_delay: Some(1000),
            steps: vec![TaskStep {
                id: format!("task-test-unit-{}", timestamp),
                name: "单元测试".to_string(),
                description: None,
                action: "bash".to_string(),
                parameters: Some({
                    let mut params = HashMap::new();
                    params.insert("command".to_string(), serde_json::json!("cargo test --lib"));
                    params
                }),
                depends_on: None,
                condition: None,
                parallel: true,
                optional: false,
                retry_count: None,
                status: None,
                result: None,
                error: None,
                started_at: None,
                completed_at: None,
            }],
            dependencies: None,
            sub_tasks: None,
            assignee_id: None,
            assignee_ids: None,
            required_capabilities: Some(vec!["testing".to_string()]),
            context: None,
            parameters: Some({
                let mut params = HashMap::new();
                params.insert("objective".to_string(), serde_json::json!(objective));
                params
            }),
            result: None,
            error: None,
            created_at: timestamp,
            updated_at: timestamp,
            assigned_at: None,
            started_at: None,
            completed_at: None,
            metadata: None,
        });
        
        // 并行测试任务 2: 集成测试
        tasks.push(Task {
            id: format!("task-test-integration-{}", timestamp),
            title: "运行集成测试".to_string(),
            description: Some("执行集成测试套件".to_string()),
            priority: TaskPriority::High,
            status: TaskStatus::Pending,
            execution_mode: ExecutionMode::Parallel,
            max_agents: Some(2),
            timeout: Some(600_000),
            retry_count: Some(1),
            retry_delay: Some(1000),
            steps: vec![TaskStep {
                id: format!("task-test-integration-{}", timestamp),
                name: "集成测试".to_string(),
                description: None,
                action: "bash".to_string(),
                parameters: Some({
                    let mut params = HashMap::new();
                    params.insert("command".to_string(), serde_json::json!("cargo test --test '*'"));
                    params
                }),
                depends_on: None,
                condition: None,
                parallel: true,
                optional: false,
                retry_count: None,
                status: None,
                result: None,
                error: None,
                started_at: None,
                completed_at: None,
            }],
            dependencies: None,
            sub_tasks: None,
            assignee_id: None,
            assignee_ids: None,
            required_capabilities: Some(vec!["testing".to_string()]),
            context: None,
            parameters: None,
            result: None,
            error: None,
            created_at: timestamp,
            updated_at: timestamp,
            assigned_at: None,
            started_at: None,
            completed_at: None,
            metadata: None,
        });
        
        Ok(tasks)
    }
    
    /// 创建通用任务
    fn create_generic_task(&self, objective: &str, _context: &PlanningContext, timestamp: i64) -> Result<Task, String> {
        Ok(Task {
            id: format!("task-generic-{}", timestamp),
            title: objective.to_string(),
            description: None,
            priority: TaskPriority::Medium,
            status: TaskStatus::Pending,
            execution_mode: ExecutionMode::Sequential,
            max_agents: Some(1),
            timeout: Some(self.config.default_timeout_ms),
            retry_count: Some(1),
            retry_delay: Some(1000),
            steps: vec![TaskStep {
                id: format!("task-generic-{}", timestamp),
                name: "执行任务".to_string(),
                description: None,
                action: "skill".to_string(),
                parameters: None,
                depends_on: None,
                condition: None,
                parallel: false,
                optional: false,
                retry_count: None,
                status: None,
                result: None,
                error: None,
                started_at: None,
                completed_at: None,
            }],
            dependencies: None,
            sub_tasks: None,
            assignee_id: None,
            assignee_ids: None,
            required_capabilities: None,
            context: None,
            parameters: Some({
                let mut params = HashMap::new();
                params.insert("objective".to_string(), serde_json::json!(objective));
                params
            }),
            result: None,
            error: None,
            created_at: timestamp,
            updated_at: timestamp,
            assigned_at: None,
            started_at: None,
            completed_at: None,
            metadata: None,
        })
    }
    
    /// 分析依赖关系
    fn analyze_dependencies(&self, tasks: &[Task]) -> Result<DependencyGraph, String> {
        let mut edges = Vec::new();
        let nodes: Vec<String> = tasks.iter().map(|t| t.id.clone()).collect();
        
        // 从任务依赖构建边
        for task in tasks {
            if let Some(deps) = &task.dependencies {
                for dep_id in deps {
                    edges.push(DependencyEdge {
                        from: dep_id.clone(),
                        to: task.id.clone(),
                        edge_type: "required".to_string(),
                    });
                }
            }
        }
        
        // 计算关键路径（简化实现）
        let critical_path = self.calculate_critical_path(&nodes, &edges);
        
        // 识别可并行执行的组
        let parallel_groups = self.identify_parallel_groups(&nodes, &edges);
        
        // 预估总时长
        let estimated_duration = self.estimate_total_duration(&nodes, &edges, tasks);
        
        Ok(DependencyGraph {
            nodes,
            edges,
            critical_path,
            parallel_groups,
            estimated_duration,
        })
    }
    
    /// 计算关键路径
    fn calculate_critical_path(&self, nodes: &[String], edges: &[DependencyEdge]) -> Vec<String> {
        // 简化实现：按依赖顺序排列
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        
        for node in nodes {
            in_degree.insert(node.clone(), 0);
            adj.insert(node.clone(), Vec::new());
        }
        
        for edge in edges {
            *in_degree.entry(edge.to.clone()).or_insert(0) += 1;
            adj.entry(edge.from.clone()).or_default().push(edge.to.clone());
        }
        
        let mut queue: VecDeque<String> = in_degree.iter()
            .filter(|(_, &d)| d == 0)
            .map(|(n, _)| n.clone())
            .collect();
        
        let mut result = Vec::new();
        
        while let Some(node) = queue.pop_front() {
            result.push(node.clone());
            
            if let Some(neighbors) = adj.get(&node) {
                for neighbor in neighbors {
                    let degree = in_degree.get_mut(neighbor).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }
        
        result
    }
    
    /// 识别可并行执行的组
    fn identify_parallel_groups(&self, nodes: &[String], edges: &[DependencyEdge]) -> Vec<Vec<String>> {
        // 简化实现：将所有无依赖的任务分到一组
        let has_incoming: HashSet<String> = edges.iter().map(|e| e.to.clone()).collect();
        
        let parallel_tasks: Vec<String> = nodes.iter()
            .filter(|n| !has_incoming.contains(*n))
            .cloned()
            .collect();
        
        if parallel_tasks.is_empty() {
            vec![nodes.to_vec()]
        } else {
            vec![parallel_tasks]
        }
    }
    
    /// 确定执行策略
    fn determine_strategy(&self, tasks: &[Task], _graph: &DependencyGraph) -> ExecutionStrategy {
        if tasks.len() == 1 {
            return ExecutionStrategy::Sequential;
        }
        
        let has_dependencies = tasks.iter().any(|t| t.dependencies.is_some());
        let all_parallelizable = tasks.iter().all(|t| {
            t.execution_mode == ExecutionMode::Parallel || 
            t.execution_mode == ExecutionMode::Swarm
        });
        
        if !has_dependencies && all_parallelizable {
            ExecutionStrategy::Parallel
        } else if has_dependencies {
            ExecutionStrategy::Pipeline
        } else {
            ExecutionStrategy::Sequential
        }
    }
    
    /// 计算任务阶段
    fn calculate_stage(&self, task: &Task, graph: &DependencyGraph) -> u32 {
        if let Some(deps) = &task.dependencies {
            if deps.is_empty() {
                0
            } else {
                // 找到依赖的最大阶段 + 1
                1
            }
        } else {
            0
        }
    }
    
    /// 检查是否可并行
    fn is_parallelizable(&self, task: &Task) -> bool {
        task.execution_mode == ExecutionMode::Parallel ||
        task.execution_mode == ExecutionMode::Swarm
    }
    
    /// 预估任务时长
    fn estimate_task_duration(&self, task: &Task) -> i64 {
        // 基于步骤数量和复杂度预估
        let base_duration = 1000i64; // 1秒基础时间
        let step_duration = 5000i64; // 每步5秒
        
        base_duration + (task.steps.len() as i64 * step_duration)
    }
    
    /// 预估总时长
    fn estimate_total_duration(&self, nodes: &[String], edges: &[DependencyEdge], tasks: &[Task]) -> i64 {
        let task_map: HashMap<String, &Task> = tasks.iter().map(|t| (t.id.clone(), t)).collect();
        
        // 简化实现：累加所有任务时长
        nodes.iter()
            .filter_map(|id| task_map.get(id))
            .map(|t| self.estimate_task_duration(t))
            .sum()
    }
}

/// 规划上下文
#[derive(Debug, Clone, Default)]
pub struct PlanningContext {
    /// 可用 Agent 能力
    pub available_capabilities: Vec<String>,
    /// 历史执行数据
    pub historical_data: Option<HistoricalData>,
    /// 约束条件
    pub constraints: Vec<Constraint>,
}

/// 历史数据
#[derive(Debug, Clone)]
pub struct HistoricalData {
    pub avg_task_duration: i64,
    pub success_rate: f32,
    pub common_patterns: Vec<String>,
}

/// 约束条件
#[derive(Debug, Clone)]
pub struct Constraint {
    pub constraint_type: String,
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_planner_creation() {
        let planner = TaskPlanner::default();
        assert!(planner.config.enable_parallelization);
    }
    
    #[test]
    fn test_plan_analysis() {
        let planner = TaskPlanner::default();
        let context = PlanningContext::default();
        
        let plan = planner.plan("analyze the codebase", &context).unwrap();
        assert!(!plan.tasks.is_empty());
        assert_eq!(plan.objective, "analyze the codebase");
    }
    
    #[test]
    fn test_plan_refactor() {
        let planner = TaskPlanner::default();
        let context = PlanningContext::default();
        
        let plan = planner.plan("refactor user authentication", &context).unwrap();
        assert!(plan.tasks.len() >= 3); // 分析、设计、执行、验证
        assert!(matches!(plan.strategy, ExecutionStrategy::Pipeline));
    }
}
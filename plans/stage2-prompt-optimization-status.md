# 阶段 2 实施状态：Prompt 优化 + 反思层

**实施日期**: 2026-03-17
**状态**: 🟡 进行中 (70% 完成)

---

## ✅ 已完成

### 1. 反思层数据结构

**文件**: `alou-desktop/src-tauri/src/agent/executor.rs`

```rust
/// 推理结果（Reason 层输出 - LLM 单次调用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thought {
    /// 分析过程
    pub analysis: String,
    /// 🔥 新增：强制反思
    pub reflection: Option<Reflection>,
    /// 下一步行动
    pub action: Action,
}

/// 🔥 反思层结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reflection {
    /// 信息充分性评估
    pub information_assessment: InformationAssessment,
    /// 任务进度评估
    pub task_progress: TaskProgress,
    /// 决策置信度 (0.0-1.0)
    pub confidence: f32,
    /// 决策理由
    pub reasoning: String,
    /// 建议的下一步
    pub suggested_next_step: String,
}

/// 信息充分性评估
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InformationAssessment {
    /// 状态："sufficient" | "insufficient" | "uncertain"
    pub status: String,
    /// 缺少的信息
    #[serde(default)]
    pub missing_info: Vec<String>,
    /// 建议："proceed" | "gather_more" | "ask_user"
    pub suggestion: String,
}

/// 任务进度评估
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgress {
    /// 整体进度 (0.0-1.0)
    pub overall_progress: f32,
    /// 已完成的步骤
    #[serde(default)]
    pub completed_steps: Vec<String>,
    /// 待处理的步骤
    #[serde(default)]
    pub pending_steps: Vec<String>,
    /// 预计剩余迭代次数
    pub estimated_remaining_iterations: u32,
}
```

**特点**:
- ✅ 所有结构都实现了 `Default` trait
- ✅ 使用 `Option<Reflection>` 保持向后兼容
- ✅ 使用 `#[serde(default)]` 处理缺失字段

---

## ⏭️ 待完成

### 1. 更新 `build_reasoning_prompt`

**目标**: 在 Prompt 中强制 AI 填写 reflection 字段

**待添加内容**:

```rust
## 回复格式
你必须以 JSON 格式回复，格式如下：

{
  "analysis": "你的分析过程",
  "reflection": {
    "information_assessment": {
      "status": "sufficient | insufficient | uncertain",
      "missing_info": [],
      "suggestion": "proceed | gather_more | ask_user"
    },
    "task_progress": {
      "overall_progress": 0.5,
      "completed_steps": [],
      "pending_steps": [],
      "estimated_remaining_iterations": 3
    },
    "confidence": 0.85,
    "reasoning": "我为什么选择这个 action",
    "suggested_next_step": "具体下一步建议"
  },
  "action": {
    "type": "tool_call | complete | continue | fail",
    ...
  }
}

## 决策前的强制反思

在做出决策之前，你**必须**完成以下反思：

### 1. 信息充分性评估
- 当前信息是否足够回答/执行任务？
- 如果不够，缺少什么信息？应该使用哪个工具获取？
- 建议：proceed（继续）/ gather_more（获取更多信息）/ ask_user（询问用户）

### 2. 任务进度评估
- 整体进度百分比？
- 已完成哪些步骤？
- 还有哪些待处理步骤？
- 预计还需要多少次迭代？

### 3. 决策置信度
- 对当前决策的置信度（0.0-1.0）
- 低于 0.7 时应该获取更多信心

### 4. Continue 使用规范
- 🚫 禁止：在没有任何 tool_call 的情况下单独使用 Continue
- ✅ 允许：执行 tool_call 后，等待结果时系统会自动继续
- ✅ 允许：需要主动等待用户输入时（如"请提供更多细节"）
```

**文件位置**: `executor.rs:build_reasoning_prompt()`

---

### 2. 更新 `parse_thought`

**目标**: 正确解析 AI 返回的 reflection 字段

**待添加逻辑**:

```rust
fn parse_thought(&self, json_str: &str) -> Result<Thought, ExecutorError> {
    // 解析 JSON
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| ExecutorError::AiError(format!("JSON 解析失败：{}", e)))?;
    
    // 提取 reflection 字段（可选）
    let reflection = value.get("reflection")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();  // 如果没有 reflection，使用默认值
    
    // 提取其他字段...
    let analysis = value["analysis"].as_str().unwrap_or("").to_string();
    let action = self.parse_action(&value["action"])?;
    
    Ok(Thought {
        analysis,
        reflection: Some(reflection),
        action,
    })
}
```

**文件位置**: `executor.rs:parse_thought()`

---

### 3. 实现基于反思的干预逻辑

**目标**: 在 `reason()` 方法中检查 reflection，必要时自动干预

**待添加逻辑**:

```rust
async fn reason(&self, state: &EnvironmentState) -> Result<Thought, ExecutorError> {
    let prompt = self.build_reasoning_prompt(state);
    let messages = vec![
        AiMessage::system("你是一个自主智能体..."),
        AiMessage::user(prompt),
    ];
    
    let response = self.ai_client.send_message(messages, None).await?;
    let mut thought = self.parse_thought(&response.content)?;
    
    // 🔥 新增：基于反思的干预
    if let Some(reflection) = &thought.reflection {
        // 干预 1: 信息不足但未调用工具
        if reflection.information_assessment.suggestion == "gather_more" 
            && !matches!(thought.action, Action::ToolCall { .. }) {
            log::warn!("[AgentReasoning] AI 建议获取更多信息但未调用工具，自动干预");
            
            // 自动改为调用 memory_search
            thought.action = Action::ToolCall {
                tool: "memory_search".to_string(),
                args: json!({
                    "query": reflection.information_assessment.missing_info.join(" ")
                }),
                id: format!("auto_{}", uuid::Uuid::new_v4()),
            };
        }
        
        // 干预 2: 置信度过低
        if reflection.confidence < 0.5 {
            log::warn!("[AgentReasoning] 置信度过低 ({:.2})，建议询问用户", reflection.confidence);
            // 可以在这里添加额外逻辑，如强制 ask_user
        }
        
        // 干预 3: 简单对话但选择 tool_call
        if self.is_simple_conversation(&state.messages) 
            && matches!(thought.action, Action::ToolCall { .. }) {
            log::warn!("[AgentReasoning] 简单对话但调用工具，建议改为 complete");
            // 可以记录警告或自动干预
        }
    }
    
    Ok(thought)
}
```

**文件位置**: `executor.rs:reason()`

---

### 4. 添加 Continue 使用验证

**目标**: 在 `act()` 方法中检查 Continue 的合法使用

**待添加逻辑**:

```rust
async fn act(&self, thought: &Thought, state: &EnvironmentState) -> Result<ActionResult, ExecutorError> {
    match &thought.action {
        Action::Continue => {
            // 🔥 新增：验证 Continue 的合法使用
            // 检查上一次迭代是否有 tool_call
            let has_pending_tool_calls = state.tool_call_count > 0;
            
            if !has_pending_tool_calls {
                log::warn!("[AgentReasoning] 检测到非法的 Continue 使用（无 pending tool_call）");
                // 自动干预：改为询问用户
                return Ok(ActionResult {
                    action_id: "auto_complete".to_string(),
                    tool_name: "".to_string(),
                    success: true,
                    output: Some(json!({
                        "content": "我注意到需要更多信息来完成此任务。请提供更多细节或明确指示。"
                    })),
                    error: None,
                });
            }
        }
        _ => {}
    }
    
    // 正常处理其他 action...
}
```

**文件位置**: `executor.rs:act()`

---

## 🧪 测试计划

### 单元测试

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_reflection_default() {
        let reflection = Reflection::default();
        assert_eq!(reflection.information_assessment.status, "uncertain");
        assert_eq!(reflection.confidence, 0.5);
    }
    
    #[test]
    fn test_thought_with_optional_reflection() {
        let thought = Thought {
            analysis: "test".to_string(),
            reflection: None,  // 向后兼容
            action: Action::Continue,
        };
        assert!(thought.reflection.is_none());
    }
}
```

### 集成测试

```rust
#[tokio::test]
async fn test_greeting_direct_complete() {
    // 设置 executor
    let executor = create_test_executor();
    
    // 创建简单问候任务
    let task_id = create_task("你好").await;
    let state = executor.perceive_environment(&task_id).await.unwrap();
    
    // 执行决策
    let thought = executor.reason(&state).await.unwrap();
    
    // 验证：应该直接 complete，不再 Continue
    assert!(matches!(thought.action, Action::Complete(_)));
    
    // 验证：reflection 应该显示信息充分
    if let Some(reflection) = &thought.reflection {
        assert_eq!(reflection.information_assessment.status, "sufficient");
        assert!(reflection.confidence > 0.9);
    }
}

#[tokio::test]
async fn test_low_confidence_intervention() {
    // 测试低置信度时的自动干预
    // ...
}
```

---

## 📊 进度追踪

| 任务 | 状态 | 预计时间 | 实际时间 |
|------|------|----------|----------|
| 扩展 Thought 结构 | ✅ 完成 | 30min | 25min |
| 更新 build_reasoning_prompt | ⏭️ 进行中 | 1h | - |
| 更新 parse_thought | ⏭️ 待开始 | 30min | - |
| 实现干预逻辑 | ⏭️ 待开始 | 1h | - |
| 添加 Continue 验证 | ⏭️ 待开始 | 30min | - |
| 编写单元测试 | ⏭️ 待开始 | 1h | - |
| 编写集成测试 | ⏭️ 待开始 | 1h | - |

**总进度**: 1/7 (14%)
**预计完成时间**: 5 小时

---

## 🔗 相关文件

- `alou-desktop/src-tauri/src/agent/executor.rs` - 核心实现
- `plans/autonomous-agent-enhancement-plan.md` - 总体实施计划
- `MEDIA_API_CONFIG_OPTIMIZATION.md` - API 配置优化文档

---

**最后更新**: 2026-03-17
**下次更新**: 完成 prompt 更新后

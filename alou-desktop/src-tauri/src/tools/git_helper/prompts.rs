//! Git助手工具 - 提示词生成模块

use super::types::GitPrompt;

/// 根据场景生成Git提示词
pub fn generate_prompt(scenario: &str) -> GitPrompt {
    match scenario {
        "before_commit" => GitPrompt {
            scenario: "before_commit".to_string(),
            description: "准备提交代码前的检查清单".to_string(),
            suggested_commands: vec![
                "git status".to_string(),
                "git diff --stat".to_string(),
                "git add -A".to_string(),
                "git commit -m 'your message'".to_string(),
            ],
            safety_checklist: vec![
                "检查是否包含敏感信息".to_string(),
                "确认测试通过".to_string(),
                "检查提交信息是否清晰".to_string(),
            ],
            best_practice: "使用有意义的提交信息，一个提交只做一件事".to_string(),
        },
        "before_push" => GitPrompt {
            scenario: "before_push".to_string(),
            description: "推送代码到远程前的准备".to_string(),
            suggested_commands: vec![
                "git fetch origin".to_string(),
                "git status".to_string(),
                "git log --oneline origin/main..HEAD".to_string(),
                "git push origin <branch>".to_string(),
            ],
            safety_checklist: vec![
                "拉取最新代码并解决冲突".to_string(),
                "检查推送的提交".to_string(),
                "确认分支正确".to_string(),
            ],
            best_practice: "先fetch查看差异，避免强制推送".to_string(),
        },
        "merge_conflict" => GitPrompt {
            scenario: "merge_conflict".to_string(),
            description: "解决合并冲突的步骤".to_string(),
            suggested_commands: vec![
                "git status".to_string(),
                "# 编辑冲突文件".to_string(),
                "git add <resolved-files>".to_string(),
                "git commit".to_string(),
            ],
            safety_checklist: vec![
                "理解双方变更的意图".to_string(),
                "测试冲突解决后的代码".to_string(),
                "不要只保留一方的代码".to_string(),
            ],
            best_practice: "与冲突代码的作者沟通，确保正确解决".to_string(),
        },
        "create_branch" => GitPrompt {
            scenario: "create_branch".to_string(),
            description: "创建新功能分支".to_string(),
            suggested_commands: vec![
                "git checkout main".to_string(),
                "git pull origin main".to_string(),
                "git checkout -b feature/name".to_string(),
                "git push -u origin feature/name".to_string(),
            ],
            safety_checklist: vec![
                "从最新主分支创建".to_string(),
                "使用规范的分支命名".to_string(),
            ],
            best_practice: "feature/、bugfix/、hotfix/ 前缀命名".to_string(),
        },
        "undo_changes" => GitPrompt {
            scenario: "undo_changes".to_string(),
            description: "撤销不同类型的变更".to_string(),
            suggested_commands: vec![
                "git restore <file>".to_string(),
                "git reset HEAD <file>".to_string(),
                "git checkout HEAD~1 -- <file>".to_string(),
                "git revert <commit>".to_string(),
            ],
            safety_checklist: vec![
                "确认要撤销的内容".to_string(),
                "重要变更先备份".to_string(),
            ],
            best_practice: "优先使用revert撤销已推送的提交".to_string(),
        },
        _ => GitPrompt {
            scenario: scenario.to_string(),
            description: "通用Git工作流提示".to_string(),
            suggested_commands: vec![
                "git status".to_string(),
                "git log --oneline -5".to_string(),
            ],
            safety_checklist: vec!["确认当前分支".to_string()],
            best_practice: "经常提交，保持提交原子性".to_string(),
        },
    }
}

/// 生成智能提交消息建议
pub fn suggest_commit_message(files: &[String]) -> Vec<String> {
    let mut suggestions = vec![];
    
    let has_code = files.iter().any(|f| 
        f.ends_with(".rs") || f.ends_with(".js") || f.ends_with(".ts") || 
        f.ends_with(".py") || f.ends_with(".java")
    );
    let has_config = files.iter().any(|f| 
        f.contains("config") || f.ends_with(".toml") || f.ends_with(".yaml") || f.ends_with(".json")
    );
    let has_docs = files.iter().any(|f| 
        f.ends_with(".md") || f.ends_with(".txt") || f.contains("doc")
    );
    let has_tests = files.iter().any(|f| 
        f.contains("test") || f.contains("spec")
    );

    if has_tests {
        suggestions.push("Add/update tests".to_string());
    }
    if has_docs {
        suggestions.push("Update documentation".to_string());
    }
    if has_config {
        suggestions.push("Update configuration".to_string());
    }
    if has_code {
        suggestions.push("Update implementation".to_string());
    }
    if files.len() == 1 {
        suggestions.push(format!("Update {}", files[0]));
    } else {
        suggestions.push(format!("Update {} files", files.len()));
    }

    suggestions
}

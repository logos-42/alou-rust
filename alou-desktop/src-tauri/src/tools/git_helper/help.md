Git Helper Tool - 智能Git操作助手

核心操作：

1. Execute - 执行原始Git命令（带安全检查）
   {
     "action": "execute",
     "subcommand": "status",
     "args": ["-s"],
     "working_dir": "/path/to/repo"
   }

2. SmartCommit - 智能提交（推荐用于AI自动提交）
   {
     "action": "smart_commit",
     "message": "Fix memory leak in parser",
     "add_all": true,
     "allow_empty": false
   }

3. CreateFeatureBranch - 创建功能分支
   {
     "action": "create_feature_branch",
     "branch_name": "feature/new-parser",
     "base_branch": "develop"
   }

4. SafeMerge - 安全合并（带冲突检测）
   {
     "action": "safe_merge",
     "source_branch": "feature/new-parser",
     "strategy": "merge"
   }

5. StatusCheck - 完整状态检查
   {
     "action": "status_check",
     "detailed": true
   }

6. DiffSummary - 变更摘要
   {
     "action": "diff_summary",
     "stat_only": true,
     "target": "HEAD~1"
   }

7. LogHistory - 查看提交历史
   {
     "action": "log_history",
     "count": 5,
     "format": "oneline"
   }

8. StashManagement - 暂存管理
   {
     "action": "stash_management",
     "operation": "save",
     "message": "WIP: refactoring"
   }

9. GetPrompt - 获取Git提示词建议
   {
     "action": "get_prompt",
     "scenario": "before_commit"
   }

10. BatchOperation - 批量提交（适合AI）
    {
      "action": "batch_operation",
      "operations": [
        {"files": ["a.rs"], "message": "Fix bug in a"},
        {"files": ["b.rs", "c.rs"], "message": "Update utils"}
      ]
    }

11. UndoOperation - 安全撤销
    {
      "action": "undo_operation",
      "undo_type": "unstage",
      "target": "file.txt"
    }

12. RemoteSync - 远程同步
    {
      "action": "remote_sync",
      "operation": "pull",
      "remote": "origin",
      "branch": "main"
    }

AI使用最佳实践：
- 修改前先用 status_check 检查状态
- 使用 smart_commit 自动处理提交流程
- 批量修改使用 batch_operation 分批提交
- 不确定时用 get_prompt 获取建议
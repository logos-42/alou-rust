Rollback Tool - 文件备份与回滚系统

核心功能：
1. CreateSnapshot - 为文件或目录创建快照
   {
     "action": "create_snapshot",
     "target_path": "/path/to/file",
     "name": "Before refactoring",
     "tags": ["important", "pre-change"]
   }

2. CreateBatchSnapshot - 批量创建多个路径的快照
   {
     "action": "create_batch_snapshot",
     "target_paths": ["/path/1", "/path/2"],
     "name_prefix": "Project backup"
   }

3. ListSnapshots - 列出所有快照
   {
     "action": "list_snapshots",
     "session_id": "optional-filter",
     "tags": ["important"]
   }

4. RestoreSnapshot - 恢复快照
   {
     "action": "restore_snapshot",
     "snapshot_id": "snap_xxx",
     "restore_path": "/optional/custom/path",
     "force": false
   }

5. CompareSnapshot - 比较快照与当前状态
   {
     "action": "compare_snapshot",
     "snapshot_id": "snap_xxx"
   }

6. DeleteSnapshot - 删除快照
   {
     "action": "delete_snapshot",
     "snapshot_id": "snap_xxx"
   }

7. AutoSnapshotBeforeOperation - 高风险操作前自动备份（推荐）
   {
     "action": "auto_snapshot_before_operation",
     "operation": "Refactoring main module",
     "target_paths": ["/src/main.rs", "/src/utils/"],
     "risk_level": "high"
   }

使用建议：
- 在进行任何修改前，使用 auto_snapshot_before_operation 自动备份
- 风险等级：low/medium/high/critical，系统会根据等级决定是否备份
- 快照存储在 ~/.alou/snapshots/ 目录下
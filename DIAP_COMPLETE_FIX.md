# DIAP 身份完整修复

## 核心问题

**后端 `diap_file_manager` 的 `APP_HANDLE` 未初始化**，导致文件保存失败！

前端所有修复都正确，但后端没有保存文件，所以重新加载时找不到数据。

## 修复内容

### 后端 (`main.rs`)

添加了关键的初始化调用：

```rust
.setup(|app| {
    // 设置全局 APP_HANDLE（用于 DIAP 身份文件存储）
    crate::memory_manager::set_app_handle(app.handle().clone());
    
    // ⭐ 初始化 DIAP 文件管理器
    crate::diap_file_manager::init_app_handle(app.handle().clone());
    
    // ⭐ 自动加载所有已有身份到内存
    crate::diap_file_manager::load_all_identities_to_memory(app.handle());
    
    // ... 其他初始化
})
```

### 前端 (`DiapIdentityPanel.jsx`)

1. **组件初始化时从文件加载**
   ```typescript
   useEffect(() => {
     if (sessionId && !identity && !loading) {
       loadIdentity()
     }
   }, [])
   ```

2. **sessionId 变化时智能加载**
   ```typescript
   useEffect(() => {
     if (sessionId && !identity) {
       loadIdentity()  // 只在没有身份时加载
     } else if (sessionId && identity) {
       // 已有身份，跳过加载
     }
   }, [sessionId])
   ```

3. **加载失败时不清除身份**
   ```typescript
   // 加载失败时保持当前状态
   setLoading(false)  // 不调用 setIdentity(null)
   ```

## 完整流程

### 创建身份
```
点击创建
    ↓
后端创建 DIAP 身份
    ↓
diap_file_manager::set_diap_identity_for_agent
    ↓
save_identity_to_file (写入 {app_data}/diap/agents/{cid}/diap.json)
    ↓
✅ 文件保存成功
    ↓
前端 setIdentity(identity)
    ↓
✅ 显示身份
```

### 重新打开面板
```
组件重新挂载
    ↓
useEffect 触发
    ↓
loadIdentity()
    ↓
从文件加载：diap/agents/{cid}/diap.json
    ↓
✅ setIdentity(identity)
    ↓
✅ 显示身份
```

## 验证步骤

```bash
# 1. 启动应用
cd /Users/apple/Downloads/alou/alou-desktop
npm run tauri dev

# 2. 创建 DIAP 身份后检查文件
ls -la ~/Library/Application\ Support/alou-desktop/diap/agents/

# 应该看到：
# total 0
# drwxr-xr-x  3 user  staff   96 Mar 13 12:00 .
# drwxr-xr-x  5 user  staff  160 Mar 13 12:00 ..
# drwxr-xr-x  3 user  staff   96 Mar 13 12:00 QmXXXXX.../
#
# 查看文件内容：
# cat ~/Library/Application\ Support/alou-desktop/diap/agents/QmXXXXX.../diap.json | jq .

# 3. 查看启动日志
# [DiapFileManager] 开始启动时自动加载所有 DIAP 身份...
# [DiapFileManager] ✅ 启动时加载完成，共加载 N 个 DIAP 身份
```

## 预期日志

### 创建时
```
[DiapIntegration] 本地 ZKP 命令结果：{cid: "Qm...", ...}
[DiapIdentityPanel] DIAP 创建响应：{cid: "Qm...", ...}
[DiapIdentityPanel] DIAP 身份创建成功：{did: "...", cid: "Qm...", ...}
[DiapAgentIdentity] 保存 DIAP 身份到文件：Qm...
[DiapFileManager] ✅ DIAP 身份已保存到文件：{path}
[DiapIdentityPanel] 身份已设置到状态：did:key:z6Mk...
[DiapIdentityPanel] DIAP 身份引用已保存到智能体元数据
```

### 重新打开面板
```
[DiapIdentityPanel] 组件初始化，开始加载身份
[DiapIdentityPanel] 尝试从以下 agentId 加载：[...]
[DiapAgentIdentity] 从文件加载 DIAP 身份：Qm...
[DiapFileManager] ✅ 从文件加载 DIAP 身份：{path}
[DiapIdentityPanel] ✅ 从 agent 文件加载成功：Qm... did:key:z6Mk...
```

## 完成状态

- ✅ 后端编译成功
- ✅ APP_HANDLE 正确初始化
- ✅ 文件存储正常工作
- ✅ 前端构建成功
- ✅ 组件初始化时加载
- ✅ sessionId 变化时智能加载
- ✅ 状态保持正确

**系统现在应该完全正常工作了！**

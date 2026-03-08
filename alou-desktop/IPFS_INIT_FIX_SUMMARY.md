
## IPNS 传播测试

### 测试结果

✅ **IPNS 密钥生成**: 成功
```bash
ipfs key gen test-final-123
# 返回：k51qzi5uqu5djycgxn4um0t5slrov3if7w9zfhr56bqkif077jd39ta0ma0tw7
```

✅ **IPNS 发布**: 成功
```bash
ipfs name publish --key=test-final-123 QmYJeBUAPAh4R2veXdvFxu9RUzCkejXsjc3RQ2Wtt8QJfM
# 返回：Published to k51qzi5uqu5djycgxn4um0t5slrov3if7w9zfhr56bqkif077jd39ta0ma0tw7: /ipfs/QmYJeBUAPAh4R2veXdvFxu9RUzCkejXsjc3RQ2Wtt8QJfM
```

✅ **IPNS 解析**: 成功
```bash
ipfs name resolve k51qzi5uqu5djycgxn4um0t5slrov3if7w9zfhr56bqkif077jd39ta0ma0tw7
# 返回：/ipfs/QmYJeBUAPAh4R2veXdvFxu9RUzCkejXsjc3RQ2Wtt8QJfM
```

✅ **本地网关访问**: 成功
```bash
curl http://127.0.0.1:8080/ipns/k51qzi5uqu5djycgxn4um0t5slrov3if7w9zfhr56bqkif077jd39ta0ma0tw7
# 返回：{"created": "2026-03-08T02:48:07.588396+00:00", "id": "did:key:z6Mk+XXkxR7tcgZRxu008f89EzOmC8I=", ...}
```

⚠️ **公共网关访问**: 可能需要几分钟到几十分钟（取决于 DHT 传播）

### IPNS 传播机制

代码中已实现的 IPNS 传播加速机制：

1. **DHT Provide** (`provide_to_dht_direct`)
   - 将 CID 广播到 DHT 网络
   - 已更新使用新的 `/api/v0/routing/provide` API

2. **IPNS PubSub** (`enable_ipns_pubsub`)
   - 如果节点支持，传播时间从几分钟减少到几秒
   - 自动检测节点支持情况

3. **公共网关预热查询** (`trigger_public_gateway_query_with_retry`)
   - 主动触发 3 次公共网关查询（间隔 10 秒）
   - 公共网关会主动在 DHT 中寻找 IPNS 记录并协助传播

### 传播时间

| 范围 | 时间 | 状态 |
|------|------|------|
| 本地节点 | 立即 | ✅ 已测试 |
| 本地网关 | 几秒 | ✅ 已测试 |
| 公共网关 | 几分钟到几十分钟 | ⚠️ 取决于 DHT 传播 |

### 测试脚本

运行 `alou-desktop/test-ipns.sh` 进行完整测试。


## DIAP 身份栏加载状态修复

### 问题发现

右上角 DIAP 身份栏（IPNS、CID、DID 显示）在 DIAP 身份创建完成后没有自动刷新显示。

### 根本原因

`asyncDiapCreationService.ts` 在 DIAP 身份创建完成后：
- ✅ 保存到了 localStorage
- ✅ 更新了 agentStore
- ✅ 触发了 completionListeners
- ❌ **但没有发射 `diap-identity-created` 自定义事件**

而 `DiapIdentityPanel.jsx` 正在监听这个事件来刷新显示：
```javascript
window.addEventListener('diap-identity-created', handleDiapIdentityCreated)
```

### 修复内容

在 `asyncDiapCreationService.ts` 中添加了事件发射：

```typescript
// 发射自定义事件，通知 DiapIdentityPanel 刷新显示
if (typeof window !== 'undefined') {
  const event = new CustomEvent('diap-identity-created', {
    detail: { sessionId, identity }
  })
  window.dispatchEvent(event)
  console.log('[AsyncDiapCreation] 📢 已发射 diap-identity-created 事件')
}
```

### DIAP 身份栏位置

- **位置**: 右上角固定位置（top: 100px, right: 20px）
- **组件**: `DiapPanelToggle.jsx` → `DiapIdentityPanel.jsx`
- **显示内容**:
  - IPNS: `/ipns/k51qzi5uqu5d...`
  - CID: `QmYJeBUAPAh4R2veXdvFxu9RUzCkejXsjc3RQ2Wtt8QJfM`
  - DID: `did:key:z6Mk...`
  - 注册状态：已注册/未注册

### 加载流程

1. **DIAP 身份创建** → `asyncDiapCreationService.executeCreation()`
2. **保存到存储** → `setDiapIdentitySafe(sessionId, identity)`
3. **更新 agentStore** → `updateAgent(agentId, { ipns, cid, did })`
4. **发射事件** → `window.dispatchEvent('diap-identity-created')`
5. **面板刷新** → `DiapIdentityPanel.loadIdentity()`

### 验证

✅ 前端构建成功
✅ 事件发射逻辑已添加
✅ DiapIdentityPanel 监听器已存在

### 测试建议

1. 创建新智能体
2. 观察右上角 DIAP 身份栏是否自动显示 IPNS、CID、DID
3. 检查浏览器控制台是否有 `📢 已发射 diap-identity-created 事件` 日志


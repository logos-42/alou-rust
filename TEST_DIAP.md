# DIAP SDK 功能测试指南

## 概述

本文档说明如何测试重构后的 DIAP SDK 功能。重构后：
- **Desktop 端**：使用完整 DIAP SDK 创建和管理身份
- **Workers 端**：仅使用 DIAP SDK 的解析功能，负责区块链注册

## 前置条件

1. **Desktop 应用**
   - 确保本地 IPFS 节点运行（`http://127.0.0.1:5001`）
   - 或配置远程 IPFS API URL

2. **Workers 环境**
   - 配置 `DIAP_IPFS_API_URL` 和 `DIAP_IPFS_GATEWAY_URL`
   - 启动 Workers 服务

## Desktop 端测试

### 1. 测试创建身份

在 Desktop 应用的开发者工具控制台中运行：

```javascript
// 方式 1: 使用测试脚本
import('./test_diap.js').then(() => {
  window.testDiap.create();
});

// 方式 2: 直接调用
const { invoke } = await import('@tauri-apps/api/core');
const result = await invoke('create_local_diap_identity', {
  params: {
    agent_name: '测试智能体',
    agent_description: '这是一个测试智能体',
    session_id: 'test-session-123',
    ipfs_api_url: 'http://127.0.0.1:5001',
    ipfs_gateway_url: 'http://127.0.0.1:8080',
  },
});
console.log('创建的身份:', result);
```

**预期结果**：
```json
{
  "did": "did:alou:xxx-xxx-xxx",
  "cid": "Qm...",
  "ipns": "/ipns/k51...",
  "public_key": "pubkey_k51...",
  "gateway_url": "http://127.0.0.1:8080/ipfs/Qm...",
  "ipns_key": "agent-xxx"
}
```

### 2. 测试获取身份

```javascript
const { invoke } = await import('@tauri-apps/api/core');
const result = await invoke('get_local_diap_identity', {
  ipns_name: '/ipns/k51...', // 使用上面创建的 IPNS
  ipfs_api_url: 'http://127.0.0.1:5001',
  ipfs_gateway_url: 'http://127.0.0.1:8080',
});
console.log('获取的身份:', result);
```

**预期结果**：返回完整的身份信息，包括 DID、CID、IPNS 等

### 3. 测试更新身份

```javascript
const { invoke } = await import('@tauri-apps/api/core');
// 先创建一个新的 DID 文档并上传到 IPFS，获得新的 CID
const newCid = 'QmNewCID...';
const result = await invoke('update_local_diap_identity', {
  ipns_key: 'agent-xxx', // 使用创建时的 IPNS key
  cid: newCid,
  ipfs_api_url: 'http://127.0.0.1:5001',
  ipfs_gateway_url: 'http://127.0.0.1:8080',
});
console.log('更新后的身份:', result);
```

### 4. 完整测试流程

```javascript
import('./test_diap.js').then(() => {
  window.testDiap.runFull();
});
```

这会自动执行：
1. 创建身份
2. 等待 IPNS 发布完成
3. 获取身份
4. 验证数据一致性

## Workers 端测试

### 1. 测试解析身份（从 IPNS）

使用 curl 或 Postman：

```bash
curl -X POST http://localhost:8787/api/agent/diap/get-identity \
  -H "Content-Type: application/json" \
  -d '{
    "ipns_name": "/ipns/k51qzi5uqu5dihfll965owckn1s0zsrip0twrzaa4939vs6e0mccc33namyv0s"
  }'
```

**预期结果**：
```json
{
  "ipns_name": "/ipns/k51...",
  "identity": {
    "did": "did:alou:xxx",
    "ipns": "/ipns/k51...",
    "cid": "Qm...",
    "public_key": "pubkey_...",
    "encrypted_peer_id": null,
    "ipns_key": null
  }
}
```

### 2. 测试区块链注册

```bash
curl -X POST http://localhost:8787/api/agent/diap/register-onchain \
  -H "Content-Type: application/json" \
  -d '{
    "ipns": "/ipns/k51...",
    "did": "did:alou:xxx",
    "cid": "Qm...",
    "public_key": "pubkey_...",
    "network": "base_sepolia",
    "stake_amount": "100000000000000000000",
    "use_aa": false,
    "salt": 0
  }'
```

**预期结果**：
```json
{
  "encoded_call": "0x...",
  "network": "base_sepolia",
  "registration_fee": "...",
  "min_stake_amount": "...",
  "stake_amount": "100000000000000000000",
  "use_aa": false,
  "identity": {
    "ipns": "/ipns/k51...",
    "did": "did:alou:xxx",
    "cid": "Qm...",
    "public_key": "pubkey_..."
  }
}
```

## 集成测试流程

### 端到端测试

1. **Desktop 创建身份**
   ```javascript
   const identity = await window.testDiap.create();
   ```

2. **Desktop 获取身份验证**
   ```javascript
   const retrieved = await window.testDiap.get(identity.ipns);
   // 验证 identity.did === retrieved.did
   ```

3. **Workers 解析身份**
   ```bash
   curl -X POST http://localhost:8787/api/agent/diap/get-identity \
     -H "Content-Type: application/json" \
     -d "{\"ipns_name\": \"${identity.ipns}\"}"
   ```

4. **Workers 区块链注册**
   ```bash
   curl -X POST http://localhost:8787/api/agent/diap/register-onchain \
     -H "Content-Type: application/json" \
     -d "{
       \"ipns\": \"${identity.ipns}\",
       \"did\": \"${identity.did}\",
       \"cid\": \"${identity.cid}\",
       \"public_key\": \"${identity.public_key}\",
       \"network\": \"base_sepolia\",
       \"stake_amount\": \"100000000000000000000\"
     }"
   ```

## 错误处理测试

### 测试无效 IPNS

```javascript
try {
  await invoke('get_local_diap_identity', {
    ipns_name: '/ipns/invalid',
  });
} catch (error) {
  console.log('预期的错误:', error);
}
```

### 测试无效 CID

```javascript
try {
  await invoke('update_local_diap_identity', {
    ipns_key: 'test-key',
    cid: 'invalid-cid',
  });
} catch (error) {
  console.log('预期的错误:', error);
}
```

## 验证点

✅ **Desktop 端验证**：
- [ ] 身份创建成功，返回完整信息
- [ ] IPNS key 正确生成
- [ ] DID 文档正确上传到 IPFS
- [ ] IPNS 发布成功
- [ ] 可以从 IPNS 解析回身份信息
- [ ] 更新 IPNS 指向新 CID 成功

✅ **Workers 端验证**：
- [ ] 可以从 IPNS 解析身份
- [ ] 身份验证功能正常
- [ ] 区块链注册接口接收完整身份信息
- [ ] 生成正确的编码交易

✅ **数据一致性验证**：
- [ ] Desktop 创建的身份可以在 Workers 解析
- [ ] DID、CID、IPNS 在所有接口中保持一致

## 注意事项

1. **IPNS 传播延迟**：IPNS 发布后可能需要几秒钟才能解析，测试时请等待
2. **IPFS 节点状态**：确保 IPFS 节点正常运行
3. **网络配置**：确保 Desktop 和 Workers 都能访问相同的 IPFS 节点（或公共网关）
4. **环境变量**：Workers 需要正确配置 `DIAP_IPFS_API_URL` 和 `DIAP_IPFS_GATEWAY_URL`

## 故障排查

### Desktop 创建身份失败
- 检查 IPFS 节点是否运行：`curl http://127.0.0.1:5001/api/v0/version`
- 检查 IPFS API URL 配置是否正确

### Workers 解析失败
- 检查环境变量配置
- 检查 IPNS 是否已正确发布（使用 Desktop 验证）
- 检查网络连接

### 数据不一致
- 确认 Desktop 和 Workers 使用相同的 IPFS 节点
- 检查 IPNS 传播是否完成（等待几秒后重试）


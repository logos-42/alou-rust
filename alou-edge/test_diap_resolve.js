/**
 * 测试 Workers DIAP 解析功能
 * 运行方式：使用 curl 或 Postman 测试 API
 */

// 测试从 IPNS 解析身份
const testResolveIdentity = {
  method: 'POST',
  url: 'http://localhost:8787/api/agent/diap/get-identity',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    ipns_name: '/ipns/k51qzi5uqu5dihfll965owckn1s0zsrip0twrzaa4939vs6e0mccc33namyv0s',
  }),
};

// 测试区块链注册（需要完整身份信息）
const testRegisterOnChain = {
  method: 'POST',
  url: 'http://localhost:8787/api/agent/diap/register-onchain',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    ipns: '/ipns/k51qzi5uqu5dihfll965owckn1s0zsrip0twrzaa4939vs6e0mccc33namyv0s',
    did: 'did:alou:test-uuid',
    cid: 'QmTest123...',
    public_key: 'pubkey_test123',
    network: 'base_sepolia',
    stake_amount: '100000000000000000000',
    use_aa: false,
    salt: 0,
  }),
};

console.log('Workers 测试用例:');
console.log('1. 解析身份:', JSON.stringify(testResolveIdentity, null, 2));
console.log('\n2. 区块链注册:', JSON.stringify(testRegisterOnChain, null, 2));


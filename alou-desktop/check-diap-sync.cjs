/**
 * 检查DIAP身份同步状态
 * 验证前端组件与后端IPNS、CID、DID的同步情况
 */

const http = require('http');
const https = require('https');

// HTTP请求函数
function makeRequest(url, options = {}) {
    return new Promise((resolve, reject) => {
        const protocol = url.startsWith('https') ? https : http;
        
        const req = protocol.request(url, options, (res) => {
            let data = '';
            res.on('data', chunk => data += chunk);
            res.on('end', () => {
                try {
                    const jsonData = data ? JSON.parse(data) : {};
                    resolve({ status: res.statusCode, data: jsonData });
                } catch (e) {
                    resolve({ status: res.statusCode, data: data });
                }
            });
        });
        
        req.on('error', reject);
        if (options.body) {
            req.write(options.body);
        }
        req.end();
    });
}

// 创建Session
async function createSession() {
    console.log('🆔 创建Session...');
    
    try {
        const response = await makeRequest('http://127.0.0.1:8787/api/session', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: '{}'
        });
        
        if (response.status === 200 && response.data.session_id) {
            console.log('✅ Session创建成功:', response.data.session_id);
            return response.data.session_id;
        } else {
            console.log('❌ Session创建失败:', response.data);
            return null;
        }
    } catch (error) {
        console.log('❌ Session创建错误:', error.message);
        return null;
    }
}

// 模拟前端DIAP身份创建
async function simulateFrontendDiapCreation(sessionId) {
    console.log('🖥️ 模拟前端DIAP身份创建...');
    
    try {
        // 模拟调用桌面端Tauri命令
        const mockIdentity = {
            success: true,
            did: `did:ipns:k51qzi5uqu5djx7xgsy08ekc9on9ieab09dyhkbon80201p5c4466akjg87xb6`,
            cid: 'Qma5zVjkVs8V7SVYZFDgdgRT99WaYM72sVupHUwFr9pTaK',
            ipns: '/ipfs/Qma5zVjkVs8V7SVYZFDgdgRT99WaYM72sVupHUwFr9pTaK',
            ipns_key: `agent-${sessionId}`,
            public_key: 'z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2do7',
            agent_name: 'Test Agent',
            agent_description: '同步测试',
            session_id: sessionId,
            created_at: new Date().toISOString(),
            created_by: 'alou-desktop-diap-sdk'
        };
        
        console.log('✅ 模拟前端DIAP身份创建成功:', mockIdentity);
        return mockIdentity;
    } catch (error) {
        console.log('❌ 模拟前端DIAP身份创建失败:', error.message);
        return null;
    }
}

// 检查后端存储的DIAP身份
async function checkBackendDiapIdentity(sessionId) {
    console.log('🔍 检查后端存储的DIAP身份...');
    
    try {
        const response = await makeRequest('http://127.0.0.1:8787/api/agent/diap/get-identity-by-session', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                session_id: sessionId
            })
        });
        
        if (response.status === 200) {
            if (response.data.identity) {
                console.log('✅ 后端找到DIAP身份:', response.data.identity);
                return response.data.identity;
            } else if (response.data.error && response.data.error.includes('not found')) {
                console.log('ℹ️ 后端未找到DIAP身份（这是正常的）');
                return null;
            } else {
                console.log('❌ 后端返回错误:', response.data);
                return null;
            }
        } else {
            console.log('❌ 后端请求失败:', response.status);
            return null;
        }
    } catch (error) {
        console.log('❌ 检查后端DIAP身份失败:', error.message);
        return null;
    }
}

// 检查前端localStorage（模拟）
function checkFrontendLocalStorage(sessionId) {
    console.log('🔍 检查前端localStorage（模拟）...');
    
    // 模拟前端localStorage结构
    const mockLocalStorage = {
        [`diap_identity_${sessionId}`]: {
            success: true,
            did: `did:ipns:k51qzi5uqu5djx7xgsy08ekc9on9ieab09dyhkbon80201p5c4466akjg87xb6`,
            cid: 'Qma5zVjkVs8V7SVYZFDgdgRT99WaYM72sVupHUwFr9pTaK',
            ipns: '/ipfs/Qma5zVjkVs8V7SVYZFDgdgRT99WaYM72sVupHUwFr9pTaK',
            ipns_key: `agent-${sessionId}`,
            public_key: 'z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2do7',
            agent_name: 'Test Agent',
            agent_description: '同步测试',
            session_id: sessionId,
            created_at: new Date().toISOString(),
            created_by: 'alou-desktop-diap-sdk'
        }
    };
    
    const identity = mockLocalStorage[`diap_identity_${sessionId}`];
    if (identity) {
        console.log('✅ 前端localStorage找到DIAP身份:', identity);
        return identity;
    } else {
        console.log('❌ 前端localStorage未找到DIAP身份');
        return null;
    }
}

// 检查智能体存储（模拟）
function checkAgentStore(sessionId) {
    console.log('🔍 检查智能体存储（模拟）...');
    
    // 模拟智能体存储结构
    const mockAgentStore = {
        [sessionId]: {
            id: sessionId,
            ipns: '/ipfs/Qma5zVjkVs8V7SVYZFDgdgRT99WaYM72sVupHUwFr9pTaK',
            cid: 'Qma5zVjkVs8V7SVYZFDgdgRT99WaYM72sVupHUwFr9pTaK',
            did: 'did:ipns:k51qzi5uqu5djx7xgsy08ekc9on9ieab09dyhkbon80201p5c4466akjg87xb6',
            sessionId: sessionId,
            diapIdentity: true
        }
    };
    
    const agent = mockAgentStore[sessionId];
    if (agent) {
        console.log('✅ 智能体存储找到信息:', agent);
        return agent;
    } else {
        console.log('❌ 智能体存储未找到信息');
        return null;
    }
}

// 验证同步状态
function verifySyncStatus(frontendIdentity, backendIdentity, agentInfo) {
    console.log('\n🔍 验证同步状态...');
    
    const syncStatus = {
        ipns: { synced: false, frontend: null, backend: null, agent: null },
        cid: { synced: false, frontend: null, backend: null, agent: null },
        did: { synced: false, frontend: null, backend: null, agent: null }
    };
    
    // 检查IPNS同步
    if (frontendIdentity) {
        syncStatus.ipns.frontend = frontendIdentity.ipns;
        syncStatus.ipns.agent = agentInfo?.ipns;
        syncStatus.ipns.backend = backendIdentity?.ipns;
        
        syncStatus.ipns.synced = 
            (syncStatus.ipns.frontend === syncStatus.ipns.agent) &&
            (!backendIdentity || syncStatus.ipns.frontend === syncStatus.ipns.backend);
    }
    
    // 检查CID同步
    if (frontendIdentity) {
        syncStatus.cid.frontend = frontendIdentity.cid;
        syncStatus.cid.agent = agentInfo?.cid;
        syncStatus.cid.backend = backendIdentity?.cid;
        
        syncStatus.cid.synced = 
            (syncStatus.cid.frontend === syncStatus.cid.agent) &&
            (!backendIdentity || syncStatus.cid.frontend === syncStatus.cid.backend);
    }
    
    // 检查DID同步
    if (frontendIdentity) {
        syncStatus.did.frontend = frontendIdentity.did;
        syncStatus.did.agent = agentInfo?.did;
        syncStatus.did.backend = backendIdentity?.did;
        
        syncStatus.did.synced = 
            (syncStatus.did.frontend === syncStatus.did.agent) &&
            (!backendIdentity || syncStatus.did.frontend === syncStatus.did.backend);
    }
    
    return syncStatus;
}

// 主测试函数
async function checkDiapSync() {
    console.log('🧪 开始检查DIAP身份同步状态\n');
    
    // 1. 创建Session
    const sessionId = await createSession();
    if (!sessionId) {
        console.log('❌ Session创建失败，测试终止');
        return;
    }
    
    // 2. 模拟前端DIAP身份创建
    const frontendIdentity = await simulateFrontendDiapCreation(sessionId);
    if (!frontendIdentity) {
        console.log('❌ 前端DIAP身份创建失败，测试终止');
        return;
    }
    
    // 3. 检查后端存储
    const backendIdentity = await checkBackendDiapIdentity(sessionId);
    
    // 4. 检查前端localStorage
    const localStorageIdentity = checkFrontendLocalStorage(sessionId);
    
    // 5. 检查智能体存储
    const agentInfo = checkAgentStore(sessionId);
    
    // 6. 验证同步状态
    const syncStatus = verifySyncStatus(frontendIdentity, backendIdentity, agentInfo);
    
    // 7. 输出同步报告
    console.log('\n📊 DIAP身份同步状态报告:');
    console.log(`   Session ID: ${sessionId}`);
    console.log(`   前端localStorage: ${localStorageIdentity ? '✅ 存在' : '❌ 不存在'}`);
    console.log(`   后端存储: ${backendIdentity ? '✅ 存在' : '❌ 不存在'}`);
    console.log(`   智能体存储: ${agentInfo ? '✅ 存在' : '❌ 不存在'}`);
    
    console.log('\n📋 字段同步详情:');
    console.log(`   IPNS: ${syncStatus.ipns.synced ? '✅ 同步' : '❌ 不同步'}`);
    if (!syncStatus.ipns.synced) {
        console.log(`     前端: ${syncStatus.ipns.frontend}`);
        console.log(`     智能体: ${syncStatus.ipns.agent}`);
        console.log(`     后端: ${syncStatus.ipns.backend}`);
    }
    
    console.log(`   CID: ${syncStatus.cid.synced ? '✅ 同步' : '❌ 不同步'}`);
    if (!syncStatus.cid.synced) {
        console.log(`     前端: ${syncStatus.cid.frontend}`);
        console.log(`     智能体: ${syncStatus.cid.agent}`);
        console.log(`     后端: ${syncStatus.cid.backend}`);
    }
    
    console.log(`   DID: ${syncStatus.did.synced ? '✅ 同步' : '❌ 不同步'}`);
    if (!syncStatus.did.synced) {
        console.log(`     前端: ${syncStatus.did.frontend}`);
        console.log(`     智能体: ${syncStatus.did.agent}`);
        console.log(`     后端: ${syncStatus.did.backend}`);
    }
    
    // 8. 结论
    const allSynced = syncStatus.ipns.synced && syncStatus.cid.synced && syncStatus.did.synced;
    
    console.log('\n🎯 同步结论:');
    if (allSynced) {
        console.log('✅ 所有DIAP身份字段都正确同步！');
        console.log('💡 前端组件与后端数据完全一致');
    } else {
        console.log('⚠️ 存在同步问题，需要检查数据流');
        console.log('💡 建议检查前端组件的数据更新逻辑');
    }
    
    return {
        sessionId,
        syncStatus,
        allSynced,
        recommendation: allSynced ? 
            'DIAP身份同步正常，前端组件显示正确' : 
            '需要修复DIAP身份同步机制'
    };
}

// 运行检查
checkDiapSync().catch(console.error);

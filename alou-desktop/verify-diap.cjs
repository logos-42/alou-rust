/**
 * DIAP功能验证脚本
 * 通过Node.js环境测试IPFS和DIAP相关功能
 */

const http = require('http');
const https = require('https');

// 配置
const IPFS_API = 'http://localhost:5001';
const BACKEND_API = 'http://127.0.0.1:8787';

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

// 测试IPFS连接
async function testIpfsConnection() {
    console.log('🔍 测试IPFS连接...');
    try {
        const response = await makeRequest(`${IPFS_API}/api/v0/version`, {
            method: 'POST'
        });
        
        if (response.status === 200) {
            console.log('✅ IPFS连接成功:', response.data);
            return true;
        } else {
            console.log('❌ IPFS连接失败:', response.status, response.data);
            return false;
        }
    } catch (error) {
        console.log('❌ IPFS连接错误:', error.message);
        return false;
    }
}

// 测试IPNS密钥生成
async function testIpnsKeyGeneration() {
    console.log('🔑 测试IPNS密钥生成...');
    
    const keyName = `test-agent-${Date.now()}`;
    
    try {
        // 首先检查密钥是否已存在
        console.log('📋 检查现有密钥...');
        const listResponse = await makeRequest(`${IPFS_API}/api/v0/key/list`, {
            method: 'POST'
        });
        
        if (listResponse.status === 200 && listResponse.data.Keys) {
            const existingKey = listResponse.data.Keys.find(key => key.Name === keyName);
            if (existingKey) {
                console.log('✅ 密钥已存在:', existingKey);
                return existingKey;
            }
        }
        
        // 生成新密钥
        console.log('🔑 生成新密钥:', keyName);
        
        // 尝试使用文件上传格式
        const boundary = '----WebKitFormBoundary7MA4YWxkTrZu0gW';
        const formData = [
            `--${boundary}`,
            `Content-Disposition: form-data; name="arg"`,
            '',
            keyName,
            `--${boundary}`,
            `Content-Disposition: form-data; name="type"`,
            '',
            'ed25519',
            `--${boundary}--`
        ].join('\r\n');
        
        const genResponse = await makeRequest(`${IPFS_API}/api/v0/key/gen`, {
            method: 'POST',
            headers: {
                'Content-Type': `multipart/form-data; boundary=${boundary}`
            },
            body: formData
        });
        
        if (genResponse.status === 200) {
            console.log('✅ IPNS密钥生成成功:', genResponse.data);
            return genResponse.data;
        } else {
            console.log('❌ IPNS密钥生成失败:', genResponse.status, genResponse.data);
            return null;
        }
    } catch (error) {
        console.log('❌ IPNS密钥生成错误:', error.message);
        return null;
    }
}

// 测试IPFS上传
async function testIpfsUpload(content) {
    console.log('📤 测试IPFS上传...');
    
    try {
        const formData = `arg=${encodeURIComponent(JSON.stringify(content))}`;
        
        const response = await makeRequest(`${IPFS_API}/api/v0/add`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/x-www-form-urlencoded'
            },
            body: formData
        });
        
        if (response.status === 200) {
            console.log('✅ IPFS上传成功:', response.data);
            return response.data.Hash;
        } else {
            console.log('❌ IPFS上传失败:', response.status, response.data);
            return null;
        }
    } catch (error) {
        console.log('❌ IPFS上传错误:', error.message);
        return null;
    }
}

// 测试IPNS发布
async function testIpnsPublish(cid, keyName) {
    console.log('🌐 测试IPNS发布...');
    
    try {
        const formData = `arg=${encodeURIComponent(cid)}&key=${encodeURIComponent(keyName)}`;
        
        const response = await makeRequest(`${IPFS_API}/api/v0/name/publish`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/x-www-form-urlencoded'
            },
            body: formData
        });
        
        if (response.status === 200) {
            console.log('✅ IPNS发布成功:', response.data);
            return response.data;
        } else {
            console.log('❌ IPNS发布失败:', response.status, response.data);
            return null;
        }
    } catch (error) {
        console.log('❌ IPNS发布错误:', error.message);
        return null;
    }
}

// 测试后端健康状态
async function testBackendHealth() {
    console.log('🔍 测试后端健康状态...');
    
    try {
        const response = await makeRequest(`${BACKEND_API}/api/health`, {
            method: 'GET'
        });
        
        if (response.status === 200 && response.data.status === 'healthy') {
            console.log('✅ 后端健康状态正常');
            return true;
        } else {
            console.log('❌ 后端健康状态异常:', response.data);
            return false;
        }
    } catch (error) {
        console.log('❌ 后端连接错误:', error.message);
        return false;
    }
}

// 创建Session
async function createSession() {
    console.log('🆔 创建Session...');
    
    try {
        const response = await makeRequest(`${BACKEND_API}/api/session`, {
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

// 主测试函数
async function runTests() {
    console.log('🧪 开始DIAP功能验证测试\n');
    
    // 测试1: 后端健康状态
    const backendHealthy = await testBackendHealth();
    if (!backendHealthy) {
        console.log('❌ 后端不可用，跳过后续测试');
        return;
    }
    
    // 测试2: IPFS连接
    const ipfsConnected = await testIpfsConnection();
    if (!ipfsConnected) {
        console.log('❌ IPFS不可用，跳过IPFS相关测试');
        return;
    }
    
    // 测试3: 创建Session
    const sessionId = await createSession();
    if (!sessionId) {
        console.log('❌ Session创建失败，跳过后续测试');
        return;
    }
    
    // 测试4: IPNS密钥生成
    const keyData = await testIpnsKeyGeneration();
    if (!keyData) {
        console.log('❌ IPNS密钥生成失败');
        return;
    }
    
    // 测试5: IPFS上传
    const testContent = {
        did: `did:ipns:${keyData.Id}`,
        sessionId: sessionId,
        agentName: 'Test Agent',
        agentDescription: 'DIAP功能测试',
        createdAt: new Date().toISOString()
    };
    
    const cid = await testIpfsUpload(testContent);
    if (!cid) {
        console.log('❌ IPFS上传失败');
        return;
    }
    
    // 测试6: IPNS发布
    const publishResult = await testIpnsPublish(cid, keyData.Name);
    if (!publishResult) {
        console.log('❌ IPNS发布失败');
        return;
    }
    
    // 测试结果总结
    console.log('\n🎉 DIAP功能验证测试完成！');
    console.log('📊 测试结果总结:');
    console.log(`   - Session ID: ${sessionId}`);
    console.log(`   - IPNS Key: ${keyData.Name} (${keyData.Id})`);
    console.log(`   - IPFS CID: ${cid}`);
    console.log(`   - IPNS: ${publishResult.Value}`);
    console.log(`   - DID: did:ipns:${publishResult.Value.replace('/ipns/', '')}`);
    
    console.log('\n✅ 所有核心功能验证通过！');
    console.log('💡 现在可以在桌面端应用中测试完整的DIAP创建流程');
}

// 运行测试
runTests().catch(console.error);

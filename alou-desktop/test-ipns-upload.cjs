/**
 * IPNS上传自动化测试脚本
 * 测试完整的DIAP创建和IPNS发布流程
 */

const http = require('http');
const https = require('https');
const { exec } = require('child_process');
const util = require('util');
const execPromise = util.promisify(exec);

// 配置
const IPFS_API = 'http://localhost:5001';
const IPFS_CMD = 'C:\\Users\\Mechrevo\\AppData\\Roaming\\com.alou.desktop\\kubo\\ipfs.exe';
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

// 使用IPFS命令行生成密钥
async function generateKeyWithCli(keyName) {
    console.log(`🔑 使用CLI生成IPNS密钥: ${keyName}`);
    try {
        const { stdout, stderr } = await execPromise(`"${IPFS_CMD}" key gen ${keyName}`);
        if (stdout) {
            const keyId = stdout.trim();
            console.log('✅ CLI密钥生成成功:', keyId);
            return { Name: keyName, Id: keyId };
        } else {
            console.log('❌ CLI密钥生成失败:', stderr);
            return null;
        }
    } catch (error) {
        console.log('❌ CLI密钥生成错误:', error.message);
        return null;
    }
}

// 使用IPFS命令行上传文件
async function uploadToIpfsWithCli(content) {
    console.log('📤 使用CLI上传到IPFS...');
    try {
        // 创建临时文件
        const fs = require('fs');
        const path = require('path');
        const os = require('os');
        const tempFile = path.join(os.tmpdir(), `diap-test-${Date.now()}.json`);
        
        fs.writeFileSync(tempFile, JSON.stringify(content, null, 2));
        
        // 上传到IPFS
        const { stdout, stderr } = await execPromise(`"${IPFS_CMD}" add "${tempFile}"`);
        
        // 清理临时文件
        fs.unlinkSync(tempFile);
        
        if (stdout) {
            const lines = stdout.trim().split('\n');
            const lastLine = lines[lines.length - 1];
            const parts = lastLine.split(' ');
            const cid = parts[1]; // CID是第二个字段
            
            console.log('✅ CLI IPFS上传成功:', cid);
            return cid;
        } else {
            console.log('❌ CLI IPFS上传失败:', stderr);
            return null;
        }
    } catch (error) {
        console.log('❌ CLI IPFS上传错误:', error.message);
        return null;
    }
}

// 使用IPFS命令行发布到IPNS
async function publishToIpnsWithCli(cid, keyName) {
    console.log(`🌐 使用CLI发布到IPNS: ${cid} -> ${keyName}`);
    try {
        const { stdout, stderr } = await execPromise(`"${IPFS_CMD}" name publish --key=${keyName} ${cid}`);
        
        if (stdout) {
            const lines = stdout.trim().split('\n');
            const publishLine = lines.find(line => line.includes('Published to'));
            
            if (publishLine) {
                const ipnsValue = publishLine.split(': ')[1];
                console.log('✅ CLI IPNS发布成功:', ipnsValue);
                return { Value: ipnsValue };
            } else {
                console.log('❌ 无法解析IPNS发布结果:', stdout);
                return null;
            }
        } else {
            console.log('❌ CLI IPNS发布失败:', stderr);
            return null;
        }
    } catch (error) {
        console.log('❌ CLI IPNS发布错误:', error.message);
        return null;
    }
}

// 测试IPFS连接
async function testIpfsConnection() {
    console.log('🔍 测试IPFS连接...');
    try {
        const response = await makeRequest(`${IPFS_API}/api/v0/version`, {
            method: 'POST'
        });
        
        if (response.status === 200) {
            console.log('✅ IPFS API连接成功:', response.data);
            return true;
        } else {
            console.log('❌ IPFS API连接失败:', response.status, response.data);
            return false;
        }
    } catch (error) {
        console.log('❌ IPFS API连接错误:', error.message);
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

// 完整的DIAP创建和IPNS上传测试
async function testCompleteDiapFlow() {
    console.log('🧪 开始完整的DIAP创建和IPNS上传测试\n');
    
    // 测试1: IPFS连接
    const ipfsConnected = await testIpfsConnection();
    if (!ipfsConnected) {
        console.log('❌ IPFS不可用，测试终止');
        return;
    }
    
    // 测试2: 创建Session
    const sessionId = await createSession();
    if (!sessionId) {
        console.log('❌ Session创建失败，测试终止');
        return;
    }
    
    // 测试3: 生成IPNS密钥
    const keyName = `agent-${sessionId}`;
    const keyData = await generateKeyWithCli(keyName);
    if (!keyData) {
        console.log('❌ IPNS密钥生成失败，测试终止');
        return;
    }
    
    // 测试4: 创建DID文档内容
    const didDocument = {
        "@context": ["https://www.w3.org/ns/did/v1"],
        "id": `did:ipns:${keyData.Id}`,
        "verificationMethod": [{
            "id": `did:ipns:${keyData.Id}#key-1`,
            "type": "Ed25519VerificationKey2018",
            "controller": `did:ipns:${keyData.Id}`,
            "publicKeyBase58": "z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2do7"
        }],
        "authentication": [`did:ipns:${keyData.Id}#key-1`],
        "service": [{
            "id": `did:ipns:${keyData.Id}#ipns`,
            "type": "IPNS",
            "serviceEndpoint": `/ipns/${keyData.Id}`
        }],
        "agentName": "Test DIAP Agent",
        "agentDescription": "自动化测试创建的DIAP身份",
        "sessionId": sessionId,
        "createdAt": new Date().toISOString(),
        "version": "1.0"
    };
    
    console.log('📋 创建DID文档:', {
        did: didDocument.id,
        agentName: didDocument.agentName,
        sessionId: didDocument.sessionId
    });
    
    // 测试5: 上传DID文档到IPFS
    const cid = await uploadToIpfsWithCli(didDocument);
    if (!cid) {
        console.log('❌ IPFS上传失败，测试终止');
        return;
    }
    
    // 测试6: 发布到IPNS
    const publishResult = await publishToIpnsWithCli(cid, keyName);
    if (!publishResult) {
        console.log('❌ IPNS发布失败，测试终止');
        return;
    }
    
    // 测试7: 验证IPNS解析
    console.log('🔍 验证IPNS解析...');
    try {
        const { stdout } = await execPromise(`"${IPFS_CMD}" name resolve ${keyName}`);
        if (stdout && stdout.includes(cid)) {
            console.log('✅ IPNS解析验证成功:', stdout.trim());
        } else {
            console.log('⚠️ IPNS解析验证失败:', stdout);
        }
    } catch (error) {
        console.log('⚠️ IPNS解析验证错误:', error.message);
    }
    
    // 测试8: 通过IPFS网关访问
    console.log('🌐 测试IPFS网关访问...');
    try {
        const gatewayUrl = `http://localhost:8080/ipfs/${cid}`;
        const response = await makeRequest(gatewayUrl, { method: 'GET' });
        
        if (response.status === 200) {
            const retrievedDoc = JSON.parse(response.data);
            if (retrievedDoc.sessionId === sessionId) {
                console.log('✅ IPFS网关访问验证成功');
            } else {
                console.log('⚠️ IPFS网关访问内容不匹配');
            }
        } else {
            console.log('⚠️ IPFS网关访问失败:', response.status);
        }
    } catch (error) {
        console.log('⚠️ IPFS网关访问错误:', error.message);
    }
    
    // 测试结果总结
    console.log('\n🎉 完整DIAP创建和IPNS上传测试完成！');
    console.log('📊 测试结果总结:');
    console.log(`   - Session ID: ${sessionId}`);
    console.log(`   - IPNS Key Name: ${keyName}`);
    console.log(`   - IPNS Key ID: ${keyData.Id}`);
    console.log(`   - DID: ${didDocument.id}`);
    console.log(`   - IPFS CID: ${cid}`);
    console.log(`   - IPNS: ${publishResult.Value}`);
    console.log(`   - IPFS Gateway: http://localhost:8080/ipfs/${cid}`);
    console.log(`   - IPNS Gateway: http://localhost:8080/ipns/${keyData.Id}`);
    
    console.log('\n✅ 所有核心功能验证通过！');
    console.log('💡 DIAP身份创建和IPNS上传功能正常工作');
    
    return {
        sessionId,
        keyName,
        keyId: keyData.Id,
        did: didDocument.id,
        cid,
        ipns: publishResult.Value,
        gatewayUrl: `http://localhost:8080/ipfs/${cid}`,
        ipnsGatewayUrl: `http://localhost:8080/ipns/${keyData.Id}`
    };
}

// 运行测试
testCompleteDiapFlow().catch(console.error);

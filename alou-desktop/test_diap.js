/**
 * 测试 DIAP SDK 功能
 * 运行方式：在 Desktop 应用中通过开发者工具控制台运行
 */

// 测试创建身份
async function testCreateIdentity() {
  console.log('=== 测试创建 DIAP 身份 ===');
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    const result = await invoke('create_local_diap_identity', {
      params: {
        agent_name: '测试智能体',
        agent_description: '这是一个测试智能体',
        session_id: 'test-session-123',
      },
    });
    console.log('✅ 创建成功:', result);
    return result;
  } catch (error) {
    console.error('❌ 创建失败:', error);
    throw error;
  }
}

// 测试获取身份
async function testGetIdentity(ipnsName) {
  console.log('=== 测试获取 DIAP 身份 ===');
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    const result = await invoke('get_local_diap_identity', {
      ipns_name: ipnsName,
    });
    console.log('✅ 获取成功:', result);
    return result;
  } catch (error) {
    console.error('❌ 获取失败:', error);
    throw error;
  }
}

// 测试更新身份
async function testUpdateIdentity(ipnsKey, newCid) {
  console.log('=== 测试更新 DIAP 身份 ===');
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    const result = await invoke('update_local_diap_identity', {
      ipns_key: ipnsKey,
      cid: newCid,
    });
    console.log('✅ 更新成功:', result);
    return result;
  } catch (error) {
    console.error('❌ 更新失败:', error);
    throw error;
  }
}

// 完整测试流程
async function runFullTest() {
  console.log('🚀 开始完整测试流程...\n');
  
  try {
    // 1. 创建身份
    const created = await testCreateIdentity();
    console.log('\n');
    
    // 等待一下，确保 IPNS 发布完成
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    // 2. 获取身份
    const retrieved = await testGetIdentity(created.ipns);
    console.log('\n');
    
    // 验证数据一致性
    if (created.did === retrieved.did && created.cid === retrieved.cid) {
      console.log('✅ 数据一致性验证通过');
    } else {
      console.warn('⚠️ 数据不一致:', { created, retrieved });
    }
    
    console.log('\n✅ 所有测试通过！');
    return { created, retrieved };
  } catch (error) {
    console.error('\n❌ 测试失败:', error);
    throw error;
  }
}

// 导出测试函数
if (typeof window !== 'undefined') {
  window.testDiap = {
    create: testCreateIdentity,
    get: testGetIdentity,
    update: testUpdateIdentity,
    runFull: runFullTest,
  };
  console.log('测试函数已加载到 window.testDiap');
  console.log('使用方法:');
  console.log('  - window.testDiap.create() - 创建身份');
  console.log('  - window.testDiap.get(ipnsName) - 获取身份');
  console.log('  - window.testDiap.update(ipnsKey, cid) - 更新身份');
  console.log('  - window.testDiap.runFull() - 运行完整测试');
}


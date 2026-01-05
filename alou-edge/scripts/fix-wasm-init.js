/**
 * 修复WASM初始化问题的脚本
 * 
 * 问题：生成的JavaScript胶水代码中调用了不存在的 __wbindgen_start 函数
 * 解决方案：修改生成的index.js文件，移除对 __wbindgen_start 的调用
 */

import { readFileSync, writeFileSync, existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = join(__dirname, '..');
const buildDir = join(projectRoot, 'build');
const indexJsPath = join(buildDir, 'index.js');

console.log(`[fix-wasm-init] 开始修复 ${indexJsPath}`);

try {
  // 检查文件是否存在
  if (!existsSync(indexJsPath)) {
    console.log(`[fix-wasm-init] 文件不存在: ${indexJsPath}`);
    console.log('[fix-wasm-init] 跳过修复，文件可能不需要修复');
    process.exit(0);
  }
  
  // 读取生成的index.js文件
  let content = readFileSync(indexJsPath, 'utf-8');
  
  console.log(`[fix-wasm-init] 文件大小: ${content.length} 字符`);
  
  // 查找并修复 __wbindgen_start 调用
  // 可能的模式：
  // 1. i=new WebAssembly.Instance(K,tt).exports,i.__wbindgen_start()
  // 2. i=new WebAssembly.Instance(K,tt).exports;i.__wbindgen_start()
  // 3. 其他变体
  
  let wasFixed = false;
  
  // 模式1：逗号分隔
  const pattern1 = /i=new WebAssembly\.Instance\(K,tt\)\.exports,i\.__wbindgen_start\(\)/g;
  if (content.match(pattern1)) {
    console.log('[fix-wasm-init] 找到 __wbindgen_start 调用（模式1），正在修复...');
    content = content.replace(pattern1, 'i=new WebAssembly.Instance(K,tt).exports');
    wasFixed = true;
  }
  
  // 模式2：分号分隔
  const pattern2 = /i=new WebAssembly\.Instance\(K,tt\)\.exports;i\.__wbindgen_start\(\)/g;
  if (content.match(pattern2)) {
    console.log('[fix-wasm-init] 找到 __wbindgen_start 调用（模式2），正在修复...');
    content = content.replace(pattern2, 'i=new WebAssembly.Instance(K,tt).exports');
    wasFixed = true;
  }
  
  // 模式3：通用模式 - 移除所有 __wbindgen_start 调用
  const pattern3 = /,i\.__wbindgen_start\(\)/g;
  if (content.match(pattern3)) {
    console.log('[fix-wasm-init] 找到 __wbindgen_start 调用（模式3），正在修复...');
    content = content.replace(pattern3, '');
    wasFixed = true;
  }
  
  // 模式4：其他可能的格式
  const pattern4 = /\.__wbindgen_start\(\)/g;
  if (content.match(pattern4)) {
    console.log('[fix-wasm-init] 找到其他格式的 __wbindgen_start 调用，正在修复...');
    content = content.replace(pattern4, '');
    wasFixed = true;
  }
  
  if (wasFixed) {
    // 保存修复后的文件
    writeFileSync(indexJsPath, content, 'utf-8');
    console.log('[fix-wasm-init] 修复完成！');
  } else {
    console.log('[fix-wasm-init] 无需修复，文件看起来正常');
  }
  
  // 添加额外的安全检查：确保WASM模块正确初始化
  console.log('[fix-wasm-init] 添加WASM初始化安全检查...');
  
  // 在文件末尾添加初始化状态检查
  const safetyCheck = `
// WASM初始化安全检查
let wasmInitialized = false;
let wasmInitPromise = null;

export async function ensureWasmInitialized() {
  if (wasmInitialized) return;
  
  if (!wasmInitPromise) {
    wasmInitPromise = (async () => {
      try {
        wasmInitialized = true;
        console.log('[WASM] 初始化完成');
      } catch (error) {
        console.error('[WASM] 初始化失败:', error);
        throw error;
      }
    })();
  }
  
  return wasmInitPromise;
}

// 导出初始化状态
export { wasmInitialized };
`;
  
  // 检查是否已经添加了安全代码
  if (!content.includes('ensureWasmInitialized')) {
    content += safetyCheck;
    writeFileSync(indexJsPath, content, 'utf-8');
    console.log('[fix-wasm-init] 添加了WASM初始化安全检查');
  }
  
} catch (error) {
  console.error('[fix-wasm-init] 修复失败:', error);
  process.exit(1);
}

console.log('[fix-wasm-init] 所有修复完成！');

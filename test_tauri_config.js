// 测试Tauri环境下的API配置
console.log('=== Tauri环境测试 ===');

// 检查环境变量
console.log('import.meta.env.DEV:', import.meta.env?.DEV);
console.log('import.meta.env.MODE:', import.meta.env?.MODE);
console.log('import.meta.env.VITE_API_BASE_URL:', import.meta.env?.VITE_API_BASE_URL);

// 检查Tauri API
console.log('window.__TAURI__:', window.__TAURI__);

// 计算当前的API_BASE_URL
const API_BASE_URL = import.meta.env.DEV ? '' : 'https://alou-edge.yuanjieliu65.workers.dev';
console.log('当前 API_BASE_URL:', API_BASE_URL);
console.log('当前 baseURL:', API_BASE_URL ? `${API_BASE_URL}/api` : '/api');

// 测试建议的修复
const isTauri = window.__TAURI__ !== undefined;
const fixedAPI_BASE_URL = isTauri ? 'http://127.0.0.1:8787' : (import.meta.env.DEV ? '' : 'https://alou-edge.yuanjieliu65.workers.dev');
console.log('\n=== 修复后的配置 ===');
console.log('isTauri:', isTauri);
console.log('修复后 API_BASE_URL:', fixedAPI_BASE_URL);
console.log('修复后 baseURL:', fixedAPI_BASE_URL ? `${fixedAPI_BASE_URL}/api` : '/api');

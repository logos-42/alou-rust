/**
 * 修复KvError显示问题的脚本
 * 将 {} 替换为 {:?} 来正确显示KvError
 */

import { readFileSync, writeFileSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = join(__dirname, '..');
const kvFilePath = join(projectRoot, 'src', 'storage', 'kv.rs');

console.log(`[fix-kv-errors] 开始修复 ${kvFilePath}`);

try {
  let content = readFileSync(kvFilePath, 'utf-8');
  
  // 修复所有KvError的显示问题
  const replacements = [
    // 修复 get 方法
    ['"Failed to get key {}: {}"', '"Failed to get key {}: {:?}"'],
    // 修复 put 方法中的创建builder错误
    ['"Failed to create put builder: {}"', '"Failed to create put builder: {:?}"'],
    // 修复 put 方法中的执行错误
    ['"Failed to put key {}: {}"', '"Failed to put key {}: {:?}"'],
    // 修复 delete 方法
    ['"Failed to delete key {}: {}"', '"Failed to delete key {}: {:?}"'],
    // 修复 check 方法
    ['"Failed to check key {}: {}"', '"Failed to check key {}: {:?}"'],
    // 修复 list 方法
    ['"Failed to list keys: {}"', '"Failed to list keys: {:?}"'],
  ];
  
  let fixedCount = 0;
  for (const [oldStr, newStr] of replacements) {
    const regex = new RegExp(oldStr.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'g');
    if (content.match(regex)) {
      content = content.replace(regex, newStr);
      fixedCount++;
      console.log(`[fix-kv-errors] 修复: ${oldStr} -> ${newStr}`);
    }
  }
  
  if (fixedCount > 0) {
    writeFileSync(kvFilePath, content, 'utf-8');
    console.log(`[fix-kv-errors] 修复了 ${fixedCount} 处错误`);
  } else {
    console.log('[fix-kv-errors] 未找到需要修复的错误');
  }
  
} catch (error) {
  console.error('[fix-kv-errors] 修复失败:', error);
  process.exit(1);
}

console.log('[fix-kv-errors] 修复完成！');

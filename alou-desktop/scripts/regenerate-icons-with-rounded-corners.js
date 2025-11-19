import sharp from 'sharp';
import { execSync } from 'child_process';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import { readdir, stat } from 'fs/promises';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const iconsDir = join(__dirname, '../src-tauri/icons');
const sourceIcon = join(iconsDir, 'icon.png');

/**
 * 为图像添加圆角
 */
async function addRoundedCorners(inputPath, radius) {
  try {
    const image = sharp(inputPath);
    const metadata = await image.metadata();
    const { width, height } = metadata;
    
    const actualRadius = Math.min(radius, Math.min(width, height) / 4);
    
    const mask = Buffer.from(
      `<svg width="${width}" height="${height}">
        <rect x="0" y="0" width="${width}" height="${height}" 
              rx="${actualRadius}" ry="${actualRadius}" fill="white"/>
      </svg>`
    );
    
    const processedBuffer = await image
      .composite([{ input: mask, blend: 'dest-in' }])
      .toBuffer();
    
    await sharp(processedBuffer).toFile(inputPath);
    
    console.log(`✓ 已处理: ${inputPath} (${width}x${height}, 圆角: ${actualRadius}px)`);
  } catch (error) {
    console.error(`✗ 处理失败: ${inputPath}`, error.message);
  }
}

/**
 * 递归查找所有 PNG 文件
 */
async function findPngFiles(dir, fileList = []) {
  const files = await readdir(dir);
  
  for (const file of files) {
    const filePath = join(dir, file);
    const fileStat = await stat(filePath);
    
    if (fileStat.isDirectory()) {
      await findPngFiles(filePath, fileList);
    } else if (file.toLowerCase().endsWith('.png')) {
      fileList.push(filePath);
    }
  }
  
  return fileList;
}

/**
 * 主函数
 */
async function main() {
  console.log('开始重新生成带圆角的图标...\n');
  
  // 步骤 1: 确保源图标有圆角
  console.log('步骤 1: 为源图标添加圆角...');
  const sourceMetadata = await sharp(sourceIcon).metadata();
  const sourceRadius = Math.max(2, Math.min(sourceMetadata.width, sourceMetadata.height) * 0.18);
  await addRoundedCorners(sourceIcon, sourceRadius);
  console.log('');
  
  // 步骤 2: 使用 Tauri CLI 重新生成所有图标（包括 .ico）
  console.log('步骤 2: 使用 Tauri CLI 重新生成图标（包括 .ico 文件）...');
  try {
    const tauriDir = join(__dirname, '../src-tauri');
    process.chdir(tauriDir);
    execSync('npx tauri icon icons/icon.png', { stdio: 'inherit' });
    console.log('✓ 图标重新生成完成\n');
  } catch (error) {
    console.error('✗ 重新生成图标失败:', error.message);
    process.exit(1);
  }
  
  // 步骤 3: 为所有生成的 PNG 图标添加圆角
  console.log('步骤 3: 为所有生成的 PNG 图标添加圆角...');
  const pngFiles = await findPngFiles(iconsDir);
  
  for (const filePath of pngFiles) {
    const metadata = await sharp(filePath).metadata();
    const { width, height } = metadata;
    const radius = Math.max(2, Math.min(width, height) * 0.18);
    await addRoundedCorners(filePath, radius);
  }
  
  console.log(`\n完成！已处理 ${pngFiles.length} 个 PNG 图标文件，并重新生成了 .ico 文件。`);
  console.log('\n提示：请重新构建应用程序以使用新的图标：');
  console.log('  npm run build:tauri');
}

main().catch(console.error);


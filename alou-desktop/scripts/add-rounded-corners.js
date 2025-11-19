import sharp from 'sharp';
import { readdir, stat } from 'fs/promises';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// 图标目录路径
const iconsDir = join(__dirname, '../src-tauri/icons');

/**
 * 递归查找所有 PNG 文件
 */
async function findPngFiles(dir, fileList = []) {
  const files = await readdir(dir);
  
  for (const file of files) {
    const filePath = join(dir, file);
    const fileStat = await stat(filePath);
    
    if (fileStat.isDirectory()) {
      // 递归查找子目录
      await findPngFiles(filePath, fileList);
    } else if (file.toLowerCase().endsWith('.png')) {
      fileList.push(filePath);
    }
  }
  
  return fileList;
}

/**
 * 为图像添加圆角
 * @param {string} inputPath - 输入文件路径
 * @param {number} radius - 圆角半径（像素）
 */
async function addRoundedCorners(inputPath, radius) {
  try {
    // 读取图像
    const image = sharp(inputPath);
    const metadata = await image.metadata();
    const { width, height } = metadata;
    
    // 如果图像太小，使用较小的圆角半径
    const actualRadius = Math.min(radius, Math.min(width, height) / 4);
    
    // 创建圆角遮罩
    const mask = Buffer.from(
      `<svg width="${width}" height="${height}">
        <rect x="0" y="0" width="${width}" height="${height}" 
              rx="${actualRadius}" ry="${actualRadius}" fill="white"/>
      </svg>`
    );
    
    // 应用圆角遮罩，先处理到缓冲区
    const processedBuffer = await image
      .composite([{ input: mask, blend: 'dest-in' }])
      .toBuffer();
    
    // 将处理后的图像写回文件
    await sharp(processedBuffer).toFile(inputPath);
    
    console.log(`✓ 已处理: ${inputPath} (${width}x${height}, 圆角: ${actualRadius}px)`);
  } catch (error) {
    console.error(`✗ 处理失败: ${inputPath}`, error.message);
  }
}

/**
 * 主函数
 */
async function main() {
  console.log('开始为所有图标添加圆角...\n');
  
  // 查找所有 PNG 文件
  const pngFiles = await findPngFiles(iconsDir);
  
  console.log(`找到 ${pngFiles.length} 个 PNG 文件\n`);
  
  // 处理每个文件
  for (const filePath of pngFiles) {
    // 读取图像尺寸以确定合适的圆角半径
    const metadata = await sharp(filePath).metadata();
    const { width, height } = metadata;
    
    // 圆角半径设置为图像尺寸的 15-20%，但至少 2px，最多不超过尺寸的 25%
    const radius = Math.max(2, Math.min(width, height) * 0.18);
    
    await addRoundedCorners(filePath, radius);
  }
  
  console.log(`\n完成！已处理 ${pngFiles.length} 个图标文件。`);
}

// 运行主函数
main().catch(console.error);


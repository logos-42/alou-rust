/**
 * 图片模糊处理工具
 * 使用 Canvas API 对图片进行模糊处理
 */

/**
 * 模糊图片
 * @param imageUrl - 图片的 data URL 或 URL
 * @param blurRadius - 模糊半径，默认 10px
 * @returns 处理后的图片 data URL
 */
export async function blurImage(imageUrl: string, blurRadius: number = 10): Promise<string> {
  return new Promise((resolve, reject) => {
    const img = new Image()
    
    img.onload = () => {
      try {
        // 创建 canvas
        const canvas = document.createElement('canvas')
        const ctx = canvas.getContext('2d')
        
        if (!ctx) {
          reject(new Error('无法获取 Canvas 上下文'))
          return
        }
        
        // 设置 canvas 尺寸
        canvas.width = img.width
        canvas.height = img.height
        
        // 检查浏览器是否支持 filter 属性
        if (ctx.filter !== undefined) {
          // 使用原生 filter 属性（性能更好）
          ctx.filter = `blur(${blurRadius}px)`
          ctx.drawImage(img, 0, 0)
        } else {
          // 降级方案：使用简单的模糊算法
          // 先绘制原图
          ctx.drawImage(img, 0, 0)
          
          // 使用简单的 box blur 算法
          const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height)
          const blurredData = boxBlur(imageData, blurRadius)
          ctx.putImageData(blurredData, 0, 0)
        }
        
        // 转换为 data URL
        const blurredDataUrl = canvas.toDataURL('image/jpeg', 0.9)
        resolve(blurredDataUrl)
      } catch (error) {
        reject(error)
      }
    }
    
    img.onerror = () => {
      reject(new Error('图片加载失败'))
    }
    
    // 设置图片源
    img.src = imageUrl
  })
}

/**
 * 简单的 Box Blur 算法（降级方案）
 * @param imageData - 原始图片数据
 * @param radius - 模糊半径
 * @returns 模糊后的图片数据
 */
function boxBlur(imageData: ImageData, radius: number): ImageData {
  const data = imageData.data
  const width = imageData.width
  const height = imageData.height
  const newData = new Uint8ClampedArray(data)
  
  // 水平模糊
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      let r = 0, g = 0, b = 0, a = 0, count = 0
      
      for (let dx = -radius; dx <= radius; dx++) {
        const px = Math.max(0, Math.min(width - 1, x + dx))
        const idx = (y * width + px) * 4
        r += data[idx]
        g += data[idx + 1]
        b += data[idx + 2]
        a += data[idx + 3]
        count++
      }
      
      const idx = (y * width + x) * 4
      newData[idx] = r / count
      newData[idx + 1] = g / count
      newData[idx + 2] = b / count
      newData[idx + 3] = a / count
    }
  }
  
  // 垂直模糊
  const tempData = new Uint8ClampedArray(newData)
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      let r = 0, g = 0, b = 0, a = 0, count = 0
      
      for (let dy = -radius; dy <= radius; dy++) {
        const py = Math.max(0, Math.min(height - 1, y + dy))
        const idx = (py * width + x) * 4
        r += tempData[idx]
        g += tempData[idx + 1]
        b += tempData[idx + 2]
        a += tempData[idx + 3]
        count++
      }
      
      const idx = (y * width + x) * 4
      newData[idx] = r / count
      newData[idx + 1] = g / count
      newData[idx + 2] = b / count
      newData[idx + 3] = a / count
    }
  }
  
  return new ImageData(newData, width, height)
}

export default blurImage

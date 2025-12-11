/**
 * 统一的降级策略管理器
 */

/**
 * 尝试多个策略，直到成功或全部失败
 * @param {Array<Function>} strategies - 策略函数数组，每个函数返回 Promise
 * @param {Object} options - 选项
 * @param {Function} options.onError - 每个策略失败时的回调 (strategyIndex, error) => void
 * @param {Function} options.shouldRetry - 判断是否应该重试 (error) => boolean
 * @returns {Promise<any>} 第一个成功策略的结果
 */
export async function tryWithFallback(strategies, options = {}) {
  const { onError, shouldRetry } = options
  const errors = []
  
  for (let i = 0; i < strategies.length; i++) {
    const strategy = strategies[i]
    
    try {
      const result = await strategy()
      
      // 如果策略返回了结果（即使可能是失败），也返回它
      // 让调用者决定如何处理
      return { success: true, result, strategyIndex: i, errors }
    } catch (error) {
      errors.push({ strategyIndex: i, error })
      
      if (onError) {
        onError(i, error)
      }
      
      // 如果 shouldRetry 返回 false，停止尝试
      if (shouldRetry && !shouldRetry(error)) {
        break
      }
      
      continue
    }
  }
  
  // 所有策略都失败了
  const lastError = errors[errors.length - 1]?.error
  throw new Error(
    `All ${strategies.length} strategies failed. Last error: ${lastError?.message || 'Unknown error'}`
  )
}

/**
 * 创建策略函数
 * @param {string} name - 策略名称
 * @param {Function} handler - 处理函数
 * @returns {Function} 策略函数
 */
export function createStrategy(name, handler) {
  return async () => {
    try {
      const result = await handler()
      return { success: true, result, strategy: name }
    } catch (error) {
      throw { strategy: name, error }
    }
  }
}


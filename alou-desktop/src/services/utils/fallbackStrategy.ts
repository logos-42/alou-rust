/**
 * 统一的降级策略管理器
 */

/**
 * 策略错误信息
 */
export interface StrategyError {
  strategyIndex: number
  error: Error
}

/**
 * 策略成功结果
 */
export interface StrategySuccessResult<T> {
  success: true
  result: T
  strategyIndex: number
  errors: StrategyError[]
}

/**
 * 降级策略选项
 */
export interface FallbackOptions {
  /** 每个策略失败时的回调 */
  onError?: (strategyIndex: number, error: Error) => void
  /** 判断是否应该重试 */
  shouldRetry?: (error: Error) => boolean
}

/**
 * 策略函数类型
 */
export type StrategyFunction<T> = () => Promise<T>

/**
 * 尝试多个策略，直到成功或全部失败
 * @param strategies - 策略函数数组，每个函数返回 Promise
 * @param options - 选项
 * @returns 第一个成功策略的结果
 */
export async function tryWithFallback<T>(
  strategies: StrategyFunction<T>[],
  options: FallbackOptions = {}
): Promise<StrategySuccessResult<T>> {
  const { onError, shouldRetry } = options
  const errors: StrategyError[] = []

  for (let i = 0; i < strategies.length; i++) {
    const strategy = strategies[i]

    try {
      const result = await strategy()

      // 如果策略返回了结果（即使可能是失败），也返回它
      // 让调用者决定如何处理
      return { success: true, result, strategyIndex: i, errors }
    } catch (error) {
      const strategyError: StrategyError = {
        strategyIndex: i,
        error: error instanceof Error ? error : new Error(String(error))
      }
      errors.push(strategyError)

      if (onError) {
        onError(i, strategyError.error)
      }

      // 如果 shouldRetry 返回 false，停止尝试
      if (shouldRetry && !shouldRetry(strategyError.error)) {
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
 * 策略执行结果
 */
export interface StrategyExecutionResult<T> {
  success: boolean
  result?: T
  strategy?: string
  error?: Error
}

/**
 * 创建策略函数
 * @param name - 策略名称
 * @param handler - 处理函数
 * @returns 策略函数
 */
export function createStrategy<T>(
  name: string,
  handler: () => Promise<T>
): () => Promise<StrategyExecutionResult<T>> {
  return async () => {
    try {
      const result = await handler()
      return { success: true, result, strategy: name }
    } catch (error) {
      throw { strategy: name, error }
    }
  }
}

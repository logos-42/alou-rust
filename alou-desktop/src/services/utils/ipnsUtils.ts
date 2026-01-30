/**
 * IPNS 工具函数
 */

/**
 * 从输入中提取 IPNS key
 * @param input - 可能包含日志消息的输入字符串
 * @returns IPNS key (k51... 或 k2...)
 */
export function extractIpnsKey(input: string): string | null {
  if (!input || typeof input !== 'string') {
    return null
  }

  // 尝试匹配 IPNS key 格式
  const match = input.match(/(k51[a-zA-Z0-9]+|k2[a-zA-Z0-9]+)/)
  return match ? match[1] : null
}

/**
 * 规范化 IPNS 名称
 * @param input - IPNS 名称或包含 IPNS 的字符串
 * @returns 规范化后的 IPNS 名称 (/ipns/k51...)
 */
export function normalizeIpns(input: string): string | null {
  if (!input || typeof input !== 'string') {
    return null
  }

  // 如果已经是 /ipns/ 格式，先提取 key
  const key = extractIpnsKey(input.replace(/^\/ipns\//, ''))
  return key ? `/ipns/${key}` : null
}

/**
 * 清理 IPNS 名称，提取纯 key
 * @param input - IPNS 名称或包含 IPNS 的字符串
 * @returns IPNS key (k51... 或 k2...)
 */
export function cleanIpnsKey(input: string): string | null {
  if (!input || typeof input !== 'string') {
    return null
  }

  // 移除 /ipns/ 前缀
  const withoutPrefix = input.replace(/^\/ipns\//, '')

  // 提取 key
  return extractIpnsKey(withoutPrefix)
}

/**
 * 判断字符串是否为 IPNS 格式
 * @param input - 输入字符串
 */
export function isIpns(input: string): boolean {
  if (!input || typeof input !== 'string') {
    return false
  }

  const trimmed = input.trim()
  // 检查是否以 /ipns/ 开头，或者包含 IPNS key 模式（k51 或 k2 开头）
  return trimmed.startsWith('/ipns/') ||
         trimmed.startsWith('ipns://') ||
         /^k51[a-zA-Z0-9]+/.test(trimmed) ||
         /^k2[a-zA-Z0-9]+/.test(trimmed) ||
         /\/ipns\/k51/.test(trimmed) ||
         /\/ipns\/k2/.test(trimmed)
}

/**
 * 判断字符串是否为 CID 格式
 * @param input - 输入字符串
 */
export function isCid(input: string): boolean {
  if (!input || typeof input !== 'string') {
    return false
  }

  return input.startsWith('Qm') ||
         input.startsWith('bafy') ||
         input.startsWith('bafk')
}

/**
 * 从输入中提取 CID
 * @param input - 可能包含日志消息的输入字符串
 * @returns CID
 */
export function extractCid(input: string): string | null {
  if (!input || typeof input !== 'string') {
    return null
  }

  const match = input.match(/(Qm[a-zA-Z0-9]+|bafy[a-zA-Z0-9]+|bafk[a-zA-Z0-9]+)/)
  return match ? match[1] : null
}

/**
 * 目标标识类型
 */
export type TargetType = 'ipns' | 'cid' | 'unknown'

/**
 * 清理后的目标标识
 */
export interface CleanedTarget {
  type: TargetType
  value: string | null
}

/**
 * 清理目标标识（IPNS 或 CID）
 * @param target - 目标标识
 */
export function cleanTarget(target: string): CleanedTarget {
  if (!target || typeof target !== 'string') {
    return { type: 'unknown', value: null }
  }

  const trimmed = target.trim()

  // 尝试提取 IPNS
  const ipnsKey = extractIpnsKey(trimmed)
  if (ipnsKey) {
    return { type: 'ipns', value: `/ipns/${ipnsKey}` }
  }

  // 尝试提取 CID
  const cid = extractCid(trimmed)
  if (cid) {
    return { type: 'cid', value: cid }
  }

  return { type: 'unknown', value: trimmed }
}

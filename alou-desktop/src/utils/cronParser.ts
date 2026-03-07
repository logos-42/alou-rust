/**
 * Cron 表达式解析工具
 * Cron Expression Parser Utility
 *
 * 提供 Cron 表达式的解析、验证和人类可读转换功能
 * Provides parsing, validation, and human-readable conversion for Cron expressions
 */

import type { ParsedCronExpression, HumanReadableCron } from '@shared/types/cron';

/**
 * Cron 字段范围定义
 * Cron field range definitions
 */
const CRON_FIELD_RANGES = {
  minute: { min: 0, max: 59 },
  hour: { min: 0, max: 23 },
  dayOfMonth: { min: 1, max: 31 },
  month: { min: 1, max: 12 },
  dayOfWeek: { min: 0, max: 6 },
};

/**
 * 星期名称映射
 * Day of week name mapping
 */
const DAY_NAMES = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];
const DAY_NAMES_CN = ['周日', '周一', '周二', '周三', '周四', '周五', '周六'];

/**
 * 月份名称映射
 * Month name mapping
 */
const MONTH_NAMES = [
  'January', 'February', 'March', 'April', 'May', 'June',
  'July', 'August', 'September', 'October', 'November', 'December'
];
const MONTH_NAMES_CN = [
  '一月', '二月', '三月', '四月', '五月', '六月',
  '七月', '八月', '九月', '十月', '十一月', '十二月'
];

/**
 * 解析单个 Cron 字段值
 * Parse a single Cron field value
 */
function parseFieldValue(value: string, fieldName: string): number | '*' {
  if (value === '*') {
    return '*';
  }

  const range = CRON_FIELD_RANGES[fieldName as keyof typeof CRON_FIELD_RANGES];
  const num = parseInt(value, 10);

  if (isNaN(num)) {
    throw new Error(`Invalid ${fieldName} value: ${value}`);
  }

  if (num < range.min || num > range.max) {
    throw new Error(`${fieldName} value ${num} is out of range (${range.min}-${range.max})`);
  }

  return num;
}

/**
 * 解析 Cron 表达式
 * Parse a Cron expression
 *
 * @param expression Cron 表达式（格式：分 时 日 月 星期）
 * @returns 解析结果
 */
export function parseCronExpression(expression: string): ParsedCronExpression {
  const parts = expression.trim().split(/\s+/);

  // 验证字段数量
  if (parts.length !== 5) {
    return {
      minute: '*',
      hour: '*',
      dayOfMonth: '*',
      month: '*',
      dayOfWeek: '*',
      original: expression,
      isValid: false,
      error: `Invalid Cron expression: expected 5 fields, got ${parts.length}`,
    };
  }

  try {
    const minute = parseFieldValue(parts[0], 'minute');
    const hour = parseFieldValue(parts[1], 'hour');
    const dayOfMonth = parseFieldValue(parts[2], 'dayOfMonth');
    const month = parseFieldValue(parts[3], 'month');
    const dayOfWeek = parseFieldValue(parts[4], 'dayOfWeek');

    return {
      minute,
      hour,
      dayOfMonth,
      month,
      dayOfWeek,
      original: expression,
      isValid: true,
    };
  } catch (error: any) {
    return {
      minute: '*',
      hour: '*',
      dayOfMonth: '*',
      month: '*',
      dayOfWeek: '*',
      original: expression,
      isValid: false,
      error: error.message,
    };
  }
}

/**
 * 计算下次运行时间
 * Calculate the next run time
 *
 * @param expression Cron 表达式
 * @param fromDate 起始时间（默认为当前时间）
 * @returns 下次运行时间字符串
 */
export function getNextRunTime(expression: string, fromDate: Date = new Date()): string {
  const parsed = parseCronExpression(expression);

  if (!parsed.isValid) {
    return 'Invalid Cron expression';
  }

  // 创建下次运行时间的副本
  const next = new Date(fromDate);
  next.setSeconds(0);
  next.setMilliseconds(0);

  // 最多查找一年内的匹配时间
  const maxIterations = 366 * 24 * 60; // 一年的分钟数
  let iterations = 0;

  while (iterations < maxIterations) {
    next.setMinutes(next.getMinutes() + 1);

    // 检查月份
    if (parsed.month !== '*' && next.getMonth() + 1 !== parsed.month) {
      continue;
    }

    // 检查日期和星期（Cron 中日期和星期是 OR 关系）
    const dayOfMonthMatch = parsed.dayOfMonth === '*' || next.getDate() === parsed.dayOfMonth;
    const dayOfWeekMatch = parsed.dayOfWeek === '*' || next.getDay() === parsed.dayOfWeek;

    // 如果日期和星期都指定了具体值，满足任一即可（OR 关系）
    // 如果只指定了一个，必须匹配该值
    let dayMatch: boolean;
    if (parsed.dayOfMonth !== '*' && parsed.dayOfWeek !== '*') {
      dayMatch = dayOfMonthMatch || dayOfWeekMatch;
    } else if (parsed.dayOfMonth !== '*') {
      dayMatch = dayOfMonthMatch;
    } else if (parsed.dayOfWeek !== '*') {
      dayMatch = dayOfWeekMatch;
    } else {
      dayMatch = true;
    }

    if (!dayMatch) {
      continue;
    }

    // 检查小时
    if (parsed.hour !== '*' && next.getHours() !== parsed.hour) {
      continue;
    }

    // 检查分钟
    if (parsed.minute !== '*' && next.getMinutes() !== parsed.minute) {
      continue;
    }

    // 所有条件都匹配
    return next.toISOString();
  }

  return 'No matching time found within one year';
}

/**
 * 将 Cron 表达式转换为人类可读文本
 * Convert Cron expression to human-readable text
 *
 * @param expression Cron 表达式
 * @param locale 语言环境 ('en' | 'zh')
 * @returns 人类可读描述
 */
export function getHumanReadable(expression: string, locale: 'en' | 'zh' = 'zh'): HumanReadableCron {
  const parsed = parseCronExpression(expression);

  if (!parsed.isValid) {
    return {
      short: 'Invalid Cron expression',
      long: parsed.error || 'Invalid Cron expression',
      nextRun: 'N/A',
    };
  }

  const { minute, hour, dayOfMonth, month, dayOfWeek } = parsed;

  // 构建简短描述
  let short = '';
  let long = '';

  // 检查是否是常见模式
  const isEveryMinute = minute === '*' && hour === '*';
  const isEveryHour = minute !== '*' && hour === '*';
  const isDaily = minute !== '*' && hour !== '*' && dayOfMonth === '*' && month === '*' && dayOfWeek === '*';
  const isWeekly = dayOfWeek !== '*' && dayOfMonth === '*';
  const isMonthly = dayOfMonth !== '*' && month === '*' && dayOfWeek === '*';

  if (isEveryMinute) {
    short = locale === 'zh' ? '每分钟' : 'Every minute';
    long = locale === 'zh' ? '每分钟执行一次' : 'Executes every minute';
  } else if (isEveryHour) {
    const minuteStr = minute as number;
    short = locale === 'zh' ? `每小时第 ${minuteStr} 分钟` : `At ${minuteStr} minutes past the hour`;
    long = locale === 'zh'
      ? `每小时第 ${minuteStr} 分钟执行一次`
      : `Executes at ${minuteStr} minutes past every hour`;
  } else if (isDaily) {
    const timeStr = formatTime(hour as number, minute as number, locale);
    short = locale === 'zh' ? `每天 ${timeStr}` : `Daily at ${timeStr}`;
    long = locale === 'zh'
      ? `每天 ${timeStr} 执行一次`
      : `Executes once a day at ${timeStr}`;
  } else if (isWeekly) {
    const timeStr = formatTime(hour as number, minute as number, locale);
    const dayName = locale === 'zh' ? DAY_NAMES_CN[dayOfWeek as number] : DAY_NAMES[dayOfWeek as number];
    short = locale === 'zh' ? `每周${dayName} ${timeStr}` : `${dayName}s at ${timeStr}`;
    long = locale === 'zh'
      ? `每周${dayName} ${timeStr} 执行一次`
      : `Executes once a week on ${dayName} at ${timeStr}`;
  } else if (isMonthly) {
    const timeStr = formatTime(hour as number, minute as number, locale);
    short = locale === 'zh' ? `每月 ${dayOfMonth} 日 ${timeStr}` : `Monthly on the ${dayOfMonth} at ${timeStr}`;
    long = locale === 'zh'
      ? `每月 ${dayOfMonth} 日 ${timeStr} 执行一次`
      : `Executes once a month on the ${dayOfMonth} at ${timeStr}`;
  } else {
    // 自定义模式
    const parts: string[] = [];

    if (minute !== '*') {
      parts.push(locale === 'zh' ? `第 ${minute} 分钟` : `${minute} minute`);
    }
    if (hour !== '*') {
      parts.push(locale === 'zh' ? `${hour} 点` : `${hour}:00`);
    }
    if (dayOfMonth !== '*') {
      parts.push(locale === 'zh' ? `${dayOfMonth} 日` : `day ${dayOfMonth}`);
    }
    if (month !== '*') {
      const monthName = locale === 'zh' ? MONTH_NAMES_CN[month - 1] : MONTH_NAMES[month - 1];
      parts.push(monthName);
    }
    if (dayOfWeek !== '*') {
      const dayName = locale === 'zh' ? DAY_NAMES_CN[dayOfWeek] : DAY_NAMES[dayOfWeek];
      parts.push(locale === 'zh' ? dayName : dayName);
    }

    short = parts.join(' ');
    long = locale === 'zh'
      ? `在 ${parts.join('、')} 执行`
      : `Executes at ${parts.join(', ')}`;
  }

  // 计算下次运行时间
  const nextRun = getNextRunTime(expression);
  const nextRunDate = new Date(nextRun);
  const nextRunStr = locale === 'zh'
    ? nextRunDate.toLocaleString('zh-CN', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit'
      })
    : nextRunDate.toLocaleString('en-US', {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit'
      });

  return {
    short,
    long,
    nextRun: nextRunStr,
  };
}

/**
 * 格式化时间
 * Format time
 */
function formatTime(hour: number, minute: number, locale: 'en' | 'zh'): string {
  if (locale === 'zh') {
    return `${hour.toString().padStart(2, '0')}:${minute.toString().padStart(2, '0')}`;
  } else {
    const period = hour >= 12 ? 'PM' : 'AM';
    const displayHour = hour % 12 || 12;
    return `${displayHour}:${minute.toString().padStart(2, '0')} ${period}`;
  }
}

/**
 * 验证 Cron 表达式是否有效
 * Validate if a Cron expression is valid
 *
 * @param expression Cron 表达式
 * @returns 是否有效
 */
export function isValidCronExpression(expression: string): boolean {
  const parsed = parseCronExpression(expression);
  return parsed.isValid;
}

/**
 * 获取 Cron 表达式的验证错误信息
 * Get validation error message for a Cron expression
 *
 * @param expression Cron 表达式
 * @returns 错误信息（如果有效则返回 null）
 */
export function getCronValidationError(expression: string): string | null {
  const parsed = parseCronExpression(expression);
  return parsed.error || null;
}

/**
 * 生成示例 Cron 表达式
 * Generate example Cron expressions
 *
 * @returns 示例表达式列表
 */
export function getCronExamples(): Array<{ label: string; expression: string; description: string }> {
  return [
    {
      label: '每天 8:00',
      expression: '0 8 * * *',
      description: '每天早上 8 点执行',
    },
    {
      label: '每天 2:00',
      expression: '0 2 * * *',
      description: '每天凌晨 2 点执行',
    },
    {
      label: '每周一 9:00',
      expression: '0 9 * * 1',
      description: '每周一早上 9 点执行',
    },
    {
      label: '每小时',
      expression: '0 * * * *',
      description: '每小时整点执行',
    },
    {
      label: '每 30 分钟',
      expression: '*/30 * * * *',
      description: '每 30 分钟执行一次',
    },
    {
      label: '每月 1 号',
      expression: '0 0 1 * *',
      description: '每月 1 号凌晨执行',
    },
  ];
}

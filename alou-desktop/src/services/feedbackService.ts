/**
 * 用户反馈服务
 * 
 * 收集用户对 AI 执行结果的反馈
 * 用于学习和改进系统
 */

import { invoke } from '@tauri-apps/api/core';

// ============ 类型定义 ============

/**
 * 反馈类型
 */
export type FeedbackType = 
  | 'task_completion'   // 任务完成度
  | 'response_quality'   // 回复质量
  | 'skill_selection'   // 技能选择
  | 'execution_result'   // 执行结果
  | 'general';          // 一般反馈

/**
 * 反馈评分 (1-5)
 */
export interface FeedbackRating {
  score: number;        // 1-5
  comment?: string;       // 可选评论
  suggestions?: string[]; // 改进建议
}

/**
 * 反馈记录
 */
export interface FeedbackRecord {
  id: string;
  timestamp: number;
  type: FeedbackType;
  task: string;
  skill_used: string;
  rating: FeedbackRating;
  outcome: 'success' | 'partial' | 'failed';
  context: Record<string, unknown>;
}

/**
 * 反馈统计
 */
export interface FeedbackStats {
  total_feedbacks: number;
  average_score: number;
  by_type: Record<string, {
    count: number;
    avg_score: number;
  }>;
  by_skill: Record<string, {
    count: number;
    avg_score: number;
  }>;
  improvement_suggestions: string[];
}

// ============ 反馈服务 ============

class FeedbackService {
  private static instance: FeedbackService;
  private storageKey = 'alou_feedback_history';

  private constructor() {}

  static getInstance(): FeedbackService {
    if (!FeedbackService.instance) {
      FeedbackService.instance = new FeedbackService();
    }
    return FeedbackService.instance;
  }

  // ============ 收集反馈 ============

  /**
   * 提交反馈
   */
  async submitFeedback(
    type: FeedbackType,
    task: string,
    skill_used: string,
    rating: FeedbackRating,
    outcome: 'success' | 'partial' | 'failed',
    context: Record<string, unknown> = {}
  ): Promise<FeedbackRecord> {
    const feedback: FeedbackRecord = {
      id: `feedback_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      timestamp: Date.now(),
      type,
      task,
      skill_used,
      rating,
      outcome,
      context,
    };

    // 保存到本地
    await this.saveFeedback(feedback);

    // 发送到后端（如果连接）
    try {
      await invoke('record_feedback', {
        feedback: {
          ...feedback,
          rating: feedback.rating.score,
        },
      });
    } catch (e) {
      console.log('反馈已保存到本地');
    }

    console.log(`\n✅ 反馈已提交: ${feedback.id}`);
    console.log(`   评分: ${'⭐'.repeat(feedback.rating.score)}${'☆'.repeat(5 - feedback.rating.score)} (${feedback.rating.score}/5)`);

    return feedback;
  }

  /**
   * 快速反馈（简化版）
   */
  async quickFeedback(
    task: string,
    skill: string,
    isGood: boolean,
    comment?: string
  ): Promise<void> {
    await this.submitFeedback(
      'execution_result',
      task,
      skill,
      {
        score: isGood ? 5 : 1,
        comment,
      },
      isGood ? 'success' : 'failed'
    );
  }

  // ============ 查询反馈 ============

  /**
   * 获取所有反馈
   */
  async getAllFeedback(): Promise<FeedbackRecord[]> {
    try {
      const data = localStorage.getItem(this.storageKey);
      return data ? JSON.parse(data) : [];
    } catch {
      return [];
    }
  }

  /**
   * 获取最近反馈
   */
  async getRecentFeedback(limit: number = 10): Promise<FeedbackRecord[]> {
    const all = await this.getAllFeedback();
    return all
      .sort((a, b) => b.timestamp - a.timestamp)
      .slice(0, limit);
  }

  /**
   * 获取反馈统计
   */
  async getStats(): Promise<FeedbackStats> {
    const feedbacks = await this.getAllFeedback();

    if (feedbacks.length === 0) {
      return {
        total_feedbacks: 0,
        average_score: 0,
        by_type: {},
        by_skill: {},
        improvement_suggestions: [],
      };
    }

    // 计算统计
    const totalScore = feedbacks.reduce((sum, f) => sum + f.rating.score, 0);
    const avgScore = totalScore / feedbacks.length;

    const byType: Record<string, { count: number; totalScore: number }> = {};
    const bySkill: Record<string, { count: number; totalScore: number }> = {};

    for (const f of feedbacks) {
      // 按类型统计
      if (!byType[f.type]) {
        byType[f.type] = { count: 0, totalScore: 0 };
      }
      byType[f.type].count++;
      byType[f.type].totalScore += f.rating.score;

      // 按技能统计
      if (!bySkill[f.skill_used]) {
        bySkill[f.skill_used] = { count: 0, totalScore: 0 };
      }
      bySkill[f.skill_used].count++;
      bySkill[f.skill_used].totalScore += f.rating.score;
    }

    // 收集改进建议
    const suggestions = new Set<string>();
    for (const f of feedbacks) {
      if (f.rating.score <= 2 && f.rating.suggestions) {
        f.rating.suggestions.forEach(s => suggestions.add(s));
      }
    }

    return {
      total_feedbacks: feedbacks.length,
      average_score: Math.round(avgScore * 10) / 10,
      by_type: Object.fromEntries(
        Object.entries(byType).map(([k, v]) => [
          k,
          { count: v.count, avg_score: Math.round(v.totalScore / v.count * 10) / 10 },
        ])
      ),
      by_skill: Object.fromEntries(
        Object.entries(bySkill).map(([k, v]) => [
          k,
          { count: v.count, avg_score: Math.round(v.totalScore / v.count * 10) / 10 },
        ])
      ),
      improvement_suggestions: Array.from(suggestions).slice(0, 5),
    };
  }

  // ============ 内部方法 ============

  private async saveFeedback(feedback: FeedbackRecord): Promise<void> {
    try {
      const feedbacks = await this.getAllFeedback();
      feedbacks.push(feedback);

      // 只保留最近 500 条
      if (feedbacks.length > 500) {
        feedbacks.shift();
      }

      localStorage.setItem(this.storageKey, JSON.stringify(feedbacks));
    } catch (error) {
      console.error('保存反馈失败:', error);
    }
  }
}

// ============ 导出 ============

export const feedbackService = FeedbackService.getInstance();

export default feedbackService;

// CLI 测试
if (require.main === module) {
  console.log('📝 反馈服务测试\n');

  feedbackService.getStats().then(stats => {
    console.log('📊 反馈统计:');
    console.log(JSON.stringify(stats, null, 2));
  });
}

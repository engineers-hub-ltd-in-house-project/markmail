import { authStore } from "$lib/stores/authStore";
import { get } from "svelte/store";
import type {
  GenerateScenarioRequest,
  GenerateScenarioResponse,
  GenerateContentRequest,
  GenerateContentResponse,
  OptimizeSubjectRequest,
  OptimizeSubjectResponse,
} from "$lib/types/ai";

const API_BASE_URL = "/api";

class ApiError extends Error {
  constructor(
    public status: number,
    message: string,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

async function apiRequest<T>(
  path: string,
  options: RequestInit = {},
): Promise<T> {
  const auth = get(authStore);
  const url = `${API_BASE_URL}${path}`;

  const response = await fetch(url, {
    ...options,
    headers: {
      "Content-Type": "application/json",
      ...(auth.token && { Authorization: `Bearer ${auth.token}` }),
      ...options.headers,
    },
  });

  if (!response.ok) {
    if (response.status === 401) {
      authStore.logout();
      throw new ApiError(401, "認証が必要です");
    }

    const errorText = await response.text();
    let errorMessage = "エラーが発生しました";

    try {
      const errorJson = JSON.parse(errorText);
      errorMessage = errorJson.message || errorJson.error || errorMessage;
    } catch {
      errorMessage = errorText || errorMessage;
    }

    throw new ApiError(response.status, errorMessage);
  }

  return response.json();
}

export const aiApi = {
  // シナリオ生成
  async generateScenario(
    request: GenerateScenarioRequest,
  ): Promise<GenerateScenarioResponse> {
    return apiRequest<GenerateScenarioResponse>("/ai/scenarios/generate", {
      method: "POST",
      body: JSON.stringify(request),
    });
  },

  // コンテンツ生成
  async generateContent(
    request: GenerateContentRequest,
  ): Promise<GenerateContentResponse> {
    return apiRequest<GenerateContentResponse>("/ai/content/generate", {
      method: "POST",
      body: JSON.stringify(request),
    });
  },

  // 件名最適化
  async optimizeSubject(
    request: OptimizeSubjectRequest,
  ): Promise<OptimizeSubjectResponse> {
    return apiRequest<OptimizeSubjectResponse>("/ai/content/optimize-subject", {
      method: "POST",
      body: JSON.stringify(request),
    });
  },
};

// ヘルパー関数

/**
 * コンテンツトーンの日本語表示
 */
export function getContentToneLabel(tone: string): string {
  const toneLabels: Record<string, string> = {
    formal: "フォーマル",
    casual: "カジュアル",
    professional: "プロフェッショナル",
    friendly: "フレンドリー",
    urgent: "緊急",
  };
  return toneLabels[tone] || tone;
}

/**
 * ステップタイプの日本語表示
 */
export function getStepTypeLabel(type: string): string {
  const stepLabels: Record<string, string> = {
    email: "メール送信",
    wait: "待機",
    condition: "条件分岐",
    tag: "タグ付け",
  };
  return stepLabels[type] || type;
}

/**
 * トリガータイプの日本語表示
 */
export function getTriggerTypeLabel(type: string): string {
  const triggerLabels: Record<string, string> = {
    manual: "手動",
    subscriber_created: "購読者登録時",
    form_submission: "フォーム送信時",
    tag_added: "タグ追加時",
  };
  return triggerLabels[type] || type;
}

/**
 * 遅延単位の日本語表示
 */
export function getDelayUnitLabel(unit: string): string {
  const unitLabels: Record<string, string> = {
    minutes: "分",
    hours: "時間",
    days: "日",
  };
  return unitLabels[unit] || unit;
}

/**
 * 開封率のパーセンテージ表示
 */
export function formatOpenRate(rate: number): string {
  return `${(rate * 100).toFixed(1)}%`;
}

/**
 * 読了時間の表示
 */
export function formatReadingTime(seconds: number): string {
  if (seconds < 60) {
    return `${seconds}秒`;
  }
  const minutes = Math.ceil(seconds / 60);
  return `${minutes}分`;
}

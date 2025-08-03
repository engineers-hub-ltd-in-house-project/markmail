import { authStore } from "../stores/authStore";
import type {
  SubscriptionPlan,
  PlansResponse,
  SubscriptionDetailsResponse,
  PaymentHistory,
  UpgradeRequest,
  CancelRequest,
  UsageSummary,
  UserSubscription,
} from "../types/subscription";

const API_BASE_URL = "/api";

// 認証トークンを取得する
const getAuthToken = (): string | null => {
  let token: string | null = null;
  authStore.subscribe((state) => {
    token = state.token;
  })();
  return token;
};

// APIリクエストを実行
async function fetchAPI<T = any>(
  path: string,
  options: RequestInit = {},
): Promise<T> {
  const token = getAuthToken();
  const headers = new Headers(options.headers || {});

  headers.set("Content-Type", "application/json");
  if (token) {
    headers.set("Authorization", `Bearer ${token}`);
  }

  const response = await fetch(`${API_BASE_URL}${path}`, {
    ...options,
    headers,
  });

  if (!response.ok) {
    if (response.status === 401) {
      // 認証エラーの場合はログアウト
      authStore.logout();
    }

    let errorMessage = `API Error: ${response.status}`;
    try {
      const error = await response.json();
      errorMessage = error.error || error.message || errorMessage;
    } catch {
      // JSON解析エラーは無視
    }

    throw new Error(errorMessage);
  }

  return response.json();
}

export const subscriptionService = {
  // 全てのプランを取得
  async getPlans(): Promise<PlansResponse> {
    return fetchAPI<PlansResponse>("/subscriptions/plans");
  },

  // 現在のサブスクリプション情報を取得
  async getCurrentSubscription(): Promise<SubscriptionDetailsResponse> {
    return fetchAPI<SubscriptionDetailsResponse>("/subscriptions/current");
  },

  // プランをアップグレード
  async upgradePlan(planId: string): Promise<UserSubscription> {
    const request: UpgradeRequest = { plan_id: planId };
    return fetchAPI<UserSubscription>("/subscriptions/upgrade", {
      method: "POST",
      body: JSON.stringify(request),
    });
  },

  // サブスクリプションをキャンセル
  async cancelSubscription(
    cancelAtPeriodEnd: boolean = true,
  ): Promise<UserSubscription> {
    const request: CancelRequest = { cancel_at_period_end: cancelAtPeriodEnd };
    return fetchAPI<UserSubscription>("/subscriptions/cancel", {
      method: "POST",
      body: JSON.stringify(request),
    });
  },

  // 支払い履歴を取得
  async getPaymentHistory(
    limit: number = 50,
    offset: number = 0,
  ): Promise<{
    payments: PaymentHistory[];
    limit: number;
    offset: number;
  }> {
    return fetchAPI(
      `/subscriptions/payment-history?limit=${limit}&offset=${offset}`,
    );
  },

  // 使用量を取得
  async getUsage(): Promise<UsageSummary> {
    return fetchAPI<UsageSummary>("/subscriptions/usage");
  },

  // Stripe Checkoutセッションを作成してリダイレクト
  async createCheckoutSession(
    planId: string,
    successUrl?: string,
    cancelUrl?: string,
  ): Promise<{ checkout_url: string }> {
    const request = {
      plan_id: planId,
      success_url:
        successUrl || `${window.location.origin}/subscriptions/success`,
      cancel_url: cancelUrl || `${window.location.origin}/subscriptions`,
    };

    const response = await fetchAPI<{ checkout_url: string }>(
      "/subscriptions/checkout",
      {
        method: "POST",
        body: JSON.stringify(request),
      },
    );

    // Stripe Checkoutへリダイレクト
    if (response.checkout_url) {
      window.location.href = response.checkout_url;
    }

    return response;
  },

  // AI使用量統計を取得
  async getAIUsageStats(): Promise<{
    total_usage: number;
    scenario_usage: number;
    content_usage: number;
    subject_usage: number;
  }> {
    return fetchAPI("/ai/usage/stats");
  },

  // AI使用履歴を取得
  async getAIUsageHistory(
    limit: number = 50,
    offset: number = 0,
  ): Promise<{
    usage_logs: Array<{
      id: string;
      user_id: string;
      feature_type: string;
      prompt?: string;
      response?: string;
      tokens_used?: number;
      model_used?: string;
      created_at: string;
    }>;
    total: number;
    limit: number;
    offset: number;
  }> {
    return fetchAPI(`/ai/usage/history?limit=${limit}&offset=${offset}`);
  },
};

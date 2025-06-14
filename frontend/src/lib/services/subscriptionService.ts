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
};

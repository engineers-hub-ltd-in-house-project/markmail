import { get } from "svelte/store";
import { authStore } from "../stores/authStore";
import type {
  Subscriber,
  SubscriberListResponse,
  CreateSubscriberRequest,
  UpdateSubscriberRequest,
  ImportSubscribersRequest,
  ImportSubscribersResponse,
} from "../types/subscriber";

/**
 * API エラーレスポンス
 */
class ApiError extends Error {
  statusCode: number;

  constructor(message: string, statusCode: number) {
    super(message);
    this.statusCode = statusCode;
    this.name = "ApiError";
  }
}

/**
 * 購読者管理APIサービス
 */
export const subscriberService = {
  /**
   * 認証付きリクエストヘッダーを作成
   */
  getAuthHeaders(): HeadersInit {
    const { token } = get(authStore);
    if (!token) throw new Error("認証が必要です");

    return {
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
    };
  },

  /**
   * 購読者一覧を取得
   */
  async listSubscribers(
    limit = 50,
    offset = 0,
    search?: string,
    tag?: string,
    status?: string,
    sortBy = "created_at",
    sortOrder = "DESC",
  ): Promise<SubscriberListResponse> {
    const params = new URLSearchParams();
    params.append("limit", limit.toString());
    params.append("offset", offset.toString());
    if (search) params.append("search", search);
    if (tag) params.append("tag", tag);
    if (status) params.append("status", status);
    params.append("sort_by", sortBy);
    params.append("sort_order", sortOrder);

    const response = await fetch(`/api/subscribers?${params.toString()}`, {
      method: "GET",
      headers: this.getAuthHeaders(),
    });

    if (!response.ok) {
      const errorData = await response.json();
      throw new ApiError(
        errorData.error || "購読者一覧の取得に失敗しました",
        response.status,
      );
    }

    return await response.json();
  },

  /**
   * 購読者を作成
   */
  async createSubscriber(
    subscriberData: CreateSubscriberRequest,
  ): Promise<Subscriber> {
    const response = await fetch("/api/subscribers", {
      method: "POST",
      headers: this.getAuthHeaders(),
      body: JSON.stringify(subscriberData),
    });

    if (!response.ok) {
      const errorData = await response.json();
      throw new ApiError(
        errorData.error || "購読者の作成に失敗しました",
        response.status,
      );
    }

    return await response.json();
  },

  /**
   * 購読者詳細を取得
   */
  async getSubscriber(id: string): Promise<Subscriber> {
    const response = await fetch(`/api/subscribers/${id}`, {
      method: "GET",
      headers: this.getAuthHeaders(),
    });

    if (!response.ok) {
      const errorData = await response.json();
      throw new ApiError(
        errorData.error || "購読者の取得に失敗しました",
        response.status,
      );
    }

    return await response.json();
  },

  /**
   * 購読者を更新
   */
  async updateSubscriber(
    id: string,
    subscriberData: UpdateSubscriberRequest,
  ): Promise<Subscriber> {
    const response = await fetch(`/api/subscribers/${id}`, {
      method: "PUT",
      headers: this.getAuthHeaders(),
      body: JSON.stringify(subscriberData),
    });

    if (!response.ok) {
      const errorData = await response.json();
      throw new ApiError(
        errorData.error || "購読者の更新に失敗しました",
        response.status,
      );
    }

    return await response.json();
  },

  /**
   * 購読者を削除
   */
  async deleteSubscriber(id: string): Promise<{ message: string }> {
    const response = await fetch(`/api/subscribers/${id}`, {
      method: "DELETE",
      headers: this.getAuthHeaders(),
    });

    if (!response.ok) {
      const errorData = await response.json();
      throw new ApiError(
        errorData.error || "購読者の削除に失敗しました",
        response.status,
      );
    }

    return await response.json();
  },

  /**
   * CSVファイルから購読者を一括インポート
   */
  async importSubscribers(
    importData: ImportSubscribersRequest,
  ): Promise<ImportSubscribersResponse> {
    const formData = new FormData();
    formData.append("file", importData.file);
    if (importData.tag) {
      formData.append("tag", importData.tag);
    }

    const { token } = get(authStore);
    if (!token) throw new Error("認証が必要です");

    const response = await fetch("/api/subscribers/import", {
      method: "POST",
      headers: {
        Authorization: `Bearer ${token}`,
      },
      body: formData,
    });

    if (!response.ok) {
      const errorData = await response.json();
      throw new ApiError(
        errorData.error || "購読者のインポートに失敗しました",
        response.status,
      );
    }

    return await response.json();
  },

  /**
   * 購読者のタグを取得
   */
  async getSubscriberTags(): Promise<string[]> {
    const response = await fetch("/api/subscribers/tags", {
      method: "GET",
      headers: this.getAuthHeaders(),
    });

    if (!response.ok) {
      const errorData = await response.json();
      throw new ApiError(
        errorData.error || "購読者タグの取得に失敗しました",
        response.status,
      );
    }

    const data = await response.json();
    return data.tags;
  },
};

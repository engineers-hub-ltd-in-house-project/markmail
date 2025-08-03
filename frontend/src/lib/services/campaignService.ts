import { get } from "svelte/store";
import { authStore } from "../stores/authStore";
import type {
  Campaign,
  CampaignListResponse,
  CreateCampaignRequest,
  PreviewCampaignResponse,
  ScheduleCampaignRequest,
  UpdateCampaignRequest,
} from "../types/campaign";
import type { Subscriber } from "../types/subscriber";

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
 * キャンペーン管理APIサービス
 */
export const campaignService = {
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
   * キャンペーン一覧を取得
   */
  async listCampaigns(
    limit = 50,
    offset = 0,
    status?: string,
    sortBy = "created_at",
    sortOrder = "DESC",
  ): Promise<CampaignListResponse> {
    // クエリパラメータの構築
    const params = new URLSearchParams();
    params.append("limit", limit.toString());
    params.append("offset", offset.toString());
    if (status) params.append("status", status);
    params.append("sort_by", sortBy);
    params.append("sort_order", sortOrder);

    const response = await fetch(`/api/campaigns?${params.toString()}`, {
      method: "GET",
      headers: this.getAuthHeaders(),
    });

    if (!response.ok) {
      const errorData = await response.json();
      throw new ApiError(
        errorData.error || "キャンペーン一覧の取得に失敗しました",
        response.status,
      );
    }

    return await response.json();
  },

  /**
   * キャンペーンを作成
   */
  async createCampaign(campaignData: CreateCampaignRequest): Promise<Campaign> {
    const response = await fetch("/api/campaigns", {
      method: "POST",
      headers: this.getAuthHeaders(),
      body: JSON.stringify(campaignData),
    });

    if (!response.ok) {
      const errorData = await response.json();
      throw new ApiError(
        errorData.error || "キャンペーンの作成に失敗しました",
        response.status,
      );
    }

    return await response.json();
  },

  /**
   * キャンペーン詳細を取得
   */
  async getCampaign(id: string): Promise<Campaign> {
    const response = await fetch(`/api/campaigns/${id}`, {
      method: "GET",
      headers: this.getAuthHeaders(),
    });

    if (!response.ok) {
      const errorData = await response.json();
      throw new ApiError(
        errorData.error || "キャンペーンの取得に失敗しました",
        response.status,
      );
    }

    return await response.json();
  },

  /**
   * キャンペーンを更新
   */
  async updateCampaign(
    id: string,
    campaignData: UpdateCampaignRequest,
  ): Promise<Campaign> {
    const response = await fetch(`/api/campaigns/${id}`, {
      method: "PUT",
      headers: this.getAuthHeaders(),
      body: JSON.stringify(campaignData),
    });

    if (!response.ok) {
      const errorData = await response.json();
      throw new ApiError(
        errorData.error || "キャンペーンの更新に失敗しました",
        response.status,
      );
    }

    return await response.json();
  },

  /**
   * キャンペーンを削除
   */
  async deleteCampaign(id: string): Promise<{ message: string }> {
    const response = await fetch(`/api/campaigns/${id}`, {
      method: "DELETE",
      headers: this.getAuthHeaders(),
    });

    if (!response.ok) {
      const errorData = await response.json();
      throw new ApiError(
        errorData.error || "キャンペーンの削除に失敗しました",
        response.status,
      );
    }

    return await response.json();
  },

  /**
   * キャンペーンをスケジュール
   */
  async scheduleCampaign(
    id: string,
    scheduleData: ScheduleCampaignRequest,
  ): Promise<Campaign> {
    const response = await fetch(`/api/campaigns/${id}/schedule`, {
      method: "POST",
      headers: this.getAuthHeaders(),
      body: JSON.stringify(scheduleData),
    });

    if (!response.ok) {
      const errorData = await response.json();
      throw new ApiError(
        errorData.error || "キャンペーンのスケジュールに失敗しました",
        response.status,
      );
    }

    return await response.json();
  },

  /**
   * キャンペーンを送信
   */
  async sendCampaign(id: string): Promise<{ message: string }> {
    const response = await fetch(`/api/campaigns/${id}/send`, {
      method: "POST",
      headers: this.getAuthHeaders(),
    });

    if (!response.ok) {
      let errorMessage = "キャンペーンの送信に失敗しました";
      try {
        const errorData = await response.json();
        errorMessage = errorData.error || errorMessage;
      } catch {
        // JSONパースエラーの場合はデフォルトメッセージを使用
      }
      throw new ApiError(errorMessage, response.status);
    }

    // レスポンスボディを安全にパース
    const text = await response.text();
    if (!text) {
      return { message: "キャンペーンの送信を開始しました" };
    }
    try {
      return JSON.parse(text);
    } catch {
      return { message: "キャンペーンの送信を開始しました" };
    }
  },

  /**
   * キャンペーンのプレビューを取得
   */
  async previewCampaign(id: string): Promise<PreviewCampaignResponse> {
    const response = await fetch(`/api/campaigns/${id}/preview`, {
      method: "GET",
      headers: this.getAuthHeaders(),
    });

    if (!response.ok) {
      const errorData = await response.json();
      throw new ApiError(
        errorData.error || "キャンペーンのプレビュー取得に失敗しました",
        response.status,
      );
    }

    return await response.json();
  },

  /**
   * キャンペーンの購読者を取得
   */
  async getCampaignSubscribers(id: string): Promise<Subscriber[]> {
    const response = await fetch(`/api/campaigns/${id}/subscribers`, {
      method: "GET",
      headers: this.getAuthHeaders(),
    });

    if (!response.ok) {
      const errorData = await response.json();
      throw new ApiError(
        errorData.error || "キャンペーンの購読者取得に失敗しました",
        response.status,
      );
    }

    const data = await response.json();
    return data.subscribers || [];
  },

  /**
   * キャンペーンを再送
   */
  async resendCampaign(id: string): Promise<{ message: string }> {
    const response = await fetch(`/api/campaigns/${id}/resend`, {
      method: "POST",
      headers: this.getAuthHeaders(),
    });

    console.log(
      "Resend campaign response:",
      response.status,
      response.statusText,
    );

    if (!response.ok) {
      let errorMessage = "キャンペーンの再送に失敗しました";
      try {
        const errorData = await response.json();
        errorMessage = errorData.error || errorMessage;
      } catch {
        // JSONパースエラーの場合はデフォルトメッセージを使用
        console.error("Failed to parse error response");
      }
      console.error(
        "Resend campaign error:",
        errorMessage,
        "Status:",
        response.status,
      );
      throw new ApiError(errorMessage, response.status);
    }

    // レスポンスボディを安全にパース
    const text = await response.text();
    if (!text) {
      return { message: "キャンペーンの再送を開始しました" };
    }
    try {
      return JSON.parse(text);
    } catch {
      return { message: "キャンペーンの再送を開始しました" };
    }
  },
};

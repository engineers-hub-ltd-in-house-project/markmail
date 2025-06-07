import { authStore } from "../stores/authStore";
import type {
  Template,
  CreateTemplateRequest,
  UpdateTemplateRequest,
  TemplatePreviewRequest,
  TemplatePreviewResponse,
  TemplateListResponse,
} from "../types/template";

// APIレスポンスの型定義
interface ApiResponse<T> {
  data?: T;
  error?: string;
  status: number;
}

// APIベースURLの設定
const API_BASE_URL = "/api";

/**
 * 認証トークンを取得する
 */
const getAuthToken = (): string | null => {
  let token: string | null = null;
  authStore.subscribe((state) => {
    token = state.token;
  })();
  return token;
};

/**
 * ベースAPIリクエスト関数
 */
async function apiRequest<T = any>(
  path: string,
  options: RequestInit = {},
): Promise<ApiResponse<T>> {
  try {
    const url = `${API_BASE_URL}${path}`;
    const token = getAuthToken();

    // デフォルトヘッダー設定
    const headers = new Headers(options.headers || {});
    headers.set("Content-Type", "application/json");

    // 認証トークン設定
    if (token) {
      headers.set("Authorization", `Bearer ${token}`);
    }

    // リクエスト実行
    const response = await fetch(url, {
      ...options,
      headers,
    });

    // 応答処理
    if (response.ok) {
      const data = await response.json();
      return {
        data,
        status: response.status,
      };
    } else {
      let error: string;
      try {
        const errorData = await response.json();
        error =
          errorData.error ||
          errorData.message ||
          `API Error: ${response.status}`;
      } catch {
        error = `API Error: ${response.status}`;
      }

      // 認証エラーの場合はログアウト処理
      if (response.status === 401) {
        authStore.logout();
      }

      return {
        error,
        status: response.status,
      };
    }
  } catch (error: any) {
    return {
      error: `Network Error: ${error.message}`,
      status: 0,
    };
  }
}

/**
 * テンプレートAPIサービス
 */
export const templateApi = {
  /**
   * テンプレート一覧を取得
   */
  async getTemplates(
    limit: number = 50,
    offset: number = 0,
  ): Promise<ApiResponse<TemplateListResponse>> {
    return apiRequest<TemplateListResponse>(
      `/templates?limit=${limit}&offset=${offset}`,
    );
  },

  /**
   * テンプレートを取得
   */
  async getTemplate(id: string): Promise<ApiResponse<Template>> {
    return apiRequest<Template>(`/templates/${id}`);
  },

  /**
   * テンプレートを作成
   */
  async createTemplate(
    template: CreateTemplateRequest,
  ): Promise<ApiResponse<Template>> {
    return apiRequest<Template>("/templates", {
      method: "POST",
      body: JSON.stringify(template),
    });
  },

  /**
   * テンプレートを更新
   */
  async updateTemplate(
    id: string,
    template: UpdateTemplateRequest,
  ): Promise<ApiResponse<Template>> {
    return apiRequest<Template>(`/templates/${id}`, {
      method: "PUT",
      body: JSON.stringify(template),
    });
  },

  /**
   * テンプレートを削除
   */
  async deleteTemplate(id: string): Promise<ApiResponse<{ message: string }>> {
    return apiRequest(`/templates/${id}`, {
      method: "DELETE",
    });
  },

  /**
   * テンプレートプレビューを生成
   */
  async previewTemplate(
    id: string,
    request: TemplatePreviewRequest,
  ): Promise<ApiResponse<TemplatePreviewResponse>> {
    return apiRequest<TemplatePreviewResponse>(`/templates/${id}/preview`, {
      method: "POST",
      body: JSON.stringify(request),
    });
  },
};

/**
 * 認証APIサービス
 */
export const authApi = {
  /**
   * ユーザー登録
   */
  async register(
    name: string,
    email: string,
    password: string,
  ): Promise<ApiResponse<any>> {
    return apiRequest("/auth/register", {
      method: "POST",
      body: JSON.stringify({ name, email, password }),
    });
  },

  /**
   * ログイン
   */
  async login(email: string, password: string): Promise<ApiResponse<any>> {
    return apiRequest("/auth/login", {
      method: "POST",
      body: JSON.stringify({ email, password }),
    });
  },

  /**
   * トークン更新
   */
  async refreshToken(refreshToken: string): Promise<ApiResponse<any>> {
    return apiRequest("/auth/refresh", {
      method: "POST",
      body: JSON.stringify({ refresh_token: refreshToken }),
    });
  },

  /**
   * プロフィール取得
   */
  async getProfile(): Promise<ApiResponse<any>> {
    return apiRequest("/users/profile");
  },

  /**
   * プロフィール更新
   */
  async updateProfile(data: {
    name?: string;
    avatar_url?: string;
  }): Promise<ApiResponse<any>> {
    return apiRequest("/users/profile", {
      method: "PUT",
      body: JSON.stringify(data),
    });
  },
};

/**
 * マークダウン処理APIサービス
 */
export const markdownApi = {
  /**
   * マークダウンをHTMLに変換
   */
  async renderMarkdown(
    markdown: string,
    variables?: Record<string, string>,
  ): Promise<ApiResponse<any>> {
    return apiRequest("/markdown/render", {
      method: "POST",
      body: JSON.stringify({
        markdown,
        variables: variables || {},
      }),
    });
  },

  /**
   * マークダウン構文を検証
   */
  async validateMarkdown(markdown: string): Promise<ApiResponse<any>> {
    return apiRequest("/markdown/validate", {
      method: "POST",
      body: JSON.stringify({ markdown }),
    });
  },
};

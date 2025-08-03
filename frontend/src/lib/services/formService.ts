import { authStore } from "../stores/authStore";
import type {
  Form,
  CreateFormRequest,
  UpdateFormRequest,
  FormSubmissionsResponse,
} from "../types/form";

// APIベースURL
const API_BASE_URL = "/api";

// 認証トークンを取得
const getAuthToken = (): string | null => {
  let token: string | null = null;
  authStore.subscribe((state) => {
    token = state.token;
  })();
  return token;
};

// APIリクエストヘルパー
async function apiRequest<T = any>(
  path: string,
  options: RequestInit = {},
): Promise<{ data?: T; error?: string; status: number }> {
  try {
    const url = `${API_BASE_URL}${path}`;
    const token = getAuthToken();

    const headers = new Headers(options.headers || {});
    headers.set("Content-Type", "application/json");

    if (token && !path.includes("/public") && !path.includes("/submit")) {
      headers.set("Authorization", `Bearer ${token}`);
    }

    const response = await fetch(url, {
      ...options,
      headers,
    });

    if (response.ok) {
      if (response.status === 204) {
        return { status: response.status };
      }
      const data = await response.json();
      return { data, status: response.status };
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

      if (response.status === 401) {
        authStore.logout();
      }

      return { error, status: response.status };
    }
  } catch (error: any) {
    return {
      error: `Network Error: ${error.message}`,
      status: 0,
    };
  }
}

export const formService = {
  // フォーム一覧取得
  async getAll(): Promise<Form[]> {
    const response = await apiRequest<Form[]>("/forms");
    if (response.error) {
      throw new Error(response.error);
    }
    return response.data || [];
  },

  // フォーム詳細取得
  async getById(id: string): Promise<Form> {
    const response = await apiRequest<Form>(`/forms/${id}`);
    if (response.error) {
      throw new Error(response.error);
    }
    return response.data!;
  },

  // フォーム作成
  async create(data: CreateFormRequest): Promise<Form> {
    const response = await apiRequest<Form>("/forms", {
      method: "POST",
      body: JSON.stringify(data),
    });
    if (response.error) {
      throw new Error(response.error);
    }
    return response.data!;
  },

  // フォーム更新
  async update(id: string, data: UpdateFormRequest): Promise<Form> {
    const response = await apiRequest<Form>(`/forms/${id}`, {
      method: "PUT",
      body: JSON.stringify(data),
    });
    if (response.error) {
      throw new Error(response.error);
    }
    return response.data!;
  },

  // フォーム削除
  async delete(id: string): Promise<void> {
    const response = await apiRequest(`/forms/${id}`, {
      method: "DELETE",
    });
    if (response.error) {
      throw new Error(response.error);
    }
  },

  // フォーム送信データ取得
  async getSubmissions(
    id: string,
    limit: number = 20,
    offset: number = 0,
  ): Promise<FormSubmissionsResponse> {
    const response = await apiRequest<FormSubmissionsResponse>(
      `/forms/${id}/submissions?limit=${limit}&offset=${offset}`,
    );
    if (response.error) {
      throw new Error(response.error);
    }
    return response.data!;
  },

  // 公開フォーム取得（認証不要）
  async getPublicForm(id: string): Promise<Form> {
    const response = await apiRequest<Form>(`/forms/${id}/public`);
    if (response.error) {
      throw new Error(response.error);
    }
    return response.data!;
  },

  // フォーム送信（認証不要）
  async submitForm(id: string, data: Record<string, any>): Promise<any> {
    const response = await apiRequest(`/forms/${id}/submit`, {
      method: "POST",
      body: JSON.stringify({ data }),
    });
    if (response.error) {
      throw new Error(response.error);
    }
    return response.data;
  },
};

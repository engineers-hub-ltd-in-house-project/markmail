import { writable } from "svelte/store";
import type { Writable } from "svelte/store";

export interface User {
  id: string;
  name: string;
  email: string;
  avatar_url?: string;
  created_at: string;
  updated_at: string;
}

export interface AuthState {
  isAuthenticated: boolean;
  user: User | null;
  token: string | null;
  refreshToken: string | null;
}

// 初期状態
const initialState: AuthState = {
  isAuthenticated: false,
  user: null,
  token: null,
  refreshToken: null,
};

// ブラウザ環境ならlocalStorageからの復元を試みる
function createAuthStore() {
  const { subscribe, set, update }: Writable<AuthState> =
    writable(initialState);

  // ブラウザ環境でのみlocalStorageをチェック
  if (typeof window !== "undefined") {
    const token = localStorage.getItem("token");
    const refreshToken = localStorage.getItem("refresh_token");
    const userJson = localStorage.getItem("user");

    if (token && userJson) {
      try {
        const user = JSON.parse(userJson) as User;
        set({
          isAuthenticated: true,
          user,
          token,
          refreshToken,
        });
      } catch (err) {
        console.error("Failed to parse user data from localStorage", err);
      }
    }
  }

  return {
    subscribe,

    // ログイン処理
    login: (token: string, refreshToken: string, user: User) => {
      // localStorageに保存
      if (typeof window !== "undefined") {
        localStorage.setItem("token", token);
        localStorage.setItem("refresh_token", refreshToken);
        localStorage.setItem("user", JSON.stringify(user));
      }

      // ストアを更新
      set({
        isAuthenticated: true,
        user,
        token,
        refreshToken,
      });
    },

    // ログアウト処理
    logout: () => {
      // localStorageから削除
      if (typeof window !== "undefined") {
        localStorage.removeItem("token");
        localStorage.removeItem("refresh_token");
        localStorage.removeItem("user");
      }

      // ストアをリセット
      set(initialState);
    },

    // トークン更新処理
    updateTokens: (newToken: string, newRefreshToken: string) => {
      // localStorageを更新
      if (typeof window !== "undefined") {
        localStorage.setItem("token", newToken);
        localStorage.setItem("refresh_token", newRefreshToken);
      }

      // ストアを更新
      update((state) => ({
        ...state,
        token: newToken,
        refreshToken: newRefreshToken,
      }));
    },

    // ユーザー情報更新処理
    updateUser: (user: User) => {
      // localStorageを更新
      if (typeof window !== "undefined") {
        localStorage.setItem("user", JSON.stringify(user));
      }

      // ストアを更新
      update((state) => ({
        ...state,
        user,
      }));
    },
  };
}

export const authStore = createAuthStore();

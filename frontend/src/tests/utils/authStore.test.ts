import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { authStore } from "../../lib/stores/authStore";
import type { User } from "../../lib/stores/authStore";

// モックのlocalStorageを作成
const mockLocalStorage = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: vi.fn((key: string): string => {
      return store[key] || null;
    }),
    setItem: vi.fn((key: string, value: string): void => {
      store[key] = value;
    }),
    removeItem: vi.fn((key: string): void => {
      delete store[key];
    }),
    clear: vi.fn((): void => {
      store = {};
    }),
    key: vi.fn((index: number): string => {
      return Object.keys(store)[index] || null;
    }),
    length: 0,
  };
})();

// localStorageのモックを設定
beforeEach(() => {
  vi.stubGlobal("localStorage", mockLocalStorage);
  mockLocalStorage.clear();
});

// クリーンアップ
afterEach(() => {
  vi.unstubAllGlobals();
});

describe("authStore", () => {
  const testUser: User = {
    id: "123",
    name: "Test User",
    email: "test@example.com",
    created_at: "2025-01-01T00:00:00Z",
    updated_at: "2025-01-01T00:00:00Z",
  };

  describe("login", () => {
    it("should update store state and localStorage on login", () => {
      // Arrange
      const token = "test_token";
      const refreshToken = "test_refresh_token";

      // Act
      authStore.login(token, refreshToken, testUser);

      // Assert
      let state: any;
      authStore.subscribe((s) => (state = s))();

      expect(state.isAuthenticated).toBe(true);
      expect(state.token).toBe(token);
      expect(state.refreshToken).toBe(refreshToken);
      expect(state.user).toEqual(testUser);

      // localStorage checks
      expect(mockLocalStorage.setItem).toHaveBeenCalledWith("token", token);
      expect(mockLocalStorage.setItem).toHaveBeenCalledWith(
        "refresh_token",
        refreshToken,
      );
      expect(mockLocalStorage.setItem).toHaveBeenCalledWith(
        "user",
        JSON.stringify(testUser),
      );
    });
  });

  describe("logout", () => {
    it("should reset store state and clear localStorage on logout", () => {
      // Arrange - first login
      authStore.login("test_token", "test_refresh_token", testUser);

      // Act
      authStore.logout();

      // Assert
      let state: any;
      authStore.subscribe((s) => (state = s))();

      expect(state.isAuthenticated).toBe(false);
      expect(state.token).toBe(null);
      expect(state.refreshToken).toBe(null);
      expect(state.user).toBe(null);

      // localStorage checks
      expect(mockLocalStorage.removeItem).toHaveBeenCalledWith("token");
      expect(mockLocalStorage.removeItem).toHaveBeenCalledWith("refresh_token");
      expect(mockLocalStorage.removeItem).toHaveBeenCalledWith("user");
    });
  });

  describe("updateTokens", () => {
    it("should update only tokens in store state and localStorage", () => {
      // Arrange - first login
      authStore.login("old_token", "old_refresh", testUser);

      // Act
      const newToken = "new_token";
      const newRefreshToken = "new_refresh";
      authStore.updateTokens(newToken, newRefreshToken);

      // Assert
      let state: any;
      authStore.subscribe((s) => (state = s))();

      // Check that tokens were updated
      expect(state.token).toBe(newToken);
      expect(state.refreshToken).toBe(newRefreshToken);

      // Check that user data and auth state remain unchanged
      expect(state.isAuthenticated).toBe(true);
      expect(state.user).toEqual(testUser);

      // localStorage checks
      expect(mockLocalStorage.setItem).toHaveBeenCalledWith("token", newToken);
      expect(mockLocalStorage.setItem).toHaveBeenCalledWith(
        "refresh_token",
        newRefreshToken,
      );
    });
  });

  describe("updateUser", () => {
    it("should update only user info in store state and localStorage", () => {
      // Arrange - first login
      const token = "test_token";
      const refreshToken = "test_refresh_token";
      authStore.login(token, refreshToken, testUser);

      // Act
      const updatedUser = {
        ...testUser,
        name: "Updated Name",
        avatar_url: "https://example.com/avatar.jpg",
      };
      authStore.updateUser(updatedUser);

      // Assert
      let state: any;
      authStore.subscribe((s) => (state = s))();

      // Check that user was updated
      expect(state.user).toEqual(updatedUser);

      // Check that tokens and auth state remain unchanged
      expect(state.isAuthenticated).toBe(true);
      expect(state.token).toBe(token);
      expect(state.refreshToken).toBe(refreshToken);

      // localStorage check
      expect(mockLocalStorage.setItem).toHaveBeenCalledWith(
        "user",
        JSON.stringify(updatedUser),
      );
    });
  });

  describe("initialization from localStorage", () => {
    it("should initialize from localStorage if data exists", () => {
      // Arrange - set localStorage manually
      mockLocalStorage.setItem("token", "stored_token");
      mockLocalStorage.setItem("refresh_token", "stored_refresh");
      mockLocalStorage.setItem("user", JSON.stringify(testUser));

      // Act - recreate store to trigger initialization
      const createStore = vi.spyOn(authStore, "subscribe");

      // Subscribe to trigger store initialization
      let state: any;
      authStore.subscribe((s) => (state = s))();

      // Assert
      expect(state.isAuthenticated).toBe(true);
      expect(state.token).toBe("stored_token");
      expect(state.refreshToken).toBe("stored_refresh");
      expect(state.user).toEqual(testUser);
    });

    it("should not authenticate if localStorage data is incomplete", () => {
      // Arrange - set incomplete localStorage
      mockLocalStorage.setItem("token", "stored_token");
      // No user data

      // Act - recreate store to trigger initialization
      const createStore = vi.spyOn(authStore, "subscribe");

      // Subscribe to trigger store initialization
      let state: any;
      authStore.subscribe((s) => (state = s))();

      // Assert
      expect(state.isAuthenticated).toBe(false);
    });
  });
});

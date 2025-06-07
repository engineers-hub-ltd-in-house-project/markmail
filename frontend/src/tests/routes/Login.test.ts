import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";
import { goto } from "$app/navigation";
import { authStore } from "../../lib/stores/authStore";

// $app/navigationのgotoのモック作成
vi.mock("$app/navigation", () => ({
  goto: vi.fn(),
}));

// Mock Fetch API
const mockFetch = vi.fn();
global.fetch = mockFetch;

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

describe("Login Component", async () => {
  let LoginComponent: any;

  beforeEach(async () => {
    // Reset mocks
    vi.resetAllMocks();

    // Mock localStorage
    vi.stubGlobal("localStorage", mockLocalStorage);
    mockLocalStorage.clear();

    // Reset auth store
    authStore.logout();

    // Dynamically import the component
    const module = await import("../../routes/auth/login/+page.svelte");
    LoginComponent = module.default;
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("should render login form correctly", () => {
    render(LoginComponent);

    // Check form elements
    expect(screen.getByPlaceholderText("メールアドレス")).toBeInTheDocument();
    expect(screen.getByPlaceholderText("パスワード")).toBeInTheDocument();
    expect(screen.getByText("ログイン")).toBeInTheDocument();
    expect(screen.getByText("ログイン状態を保存")).toBeInTheDocument();
  });

  it("should validate required fields", async () => {
    render(LoginComponent);

    // Submit empty form
    const loginButton = screen.getByText("ログイン");
    await fireEvent.click(loginButton);

    // Check validation error
    expect(
      screen.getByText("メールアドレスとパスワードを入力してください"),
    ).toBeInTheDocument();

    // API should not be called
    expect(mockFetch).not.toHaveBeenCalled();
  });

  it("should call API and store auth data on successful login", async () => {
    // Mock API response
    const mockUser = {
      id: "user-1",
      name: "Test User",
      email: "test@example.com",
      created_at: "2025-01-01T00:00:00Z",
      updated_at: "2025-01-01T00:00:00Z",
    };

    mockFetch.mockImplementation(() => {
      return Promise.resolve({
        ok: true,
        json: () =>
          Promise.resolve({
            token: "test_token",
            refresh_token: "test_refresh_token",
            user: mockUser,
          }),
      });
    });

    render(LoginComponent);

    // Fill in form
    const emailInput = screen.getByPlaceholderText("メールアドレス");
    const passwordInput = screen.getByPlaceholderText("パスワード");
    await fireEvent.input(emailInput, {
      target: { value: "test@example.com" },
    });
    await fireEvent.input(passwordInput, { target: { value: "password123" } });

    // Submit form
    const loginButton = screen.getByText("ログイン");
    await fireEvent.click(loginButton);

    // Check API call
    expect(mockFetch).toHaveBeenCalledWith(
      "/api/auth/login",
      expect.objectContaining({
        method: "POST",
        headers: expect.objectContaining({
          "Content-Type": "application/json",
        }),
        body: JSON.stringify({
          email: "test@example.com",
          password: "password123",
        }),
      }),
    );

    // Auth store should be updated
    let state: any;
    authStore.subscribe((s) => (state = s))();

    expect(state.isAuthenticated).toBe(true);
    expect(state.token).toBe("test_token");
    expect(state.refreshToken).toBe("test_refresh_token");
    expect(state.user).toEqual(mockUser);

    // Should redirect
    expect(goto).toHaveBeenCalledWith("/templates");
  });

  it("should display error message on failed login", async () => {
    // Mock API error
    mockFetch.mockImplementation(() => {
      return Promise.resolve({
        ok: false,
        status: 401,
        json: () => Promise.resolve({ message: "Invalid credentials" }),
      });
    });

    render(LoginComponent);

    // Fill in form
    const emailInput = screen.getByPlaceholderText("メールアドレス");
    const passwordInput = screen.getByPlaceholderText("パスワード");
    await fireEvent.input(emailInput, {
      target: { value: "wrong@example.com" },
    });
    await fireEvent.input(passwordInput, {
      target: { value: "wrong_password" },
    });

    // Submit form
    const loginButton = screen.getByText("ログイン");
    await fireEvent.click(loginButton);

    // Should show error message
    await vi.waitFor(() => {
      expect(
        screen.getByText(/認証情報が正しくありません/),
      ).toBeInTheDocument();
    });

    // Auth store should not be updated
    let state: any;
    authStore.subscribe((s) => (state = s))();

    expect(state.isAuthenticated).toBe(false);

    // Should not redirect
    expect(goto).not.toHaveBeenCalled();
  });

  it("should redirect to templates page if already logged in", async () => {
    // Arrange - set logged in state
    authStore.login("existing_token", "existing_refresh", {
      id: "user-1",
      name: "Existing User",
      email: "existing@example.com",
      created_at: "2025-01-01T00:00:00Z",
      updated_at: "2025-01-01T00:00:00Z",
    });

    // Act
    render(LoginComponent);

    // Assert - should redirect immediately
    expect(goto).toHaveBeenCalledWith("/templates");
  });
});

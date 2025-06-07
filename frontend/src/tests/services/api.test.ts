import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { templateApi, authApi, markdownApi } from "../../lib/services/api";
import { authStore } from "../../lib/stores/authStore";

// APIリクエストのモック
const mockFetch = vi.fn();
global.fetch = mockFetch;

// テスト用データ
const mockTemplateResponse = {
  id: "test-template-id",
  name: "Test Template",
  subject_template: "Subject {{var}}",
  markdown_content: "# Content {{var}}",
  variables: { var: "test" },
  is_public: false,
  created_at: "2025-06-01T12:00:00Z",
  updated_at: "2025-06-01T12:00:00Z",
};

describe("API Service", () => {
  beforeEach(() => {
    vi.resetAllMocks();

    // 成功レスポンスのデフォルト設定
    mockFetch.mockImplementation(() =>
      Promise.resolve({
        ok: true,
        status: 200,
        json: () => Promise.resolve(mockTemplateResponse),
      }),
    );

    // 認証状態の設定
    authStore.login("test-token", "test-refresh", {
      id: "1",
      name: "Test User",
      email: "test@example.com",
      created_at: "2025-01-01T00:00:00Z",
      updated_at: "2025-01-01T00:00:00Z",
    });
  });

  afterEach(() => {
    authStore.logout();
  });

  describe("Template API", () => {
    it("should get templates list", async () => {
      // Arrange
      const mockListResponse = {
        templates: [mockTemplateResponse],
        total: 1,
        limit: 50,
        offset: 0,
      };

      mockFetch.mockImplementationOnce(() =>
        Promise.resolve({
          ok: true,
          status: 200,
          json: () => Promise.resolve(mockListResponse),
        }),
      );

      // Act
      const result = await templateApi.getTemplates(50, 0);

      // Assert
      expect(mockFetch).toHaveBeenCalledWith(
        "/api/templates?limit=50&offset=0",
        expect.objectContaining({
          headers: expect.any(Headers),
        }),
      );

      const headers = mockFetch.mock.calls[0][1].headers;
      expect(headers.get("Authorization")).toBe("Bearer test-token");
      expect(headers.get("Content-Type")).toBe("application/json");

      expect(result.status).toBe(200);
      expect(result.data).toEqual(mockListResponse);
      expect(result.error).toBeUndefined();
    });

    it("should get template by id", async () => {
      // Act
      const result = await templateApi.getTemplate("test-id");

      // Assert
      expect(mockFetch).toHaveBeenCalledWith(
        "/api/templates/test-id",
        expect.objectContaining({
          headers: expect.any(Headers),
        }),
      );
      expect(result.data).toEqual(mockTemplateResponse);
    });

    it("should create template", async () => {
      // Arrange
      const newTemplate = {
        name: "New Template",
        subject_template: "New Subject",
        markdown_content: "# New Content",
        is_public: false,
      };

      // Act
      const result = await templateApi.createTemplate(newTemplate);

      // Assert
      expect(mockFetch).toHaveBeenCalledWith(
        "/api/templates",
        expect.objectContaining({
          method: "POST",
          body: JSON.stringify(newTemplate),
          headers: expect.any(Headers),
        }),
      );
      expect(result.data).toEqual(mockTemplateResponse);
    });

    it("should update template", async () => {
      // Arrange
      const updateData = {
        name: "Updated Template",
        is_public: true,
      };

      // Act
      const result = await templateApi.updateTemplate("test-id", updateData);

      // Assert
      expect(mockFetch).toHaveBeenCalledWith(
        "/api/templates/test-id",
        expect.objectContaining({
          method: "PUT",
          body: JSON.stringify(updateData),
          headers: expect.any(Headers),
        }),
      );
      expect(result.data).toEqual(mockTemplateResponse);
    });

    it("should delete template", async () => {
      // Arrange
      mockFetch.mockImplementationOnce(() =>
        Promise.resolve({
          ok: true,
          status: 200,
          json: () => Promise.resolve({ message: "Template deleted" }),
        }),
      );

      // Act
      const result = await templateApi.deleteTemplate("test-id");

      // Assert
      expect(mockFetch).toHaveBeenCalledWith(
        "/api/templates/test-id",
        expect.objectContaining({
          method: "DELETE",
          headers: expect.any(Headers),
        }),
      );
      expect(result.data).toEqual({ message: "Template deleted" });
    });

    it("should preview template", async () => {
      // Arrange
      const previewRequest = {
        variables: { test: "value" },
      };

      const previewResponse = {
        html: "<h1>Content value</h1>",
        subject: "Subject value",
      };

      mockFetch.mockImplementationOnce(() =>
        Promise.resolve({
          ok: true,
          status: 200,
          json: () => Promise.resolve(previewResponse),
        }),
      );

      // Act
      const result = await templateApi.previewTemplate(
        "test-id",
        previewRequest,
      );

      // Assert
      expect(mockFetch).toHaveBeenCalledWith(
        "/api/templates/test-id/preview",
        expect.objectContaining({
          method: "POST",
          body: JSON.stringify(previewRequest),
          headers: expect.any(Headers),
        }),
      );
      expect(result.data).toEqual(previewResponse);
    });

    it("should handle API errors", async () => {
      // Arrange
      mockFetch.mockImplementationOnce(() =>
        Promise.resolve({
          ok: false,
          status: 404,
          json: () => Promise.resolve({ error: "Template not found" }),
        }),
      );

      // Act
      const result = await templateApi.getTemplate("invalid-id");

      // Assert
      expect(result.status).toBe(404);
      expect(result.error).toBe("Template not found");
      expect(result.data).toBeUndefined();
    });

    it("should handle network errors", async () => {
      // Arrange
      mockFetch.mockImplementationOnce(() =>
        Promise.reject(new Error("Network failure")),
      );

      // Act
      const result = await templateApi.getTemplate("test-id");

      // Assert
      expect(result.status).toBe(0);
      expect(result.error).toBe("Network Error: Network failure");
      expect(result.data).toBeUndefined();
    });

    it("should handle authentication errors and logout user", async () => {
      // Arrange
      mockFetch.mockImplementationOnce(() =>
        Promise.resolve({
          ok: false,
          status: 401,
          json: () => Promise.resolve({ error: "Unauthorized" }),
        }),
      );

      // Auth state before
      let beforeState: any;
      authStore.subscribe((state) => (beforeState = state))();
      expect(beforeState.isAuthenticated).toBe(true);

      // Act
      const result = await templateApi.getTemplate("test-id");

      // Assert
      expect(result.status).toBe(401);
      expect(result.error).toBe("Unauthorized");

      // Auth state after should be logged out
      let afterState: any;
      authStore.subscribe((state) => (afterState = state))();
      expect(afterState.isAuthenticated).toBe(false);
    });
  });

  describe("Auth API", () => {
    it("should register a new user", async () => {
      // Arrange
      const authResponse = {
        token: "new-token",
        refresh_token: "new-refresh",
        user: {
          id: "1",
          name: "New User",
          email: "new@example.com",
          created_at: "2025-01-01T00:00:00Z",
          updated_at: "2025-01-01T00:00:00Z",
        },
      };

      mockFetch.mockImplementationOnce(() =>
        Promise.resolve({
          ok: true,
          status: 200,
          json: () => Promise.resolve(authResponse),
        }),
      );

      // Act
      const result = await authApi.register(
        "New User",
        "new@example.com",
        "password123",
      );

      // Assert
      expect(mockFetch).toHaveBeenCalledWith(
        "/api/auth/register",
        expect.objectContaining({
          method: "POST",
          body: JSON.stringify({
            name: "New User",
            email: "new@example.com",
            password: "password123",
          }),
          headers: expect.any(Headers),
        }),
      );
      expect(result.data).toEqual(authResponse);
    });

    it("should login a user", async () => {
      // Arrange
      const authResponse = {
        token: "login-token",
        refresh_token: "login-refresh",
        user: {
          id: "1",
          name: "Login User",
          email: "login@example.com",
          created_at: "2025-01-01T00:00:00Z",
          updated_at: "2025-01-01T00:00:00Z",
        },
      };

      mockFetch.mockImplementationOnce(() =>
        Promise.resolve({
          ok: true,
          status: 200,
          json: () => Promise.resolve(authResponse),
        }),
      );

      // Act
      const result = await authApi.login("login@example.com", "password123");

      // Assert
      expect(mockFetch).toHaveBeenCalledWith(
        "/api/auth/login",
        expect.objectContaining({
          method: "POST",
          body: JSON.stringify({
            email: "login@example.com",
            password: "password123",
          }),
          headers: expect.any(Headers),
        }),
      );
      expect(result.data).toEqual(authResponse);
    });

    it("should get user profile", async () => {
      // Arrange
      const profileData = {
        id: "1",
        name: "Profile User",
        email: "profile@example.com",
        avatar_url: "https://example.com/avatar.jpg",
        created_at: "2025-01-01T00:00:00Z",
        updated_at: "2025-01-01T00:00:00Z",
      };

      mockFetch.mockImplementationOnce(() =>
        Promise.resolve({
          ok: true,
          status: 200,
          json: () => Promise.resolve(profileData),
        }),
      );

      // Act
      const result = await authApi.getProfile();

      // Assert
      expect(mockFetch).toHaveBeenCalledWith(
        "/api/users/profile",
        expect.objectContaining({
          headers: expect.any(Headers),
        }),
      );
      expect(result.data).toEqual(profileData);
    });
  });
});

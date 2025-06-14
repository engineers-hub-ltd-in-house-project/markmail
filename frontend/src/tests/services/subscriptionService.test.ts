import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { subscriptionService } from "../../lib/services/subscriptionService";
import { authStore } from "../../lib/stores/authStore";

// APIリクエストのモック
const mockFetch = vi.fn();
global.fetch = mockFetch;

// テスト用データ
const mockPlan = {
  id: "plan-1",
  name: "free",
  display_name: "Free",
  price: 0,
  contact_limit: 100,
  email_limit: 1000,
  campaign_limit: 10,
  sequence_limit: 5,
  form_limit: 5,
  ai_features: false,
  priority_support: false,
  custom_branding: false,
  api_access: false,
  webhook_support: false,
};

const mockSubscription = {
  id: "sub-1",
  user_id: "user-1",
  plan_id: "plan-1",
  status: "active",
  current_period_start: "2025-06-01T00:00:00Z",
  current_period_end: "2025-07-01T00:00:00Z",
  canceled_at: null,
  cancel_at: null,
};

const mockUsage = {
  contacts: { current: 50, limit: 100, percentage: 50.0 },
  emails_sent: { current: 200, limit: 1000, percentage: 20.0 },
  campaigns: { current: 3, limit: 10, percentage: 30.0 },
  sequences: { current: 2, limit: 5, percentage: 40.0 },
  forms: { current: 1, limit: 5, percentage: 20.0 },
};

describe("Subscription Service", () => {
  beforeEach(() => {
    vi.resetAllMocks();

    // 成功レスポンスのデフォルト設定
    mockFetch.mockImplementation(() =>
      Promise.resolve({
        ok: true,
        status: 200,
        json: () => Promise.resolve({}),
      }),
    );

    // 認証状態の設定
    authStore.login("test-token", "test-refresh", {
      id: "user-1",
      name: "Test User",
      email: "test@example.com",
      created_at: "2025-01-01T00:00:00Z",
      updated_at: "2025-01-01T00:00:00Z",
    });
  });

  afterEach(() => {
    authStore.logout();
  });

  describe("getPlans", () => {
    it("should get subscription plans", async () => {
      // Arrange
      const mockResponse = { plans: [mockPlan] };
      mockFetch.mockImplementationOnce(() =>
        Promise.resolve({
          ok: true,
          status: 200,
          json: () => Promise.resolve(mockResponse),
        }),
      );

      // Act
      const result = await subscriptionService.getPlans();

      // Assert
      expect(mockFetch).toHaveBeenCalledWith(
        "/api/subscriptions/plans",
        expect.objectContaining({
          headers: expect.any(Headers),
        }),
      );
      expect(result).toEqual(mockResponse);
    });

    it("should handle API errors", async () => {
      // Arrange
      mockFetch.mockImplementationOnce(() =>
        Promise.resolve({
          ok: false,
          status: 500,
          json: () => Promise.resolve({ error: "Server error" }),
        }),
      );

      // Act & Assert
      await expect(subscriptionService.getPlans()).rejects.toThrow(
        "Server error",
      );
    });
  });

  describe("getSubscription", () => {
    it("should get current subscription", async () => {
      // Arrange
      const mockResponse = {
        subscription: mockSubscription,
        plan: mockPlan,
        usage: mockUsage,
      };
      mockFetch.mockImplementationOnce(() =>
        Promise.resolve({
          ok: true,
          status: 200,
          json: () => Promise.resolve(mockResponse),
        }),
      );

      // Act
      const result = await subscriptionService.getCurrentSubscription();

      // Assert
      expect(mockFetch).toHaveBeenCalledWith(
        "/api/subscriptions/current",
        expect.objectContaining({
          headers: expect.any(Headers),
        }),
      );

      const headers = mockFetch.mock.calls[0][1].headers;
      expect(headers.get("Authorization")).toBe("Bearer test-token");

      expect(result).toEqual(mockResponse);
    });

    it("should handle authentication errors", async () => {
      // Arrange
      mockFetch.mockImplementationOnce(() =>
        Promise.resolve({
          ok: false,
          status: 401,
          json: () => Promise.resolve({ error: "Unauthorized" }),
        }),
      );

      // Act & Assert
      await expect(
        subscriptionService.getCurrentSubscription(),
      ).rejects.toThrow("Unauthorized");

      // Should logout user on 401
      let afterState: any;
      authStore.subscribe((state) => (afterState = state))();
      expect(afterState.isAuthenticated).toBe(false);
    });
  });

  describe("upgradePlan", () => {
    it("should upgrade subscription plan", async () => {
      // Arrange
      const upgradeRequest = { plan_id: "plan-2" };
      const mockResponse = {
        subscription: { ...mockSubscription, plan_id: "plan-2" },
      };

      mockFetch.mockImplementationOnce(() =>
        Promise.resolve({
          ok: true,
          status: 200,
          json: () => Promise.resolve(mockResponse),
        }),
      );

      // Act
      const result = await subscriptionService.upgradePlan(
        upgradeRequest.plan_id,
      );

      // Assert
      expect(mockFetch).toHaveBeenCalledWith(
        "/api/subscriptions/upgrade",
        expect.objectContaining({
          method: "POST",
          body: JSON.stringify(upgradeRequest),
          headers: expect.any(Headers),
        }),
      );

      const headers = mockFetch.mock.calls[0][1].headers;
      expect(headers.get("Authorization")).toBe("Bearer test-token");
      expect(headers.get("Content-Type")).toBe("application/json");

      expect(result).toEqual(mockResponse);
    });

    it("should handle upgrade validation errors", async () => {
      // Arrange
      const upgradeRequest = { plan_id: "invalid-plan" };
      mockFetch.mockImplementationOnce(() =>
        Promise.resolve({
          ok: false,
          status: 400,
          json: () => Promise.resolve({ error: "Invalid plan ID" }),
        }),
      );

      // Act & Assert
      await expect(
        subscriptionService.upgradePlan(upgradeRequest.plan_id),
      ).rejects.toThrow("Invalid plan ID");
    });

    it("should handle usage limit errors", async () => {
      // Arrange
      const upgradeRequest = { plan_id: "plan-free" };
      mockFetch.mockImplementationOnce(() =>
        Promise.resolve({
          ok: false,
          status: 400,
          json: () =>
            Promise.resolve({
              error:
                "現在のコンタクト数(150)が新しいプランの制限(100)を超えています",
            }),
        }),
      );

      // Act & Assert
      await expect(
        subscriptionService.upgradePlan(upgradeRequest.plan_id),
      ).rejects.toThrow(/コンタクト数.*制限/);
    });
  });

  describe("cancelSubscription", () => {
    it("should cancel subscription immediately", async () => {
      // Arrange
      const cancelRequest = { cancel_at_period_end: false };
      const mockResponse = {
        subscription: {
          ...mockSubscription,
          status: "canceled",
          canceled_at: "2025-06-14T12:00:00Z",
          cancel_at: "2025-06-14T12:00:00Z",
        },
      };

      mockFetch.mockImplementationOnce(() =>
        Promise.resolve({
          ok: true,
          status: 200,
          json: () => Promise.resolve(mockResponse),
        }),
      );

      // Act
      const result = await subscriptionService.cancelSubscription(
        cancelRequest.cancel_at_period_end,
      );

      // Assert
      expect(mockFetch).toHaveBeenCalledWith(
        "/api/subscriptions/cancel",
        expect.objectContaining({
          method: "POST",
          body: JSON.stringify(cancelRequest),
          headers: expect.any(Headers),
        }),
      );

      expect(result).toEqual(mockResponse);
    });

    it("should cancel subscription at period end", async () => {
      // Arrange
      const cancelRequest = { cancel_at_period_end: true };
      const mockResponse = {
        subscription: {
          ...mockSubscription,
          canceled_at: "2025-06-14T12:00:00Z",
          cancel_at: "2025-07-01T00:00:00Z",
        },
      };

      mockFetch.mockImplementationOnce(() =>
        Promise.resolve({
          ok: true,
          status: 200,
          json: () => Promise.resolve(mockResponse),
        }),
      );

      // Act
      const result = await subscriptionService.cancelSubscription(
        cancelRequest.cancel_at_period_end,
      );

      // Assert
      expect(result).toEqual(mockResponse);
    });
  });

  describe("getPaymentHistory", () => {
    it("should get payment history with pagination", async () => {
      // Arrange
      const mockPayments = [
        {
          id: "payment-1",
          amount: 4980,
          currency: "JPY",
          status: "succeeded",
          description: "Pro plan subscription",
          created_at: "2025-06-01T00:00:00Z",
        },
      ];
      const mockResponse = {
        payments: mockPayments,
        total: 1,
        limit: 50,
        offset: 0,
      };

      mockFetch.mockImplementationOnce(() =>
        Promise.resolve({
          ok: true,
          status: 200,
          json: () => Promise.resolve(mockResponse),
        }),
      );

      // Act
      const result = await subscriptionService.getPaymentHistory(50, 0);

      // Assert
      expect(mockFetch).toHaveBeenCalledWith(
        "/api/subscriptions/payment-history?limit=50&offset=0",
        expect.objectContaining({
          headers: expect.any(Headers),
        }),
      );

      expect(result).toEqual(mockResponse);
    });

    it("should handle default pagination", async () => {
      // Arrange
      mockFetch.mockImplementationOnce(() =>
        Promise.resolve({
          ok: true,
          status: 200,
          json: () =>
            Promise.resolve({ payments: [], total: 0, limit: 50, offset: 0 }),
        }),
      );

      // Act
      const result = await subscriptionService.getPaymentHistory();

      // Assert
      expect(mockFetch).toHaveBeenCalledWith(
        "/api/subscriptions/payment-history?limit=50&offset=0",
        expect.any(Object),
      );
    });
  });

  describe("getUsage", () => {
    it("should get current usage statistics", async () => {
      // Arrange
      mockFetch.mockImplementationOnce(() =>
        Promise.resolve({
          ok: true,
          status: 200,
          json: () => Promise.resolve(mockUsage),
        }),
      );

      // Act
      const result = await subscriptionService.getUsage();

      // Assert
      expect(mockFetch).toHaveBeenCalledWith(
        "/api/subscriptions/usage",
        expect.objectContaining({
          headers: expect.any(Headers),
        }),
      );

      expect(result).toEqual(mockUsage);
    });

    it("should handle usage calculation errors", async () => {
      // Arrange
      mockFetch.mockImplementationOnce(() =>
        Promise.resolve({
          ok: false,
          status: 500,
          json: () => Promise.resolve({ error: "Usage calculation failed" }),
        }),
      );

      // Act & Assert
      await expect(subscriptionService.getUsage()).rejects.toThrow(
        "Usage calculation failed",
      );
    });
  });

  describe("Network errors", () => {
    it("should handle network failures", async () => {
      // Arrange
      mockFetch.mockImplementationOnce(() =>
        Promise.reject(new Error("Network failure")),
      );

      // Act & Assert
      await expect(subscriptionService.getPlans()).rejects.toThrow(
        "Network failure",
      );
    });
  });
});

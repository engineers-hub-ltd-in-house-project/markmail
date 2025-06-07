import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";
import { goto } from "$app/navigation";
import { authStore } from "../../lib/stores/authStore";

// $app/storesのpageのモック作成
vi.mock("$app/stores", () => ({
  page: {
    subscribe: (cb: (value: any) => void) => {
      cb({
        params: {
          id: "test-template-id",
        },
        url: {
          pathname: "/templates/test-template-id",
        },
      });
      return () => {};
    },
  },
}));

// $app/navigationのgotoのモック作成
vi.mock("$app/navigation", () => ({
  goto: vi.fn(),
}));

// Mock Fetch API
const mockFetch = vi.fn();
global.fetch = mockFetch;

// テストデータ
const mockTemplate = {
  id: "test-template-id",
  name: "Test Template",
  subject_template: "Test Subject with {{variable}}",
  markdown_content: "# Test Content\n\nWith {{variable}}",
  variables: { variable: "test value" },
  is_public: true,
  created_at: "2025-06-01T12:00:00Z",
  updated_at: "2025-06-02T12:00:00Z",
};

const mockPreviewResponse = {
  html: "<h1>Test Content</h1><p>With test value</p>",
  subject: "Test Subject with test value",
};

describe("Template Detail View", async () => {
  let TemplateDetailView: any;

  beforeEach(async () => {
    // Reset mocks
    vi.resetAllMocks();

    // Set up authStore
    authStore.login("test-token", "test-refresh-token", {
      id: "user-1",
      name: "Test User",
      email: "test@example.com",
      created_at: "2025-01-01T00:00:00Z",
      updated_at: "2025-01-01T00:00:00Z",
    });

    // Mock API calls
    mockFetch.mockImplementation((url, options) => {
      if (url.includes(`/api/templates/test-template-id`)) {
        if (options && options.method === "POST" && url.includes("/preview")) {
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve(mockPreviewResponse),
          });
        }
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve(mockTemplate),
        });
      }
      return Promise.resolve({ ok: false });
    });

    // Dynamically import the component to avoid SSR/browser API issues during tests
    const module = await import("../../routes/templates/[id]/+page.svelte");
    TemplateDetailView = module.default;
  });

  it("should render template details and preview", async () => {
    const { container } = render(TemplateDetailView);

    // Wait for data loading
    await vi.waitFor(() => {
      expect(screen.queryByText("読み込み中...")).toBeNull();
    });

    // Check that template details are rendered
    expect(screen.getByText("Test Template")).toBeInTheDocument();
    expect(
      screen.getByText("Test Subject with {{variable}}"),
    ).toBeInTheDocument();

    // Check that preview is shown
    const previewButton = screen.getByText("プレビュー");
    expect(previewButton.classList.contains("bg-blue-600")).toBe(true);

    // HTML content is rendered
    expect(container.querySelector(".prose")).not.toBeNull();
  });

  it("should show markdown content when switching to markdown view", async () => {
    render(TemplateDetailView);

    // Wait for data loading
    await vi.waitFor(() => {
      expect(screen.queryByText("読み込み中...")).toBeNull();
    });

    // Click on Markdown button
    const markdownButton = screen.getByText("マークダウン");
    await fireEvent.click(markdownButton);

    // Check that markdown content is shown
    expect(screen.getByText("# Test Content")).toBeInTheDocument();
    expect(screen.getByText("With {{variable}}")).toBeInTheDocument();

    // Preview content should be hidden
    expect(container.querySelector(".prose")).toBeNull();
  });

  it("should navigate to edit page when edit button is clicked", async () => {
    render(TemplateDetailView);

    // Wait for data loading
    await vi.waitFor(() => {
      expect(screen.queryByText("読み込み中...")).toBeNull();
    });

    // Find and click edit button
    const editButton = screen.getByText("編集");
    await fireEvent.click(editButton);

    // Check that navigation was triggered
    expect(goto).toHaveBeenCalledWith("/templates/test-template-id/edit");
  });

  it("should handle API error gracefully", async () => {
    // Mock API error
    mockFetch.mockImplementation(() => {
      return Promise.resolve({
        ok: false,
        status: 404,
        json: () => Promise.resolve({ message: "Not found" }),
      });
    });

    render(TemplateDetailView);

    // Wait for error state
    await vi.waitFor(() => {
      expect(
        screen.getByText(/テンプレートの取得に失敗しました/),
      ).toBeInTheDocument();
    });

    // Check that retry button is shown
    const retryButton = screen.getByText("再試行");
    expect(retryButton).toBeInTheDocument();
  });
});

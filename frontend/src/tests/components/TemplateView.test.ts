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

// DOMPurifyのモック作成
vi.mock("dompurify", () => ({
  default: {
    sanitize: vi.fn((html) => html),
  },
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
      if (url === "/api/templates/test-template-id") {
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve(mockTemplate),
        });
      }
      if (url === "/api/templates/test-template-id/preview") {
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve(mockPreviewResponse),
        });
      }

      return Promise.resolve({
        ok: false,
        status: 404,
        json: () => Promise.resolve({ error: "Not found" }),
      });
    });

    // Dynamically import the component to avoid SSR/browser API issues during tests
    const module = await import("../../routes/templates/[id]/+page.svelte");
    TemplateDetailView = module.default;
  });

  it.skip("should render template details and preview", async () => {
    render(TemplateDetailView);

    // Wait for template name to appear
    await vi.waitFor(
      () => {
        expect(screen.getByText("Test Template")).toBeInTheDocument();
      },
      { timeout: 5000 },
    );

    // Check that template details are rendered
    expect(
      screen.getByText("Test Subject with {{variable}}"),
    ).toBeInTheDocument();

    // Check that preview is shown
    const previewButton = screen.getByText("プレビュー");
    expect(previewButton.classList.contains("bg-blue-600")).toBe(true);

    // Check that preview button is active
    expect(previewButton).toBeInTheDocument();
  });

  it.skip("should show markdown content when switching to markdown view", async () => {
    render(TemplateDetailView);

    // Wait for template to load
    await vi.waitFor(() => {
      expect(screen.getByText("Test Template")).toBeInTheDocument();
    });

    // Click on Markdown button
    const markdownButton = screen.getByText("マークダウン");
    await fireEvent.click(markdownButton);

    // Check that markdown content is shown
    expect(screen.getByText("# Test Content")).toBeInTheDocument();
    expect(screen.getByText("With {{variable}}")).toBeInTheDocument();

    // Markdown button should be active now
    expect(markdownButton.classList.contains("bg-blue-600")).toBe(true);
  });

  it.skip("should navigate to edit page when edit button is clicked", async () => {
    render(TemplateDetailView);

    // Wait for template to load
    await vi.waitFor(() => {
      expect(screen.getByText("Test Template")).toBeInTheDocument();
    });

    // Find and click edit button
    const editButton = screen.getByText("編集");
    await fireEvent.click(editButton);

    // Check that navigation was triggered
    expect(goto).toHaveBeenCalledWith("/templates/test-template-id/edit");
  });

  it.skip("should handle API error gracefully", async () => {
    // Mock API error
    mockFetch.mockImplementation(() => {
      return Promise.resolve({
        ok: false,
        status: 404,
        json: () => Promise.resolve({ error: "Template not found" }),
      });
    });

    render(TemplateDetailView);

    // Wait for error state
    await vi.waitFor(() => {
      expect(screen.getByText("Template not found")).toBeInTheDocument();
    });

    // Check that retry button is shown
    const retryButton = screen.getByText("再試行");
    expect(retryButton).toBeInTheDocument();
  });
});

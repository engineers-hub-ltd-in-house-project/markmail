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
          pathname: "/templates/test-template-id/edit",
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

// markedのモック作成
vi.mock("marked", () => ({
  marked: vi.fn(async (markdown) => `<p>${markdown}</p>`),
}));

// Mock Fetch API
const mockFetch = vi.fn();
global.fetch = mockFetch;

// mockPreviewResponseの定義
const mockPreviewResponse = {
  html: "<h1>Test Content</h1><p>With test value</p>",
  subject: "Test Subject with test value",
};

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

describe("Template Edit", async () => {
  let TemplateEditComponent: any;

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
        if (options && options.method === "PUT") {
          return Promise.resolve({
            ok: true,
            json: () =>
              Promise.resolve({ ...mockTemplate, ...JSON.parse(options.body) }),
          });
        }
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve(mockTemplate),
        });
      }
      if (url === "/api/markdown/render") {
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
    const module = await import(
      "../../routes/templates/[id]/edit/+page.svelte"
    );
    TemplateEditComponent = module.default;
  });

  it.skip("should load template data correctly", async () => {
    render(TemplateEditComponent);

    // Wait for template to load
    await vi.waitFor(() => {
      expect(screen.getByLabelText("テンプレート名 *")).toBeInTheDocument();
    });

    // Check that form fields are populated with template data
    const nameInput = screen.getByLabelText(
      "テンプレート名 *",
    ) as HTMLInputElement;
    expect(nameInput.value).toBe(mockTemplate.name);

    const subjectInput = screen.getByLabelText(
      "件名テンプレート *",
    ) as HTMLInputElement;
    expect(subjectInput.value).toBe(mockTemplate.subject_template);

    const contentTextarea = screen.getByLabelText(
      "マークダウン内容 *",
    ) as HTMLTextAreaElement;
    expect(contentTextarea.value).toBe(mockTemplate.markdown_content);

    const publicCheckbox = screen.getByLabelText(
      "公開テンプレートにする",
    ) as HTMLInputElement;
    expect(publicCheckbox.checked).toBe(mockTemplate.is_public);

    const variablesTextarea = screen.getByPlaceholderText(
      /user_name=田中太郎/,
    ) as HTMLTextAreaElement;
    expect(variablesTextarea.value).toBe("variable=test value");
  });

  it.skip("should update template when form is submitted", async () => {
    render(TemplateEditComponent);

    // Wait for template to load
    await vi.waitFor(() => {
      expect(screen.getByLabelText("テンプレート名 *")).toBeInTheDocument();
    });

    // Update form fields
    const nameInput = screen.getByLabelText(
      "テンプレート名 *",
    ) as HTMLInputElement;
    await fireEvent.input(nameInput, { target: { value: "Updated Template" } });

    const subjectInput = screen.getByLabelText(
      "件名テンプレート *",
    ) as HTMLInputElement;
    await fireEvent.input(subjectInput, {
      target: { value: "Updated Subject" },
    });

    const contentTextarea = screen.getByLabelText(
      "マークダウン内容 *",
    ) as HTMLTextAreaElement;
    await fireEvent.input(contentTextarea, {
      target: { value: "# Updated Content" },
    });

    // Submit form
    const updateButton = screen.getByText("更新");
    await fireEvent.click(updateButton);

    // Check that API was called with correct data
    expect(mockFetch).toHaveBeenCalledWith(
      `/api/templates/test-template-id`,
      expect.objectContaining({
        method: "PUT",
        headers: expect.objectContaining({
          Authorization: "Bearer test-token",
          "Content-Type": "application/json",
        }),
        body: expect.stringMatching(/"name":"Updated Template"/),
      }),
    );

    // Check navigation
    expect(goto).toHaveBeenCalledWith("/templates/test-template-id");
  });

  it.skip("should toggle between edit and preview modes", async () => {
    render(TemplateEditComponent);

    // Wait for template to load
    await vi.waitFor(() => {
      expect(screen.getByLabelText("テンプレート名 *")).toBeInTheDocument();
    });

    // Initially in edit mode
    expect(screen.getByLabelText("マークダウン内容 *")).toBeInTheDocument();

    // Click on preview button
    const previewButton = screen.getByText("プレビュー");
    await fireEvent.click(previewButton);

    // Should be in preview mode
    expect(screen.getByText("プレビュー")).toBeInTheDocument();
    expect(screen.queryByLabelText("マークダウン内容 *")).toBeNull();

    // Click on editor button
    const editorButton = screen.getByText("エディター");
    await fireEvent.click(editorButton);

    // Should be back in edit mode
    expect(screen.getByLabelText("マークダウン内容 *")).toBeInTheDocument();
  });

  it.skip("should handle API error gracefully", async () => {
    // Mock API error
    mockFetch.mockImplementationOnce(() => {
      return Promise.resolve({
        ok: false,
        status: 404,
        json: () => Promise.resolve({ error: "Template not found" }),
      });
    });

    render(TemplateEditComponent);

    // Wait for error state
    await vi.waitFor(() => {
      expect(screen.getByText("Template not found")).toBeInTheDocument();
    });
  });

  it.skip("should validate required fields", async () => {
    render(TemplateEditComponent);

    // Wait for template to load
    await vi.waitFor(() => {
      expect(screen.getByLabelText("テンプレート名 *")).toBeInTheDocument();
    });

    // Clear required fields
    const nameInput = screen.getByLabelText(
      "テンプレート名 *",
    ) as HTMLInputElement;
    await fireEvent.input(nameInput, { target: { value: "" } });

    const subjectInput = screen.getByLabelText(
      "件名テンプレート *",
    ) as HTMLInputElement;
    await fireEvent.input(subjectInput, { target: { value: "" } });

    // Submit form
    const updateButton = screen.getByText("更新");
    await fireEvent.click(updateButton);

    // Check validation error
    expect(
      screen.getByText("テンプレート名、件名、内容は必須です"),
    ).toBeInTheDocument();

    // API should not be called
    expect(mockFetch).not.toHaveBeenCalledWith(
      `/api/templates/test-template-id`,
      expect.objectContaining({
        method: "PUT",
      }),
    );
  });
});

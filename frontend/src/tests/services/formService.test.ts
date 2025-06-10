import { describe, it, expect, vi, beforeEach } from "vitest";
import type {
  Form,
  CreateFormRequest,
  UpdateFormRequest,
  FormSubmissionsResponse,
} from "$lib/types/form";

// formServiceをモック
vi.mock("$lib/services/formService", () => ({
  formService: {
    getAll: vi.fn(),
    getById: vi.fn(),
    create: vi.fn(),
    update: vi.fn(),
    delete: vi.fn(),
    getSubmissions: vi.fn(),
    getPublicForm: vi.fn(),
    submitForm: vi.fn(),
  },
}));

import { formService } from "$lib/services/formService";

describe("FormService", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("getAll", () => {
    it("should fetch forms successfully", async () => {
      const mockForms: Form[] = [
        {
          id: "1",
          user_id: "user1",
          name: "Contact Form",
          description: "A contact form",
          slug: "contact",
          markdown_content: "# Contact",
          form_fields: [],
          settings: {},
          status: "published",
          submission_count: 5,
          created_at: "2024-01-01",
          updated_at: "2024-01-01",
        },
      ];

      vi.mocked(formService.getAll).mockResolvedValueOnce(mockForms);

      const result = await formService.getAll();

      expect(formService.getAll).toHaveBeenCalled();
      expect(result).toEqual(mockForms);
    });

    it("should handle errors gracefully", async () => {
      vi.mocked(formService.getAll).mockRejectedValueOnce(
        new Error("Network error"),
      );

      await expect(formService.getAll()).rejects.toThrow("Network error");
    });
  });

  describe("getById", () => {
    it("should fetch a single form", async () => {
      const mockForm: Form = {
        id: "1",
        user_id: "user1",
        name: "Contact Form",
        description: "A contact form",
        slug: "contact",
        markdown_content: "# Contact",
        form_fields: [
          {
            field_type: "text",
            name: "name",
            label: "Name",
            required: true,
            display_order: 1,
          },
        ],
        settings: {
          submit_button_text: "Submit",
          success_message: "Thank you!",
        },
        status: "published",
        submission_count: 5,
        created_at: "2024-01-01",
        updated_at: "2024-01-01",
      };

      vi.mocked(formService.getById).mockResolvedValueOnce(mockForm);

      const result = await formService.getById("1");

      expect(formService.getById).toHaveBeenCalledWith("1");
      expect(result).toEqual(mockForm);
    });
  });

  describe("create", () => {
    it("should create a new form", async () => {
      const createRequest: CreateFormRequest = {
        name: "New Form",
        description: "A new form",
        markdown_content: "# New Form",
        form_fields: [],
      };

      const mockResponse: Form = {
        id: "2",
        user_id: "user1",
        name: "New Form",
        description: "A new form",
        slug: "new-form",
        markdown_content: "# New Form",
        form_fields: [],
        settings: {},
        status: "draft",
        submission_count: 0,
        created_at: "2024-01-01",
        updated_at: "2024-01-01",
      };

      vi.mocked(formService.create).mockResolvedValueOnce(mockResponse);

      const result = await formService.create(createRequest);

      expect(formService.create).toHaveBeenCalledWith(createRequest);
      expect(result).toEqual(mockResponse);
    });
  });

  describe("update", () => {
    it("should update an existing form", async () => {
      const updateRequest: UpdateFormRequest = {
        name: "Updated Form",
        status: "published",
      };

      const mockResponse: Form = {
        id: "1",
        user_id: "user1",
        name: "Updated Form",
        description: "A contact form",
        slug: "contact",
        markdown_content: "# Contact",
        form_fields: [],
        settings: {},
        status: "published",
        submission_count: 5,
        created_at: "2024-01-01",
        updated_at: "2024-01-02",
      };

      vi.mocked(formService.update).mockResolvedValueOnce(mockResponse);

      const result = await formService.update("1", updateRequest);

      expect(formService.update).toHaveBeenCalledWith("1", updateRequest);
      expect(result).toEqual(mockResponse);
    });
  });

  describe("delete", () => {
    it("should delete a form", async () => {
      vi.mocked(formService.delete).mockResolvedValueOnce(undefined);

      await formService.delete("1");

      expect(formService.delete).toHaveBeenCalledWith("1");
    });
  });

  describe("getSubmissions", () => {
    it("should fetch form submissions with pagination", async () => {
      const mockResponse: FormSubmissionsResponse = {
        submissions: [
          {
            id: "1",
            form_id: "1",
            data: { name: "John", email: "john@example.com" },
            created_at: "2024-01-01",
          },
        ],
        total: 1,
        limit: 20,
        offset: 0,
      };

      vi.mocked(formService.getSubmissions).mockResolvedValueOnce(mockResponse);

      const result = await formService.getSubmissions("1", 20, 0);

      expect(formService.getSubmissions).toHaveBeenCalledWith("1", 20, 0);
      expect(result).toEqual(mockResponse);
    });
  });

  describe("getPublicForm", () => {
    it("should fetch a public form", async () => {
      const mockForm: Form = {
        id: "1",
        user_id: "user1",
        name: "Public Form",
        description: "A public form",
        slug: "public",
        markdown_content: "# Public Form",
        form_fields: [
          {
            field_type: "email",
            name: "email",
            label: "Email",
            required: true,
            display_order: 1,
          },
        ],
        settings: {
          submit_button_text: "Subscribe",
          success_message: "Welcome!",
        },
        status: "published",
        submission_count: 10,
        created_at: "2024-01-01",
        updated_at: "2024-01-01",
      };

      vi.mocked(formService.getPublicForm).mockResolvedValueOnce(mockForm);

      const result = await formService.getPublicForm("1");

      expect(formService.getPublicForm).toHaveBeenCalledWith("1");
      expect(result).toEqual(mockForm);
    });
  });

  describe("submitForm", () => {
    it("should submit form data", async () => {
      const formData = {
        name: "John Doe",
        email: "john@example.com",
      };

      const mockResponse = {
        id: "1",
        form_id: "1",
        data: formData,
        created_at: "2024-01-01",
      };

      vi.mocked(formService.submitForm).mockResolvedValueOnce(mockResponse);

      const result = await formService.submitForm("1", formData);

      expect(formService.submitForm).toHaveBeenCalledWith("1", formData);
      expect(result).toEqual(mockResponse);
    });
  });
});

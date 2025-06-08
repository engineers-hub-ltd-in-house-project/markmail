export interface FormField {
  field_type: "text" | "email" | "textarea" | "select" | "radio" | "checkbox";
  name: string;
  label: string;
  placeholder?: string;
  required: boolean;
  validation_rules?: Record<string, any>;
  options?: any[];
  display_order: number;
}

export interface Form {
  id: string;
  user_id: string;
  name: string;
  description?: string;
  slug: string;
  markdown_content: string;
  form_fields: FormField[];
  settings: Record<string, any>;
  status: "draft" | "published" | "archived";
  submission_count: number;
  created_at: string;
  updated_at: string;
}

export interface CreateFormRequest {
  name: string;
  description?: string;
  slug?: string;
  markdown_content: string;
  form_fields?: FormField[];
  settings?: Record<string, any>;
}

export interface UpdateFormRequest {
  name?: string;
  description?: string;
  markdown_content?: string;
  form_fields?: FormField[];
  settings?: Record<string, any>;
  status?: "draft" | "published" | "archived";
}

export interface FormSubmission {
  id: string;
  form_id: string;
  subscriber_id?: string;
  data: Record<string, any>;
  ip_address?: string;
  user_agent?: string;
  referrer?: string;
  confirmation_token?: string;
  confirmed_at?: string;
  created_at: string;
}

export interface FormSubmissionsResponse {
  submissions: FormSubmission[];
  total: number;
  limit: number;
  offset: number;
}

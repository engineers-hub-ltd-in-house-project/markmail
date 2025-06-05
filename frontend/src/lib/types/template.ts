export interface Template {
  id: string;
  name: string;
  subject_template: string;
  markdown_content: string;
  variables: Record<string, string>;
  is_public: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateTemplateRequest {
  name: string;
  subject_template: string;
  markdown_content: string;
  variables?: Record<string, string>;
  is_public?: boolean;
}

export interface UpdateTemplateRequest {
  name?: string;
  subject_template?: string;
  markdown_content?: string;
  variables?: Record<string, string>;
  is_public?: boolean;
}

export interface TemplatePreviewRequest {
  variables: Record<string, string>;
}

export interface TemplatePreviewResponse {
  html: string;
  subject: string;
}

export interface TemplateListResponse {
  templates: Template[];
  total: number;
  limit: number;
  offset: number;
}

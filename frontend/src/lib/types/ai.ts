// AI機能の型定義

// サポートされる言語
export type Language = "ja" | "en";

// コンテンツタイプ
export type ContentType =
  | "email_template"
  | "subject"
  | "sequence"
  | "form"
  | "scenario";

// コンテンツトーン
export type ContentTone =
  | "formal"
  | "casual"
  | "professional"
  | "friendly"
  | "urgent";

// シナリオ生成リクエスト
export interface GenerateScenarioRequest {
  industry: string;
  target_audience: string;
  goal: string;
  additional_context?: string;
  language?: Language;
}

// シナリオ生成レスポンス
export interface GenerateScenarioResponse {
  scenario_name: string;
  description: string;
  sequence: GeneratedSequence;
  forms: GeneratedForm[];
  templates: GeneratedTemplate[];
}

// 生成されたシーケンス
export interface GeneratedSequence {
  name: string;
  description: string;
  trigger_type: string;
  steps: GeneratedSequenceStep[];
}

// 生成されたシーケンスステップ
export interface GeneratedSequenceStep {
  name: string;
  step_type: string;
  delay_value: number;
  delay_unit: string;
  template_index?: number;
  conditions?: Record<string, any>;
}

// 生成されたフォーム
export interface GeneratedForm {
  name: string;
  description: string;
  fields: GeneratedFormField[];
}

// 生成されたフォームフィールド
export interface GeneratedFormField {
  field_type: string;
  name: string;
  label: string;
  required: boolean;
  options?: string[];
}

// 生成されたテンプレート
export interface GeneratedTemplate {
  name: string;
  subject: string;
  content: string;
  variables: string[];
}

// コンテンツ生成リクエスト
export interface GenerateContentRequest {
  content_type: string;
  context: ContentContext;
  options?: GenerationOptions;
}

// コンテンツコンテキスト
export interface ContentContext {
  industry?: string;
  target_audience?: string;
  tone?: ContentTone;
  language?: Language;
  existing_content?: string;
}

// 生成オプション
export interface GenerationOptions {
  variations?: number;
  max_length?: number;
  include_personalization?: boolean;
}

// コンテンツ生成レスポンス
export interface GenerateContentResponse {
  content: string;
  variations?: string[];
  suggested_variables: string[];
  metadata: ContentMetadata;
}

// コンテンツメタデータ
export interface ContentMetadata {
  estimated_reading_time: number;
  word_count: number;
  personalization_score: number;
  clarity_score: number;
}

// 件名最適化リクエスト
export interface OptimizeSubjectRequest {
  original_subject: string;
  target_audience: string;
  campaign_goal?: string;
  variations_count?: number;
  language?: Language;
}

// 件名最適化レスポンス
export interface OptimizeSubjectResponse {
  optimized_subjects: SubjectVariation[];
  best_pick: number;
}

// 件名バリエーション
export interface SubjectVariation {
  subject: string;
  predicted_open_rate: number;
  reasoning: string;
}

// AI生成ステータス
export type AIGenerationStatus = "idle" | "generating" | "success" | "error";

// AI生成ステート
export interface AIGenerationState {
  status: AIGenerationStatus;
  error?: string;
  progress?: number;
}

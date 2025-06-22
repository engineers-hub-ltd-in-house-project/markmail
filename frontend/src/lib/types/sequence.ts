// シーケンス関連の型定義

export interface Sequence {
  id: string;
  user_id: string;
  name: string;
  description?: string;
  trigger_type: TriggerType;
  trigger_config?: Record<string, any>;
  status: SequenceStatus;
  active_subscribers: number;
  completed_subscribers: number;
  created_at: string;
  updated_at: string;
}

export type TriggerType =
  | "manual"
  | "subscriber_created"
  | "form_submission"
  | "tag_added";
export type SequenceStatus = "draft" | "active" | "paused" | "archived";

export interface SequenceStep {
  id: string;
  sequence_id: string;
  name: string;
  step_order: number;
  step_type: StepType;
  delay_value: number;
  delay_unit: string;
  template_id?: string;
  subject?: string;
  conditions?: any;
  action_config?: any;
  created_at: string;
  updated_at: string;
}

export type StepType = "email" | "wait" | "condition" | "tag";

export interface StepCondition {
  field: string;
  operator:
    | "equals"
    | "not_equals"
    | "contains"
    | "not_contains"
    | "greater_than"
    | "less_than";
  value: any;
}

export interface SequenceEnrollment {
  id: string;
  sequence_id: string;
  subscriber_id: string;
  current_step_id?: string;
  status: EnrollmentStatus;
  enrolled_at: string;
  completed_at?: string;
  cancelled_at?: string;
  next_step_at?: string;
  metadata?: any;
  created_at: string;
  updated_at: string;
}

export type EnrollmentStatus = "active" | "paused" | "completed" | "cancelled";

// API リクエスト/レスポンス用の型

export interface CreateSequenceRequest {
  name: string;
  description?: string;
  trigger_type: TriggerType;
  trigger_config?: Record<string, any>;
}

export interface UpdateSequenceRequest {
  name?: string;
  description?: string;
  trigger_type?: TriggerType;
  trigger_config?: Record<string, any>;
  status?: SequenceStatus;
}

export interface CreateSequenceStepRequest {
  name: string;
  step_order: number;
  step_type: StepType;
  delay_value?: number;
  delay_unit?: string;
  template_id?: string;
  subject?: string;
  conditions?: any;
  action_config?: any;
}

export interface UpdateSequenceStepRequest {
  step_order?: number;
  step_type?: StepType;
  delay_minutes?: number;
  template_id?: number;
  conditions?: StepCondition[];
}

export interface SequenceWithSteps extends Sequence {
  steps: SequenceStep[];
}

// バックエンドの#[serde(flatten)]により、Sequenceのフィールドがトップレベルに展開される
export type SequenceWithStepsAndTemplates = Sequence & {
  steps: SequenceStepWithTemplate[];
};

// バックエンドの#[serde(flatten)]により、SequenceStepのフィールドがトップレベルに展開される
export type SequenceStepWithTemplate = SequenceStep & {
  template?: {
    id: string;
    user_id: string;
    name: string;
    subject_template: string;
    markdown_content: string;
    html_content?: string;
    variables: any;
    is_public: boolean;
    created_at: string;
    updated_at: string;
  };
};

// ビジュアルエディタ用の型

export interface SequenceNode {
  id: string;
  type: "trigger" | "email" | "delay" | "condition" | "end";
  data: {
    label: string;
    step?: SequenceStep;
    trigger?: TriggerType;
  };
  position: { x: number; y: number };
}

export interface SequenceEdge {
  id: string;
  source: string;
  target: string;
  label?: string;
}

// エラー型

export interface SequenceError {
  message: string;
  field?: string;
  code?: string;
}

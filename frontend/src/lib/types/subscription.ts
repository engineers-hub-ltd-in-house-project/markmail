export interface SubscriptionPlan {
  id: string;
  name: string;
  display_name: string;
  description?: string;
  price: number;
  billing_interval: string;
  // 制限項目
  contact_limit: number;
  monthly_email_limit: number;
  campaign_limit: number;
  template_limit: number;
  sequence_limit: number;
  sequence_step_limit: number;
  form_limit: number;
  form_submission_limit: number;
  user_limit: number;
  webhook_limit: number;
  // 機能フラグ
  custom_markdown_components: boolean;
  ai_features: boolean;
  advanced_analytics: boolean;
  ab_testing: boolean;
  api_access: boolean;
  priority_support: boolean;
  custom_domain: boolean;
  white_label: boolean;
  // メタデータ
  sort_order: number;
  is_active: boolean;
  features?: Record<string, any>;
  created_at: string;
  updated_at: string;
}

export interface UserSubscription {
  id: string;
  user_id: string;
  plan_id: string;
  status: "active" | "canceled" | "past_due" | "trialing";
  current_period_start: string;
  current_period_end: string;
  cancel_at?: string;
  canceled_at?: string;
  trial_end?: string;
  metadata?: Record<string, any>;
  stripe_subscription_id?: string;
  stripe_customer_id?: string;
  created_at: string;
  updated_at: string;
}

export interface UsageMetric {
  current: number;
  limit: number;
  percentage: number;
}

export interface UsageSummary {
  contacts: UsageMetric;
  emails_sent: UsageMetric;
  campaigns: UsageMetric;
  templates: UsageMetric;
  sequences: UsageMetric;
  forms: UsageMetric;
  form_submissions: UsageMetric;
}

export interface SubscriptionDetailsResponse {
  subscription: UserSubscription;
  plan: SubscriptionPlan;
  usage: UsageSummary;
}

export interface PaymentHistory {
  id: string;
  user_id: string;
  subscription_id?: string;
  amount: number;
  currency: string;
  status: "succeeded" | "failed" | "pending" | "refunded";
  description?: string;
  stripe_payment_intent_id?: string;
  stripe_invoice_id?: string;
  metadata?: Record<string, any>;
  paid_at?: string;
  created_at: string;
}

export interface PlansResponse {
  plans: SubscriptionPlan[];
}

export interface UpgradeRequest {
  plan_id: string;
}

export interface CancelRequest {
  cancel_at_period_end: boolean;
}

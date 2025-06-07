// import type { User } from "../stores/authStore";

/**
 * 購読者のステータス
 */
export enum SubscriberStatus {
  ACTIVE = "active",
  UNSUBSCRIBED = "unsubscribed",
  BOUNCED = "bounced",
  COMPLAINED = "complained",
}

/**
 * 購読者モデル
 */
export interface Subscriber {
  id: string;
  user_id: string;
  email: string;
  name?: string;
  status: SubscriberStatus;
  tags: string[];
  custom_fields: Record<string, unknown>;
  subscribed_at: string;
  unsubscribed_at?: string;
  created_at: string;
  updated_at: string;
}

/**
 * 購読者作成リクエスト
 */
export interface CreateSubscriberRequest {
  email: string;
  name?: string;
  tags?: string[];
  custom_fields?: Record<string, unknown>;
}

/**
 * 購読者更新リクエスト
 */
export interface UpdateSubscriberRequest {
  name?: string;
  status?: SubscriberStatus;
  tags?: string[];
  custom_fields?: Record<string, unknown>;
}

/**
 * 購読者CSV一括インポートリクエスト
 */
export interface ImportSubscribersRequest {
  file: unknown; // File type from DOM API
  tag?: string;
}

/**
 * 購読者一覧レスポンス
 */
export interface SubscriberListResponse {
  subscribers: Subscriber[];
  total: number;
  limit: number;
  offset: number;
  available_tags?: string[];
}

/**
 * 購読者インポートレスポンス
 */
export interface ImportSubscribersResponse {
  message: string;
  imported: number;
  failed: number;
  errors?: string[];
}

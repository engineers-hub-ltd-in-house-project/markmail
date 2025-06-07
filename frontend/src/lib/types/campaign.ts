import type { User } from "../stores/authStore";

/**
 * キャンペーンのステータス
 */
export enum CampaignStatus {
  DRAFT = "draft",
  SCHEDULED = "scheduled",
  SENDING = "sending",
  SENT = "sent",
  CANCELED = "canceled",
}

/**
 * キャンペーンモデル
 */
export interface Campaign {
  id: string;
  user_id: string;
  name: string;
  description?: string;
  status: CampaignStatus;
  template_id: string;
  subject: string;
  recipient_list?: string[];
  scheduled_at?: string;
  sent_at?: string;
  total_recipients: number;
  opened_count: number;
  clicked_count: number;
  created_at: string;
  updated_at: string;
}

/**
 * キャンペーン作成リクエスト
 */
export interface CreateCampaignRequest {
  name: string;
  description?: string;
  template_id: string;
  subject: string;
  recipient_list?: string[];
}

/**
 * キャンペーン更新リクエスト
 */
export interface UpdateCampaignRequest {
  name?: string;
  description?: string;
  template_id?: string;
  subject?: string;
  recipient_list?: string[];
}

/**
 * キャンペーンスケジュールリクエスト
 */
export interface ScheduleCampaignRequest {
  scheduled_at: string; // ISO 8601フォーマット: '2025-06-10T15:00:00Z'
}

/**
 * キャンペーンプレビューレスポンス
 */
export interface PreviewCampaignResponse {
  html: string;
}

/**
 * キャンペーン一覧レスポンス
 */
export interface CampaignListResponse {
  campaigns: Campaign[];
  total: number;
  limit: number;
  offset: number;
}

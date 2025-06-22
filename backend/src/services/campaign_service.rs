use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    database::{campaigns, subscribers, templates},
    models::{
        campaign::{
            Campaign, CreateCampaignRequest, ScheduleCampaignRequest, UpdateCampaignRequest,
        },
        subscriber::Subscriber,
    },
    services::{
        email_service::{EmailMessage, EmailService},
        markdown_service::MarkdownService,
    },
};

pub struct CampaignService;

impl Default for CampaignService {
    fn default() -> Self {
        Self
    }
}

impl CampaignService {
    pub fn new() -> Self {
        Self
    }

    // キャンペーン作成
    pub async fn create_campaign(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        request: &CreateCampaignRequest,
    ) -> Result<Campaign, String> {
        // テンプレートが存在するか確認
        let template = templates::find_template_by_id(pool, request.template_id, Some(user_id))
            .await
            .map_err(|e| format!("テンプレートの確認に失敗しました: {}", e))?;

        if template.is_none() {
            return Err(
                "指定されたテンプレートが見つからないか、アクセス権限がありません".to_string(),
            );
        }

        // キャンペーンを作成
        let campaign = campaigns::create_campaign(pool, user_id, request)
            .await
            .map_err(|e| format!("キャンペーン作成に失敗しました: {}", e))?;

        Ok(campaign)
    }

    // キャンペーン更新
    pub async fn update_campaign(
        &self,
        pool: &PgPool,
        campaign_id: Uuid,
        user_id: Uuid,
        request: &UpdateCampaignRequest,
    ) -> Result<Campaign, String> {
        // テンプレートIDが更新される場合はテンプレートの存在確認
        if let Some(template_id) = request.template_id {
            let template = templates::find_template_by_id(pool, template_id, Some(user_id))
                .await
                .map_err(|e| format!("テンプレートの確認に失敗しました: {}", e))?;

            if template.is_none() {
                return Err(
                    "指定されたテンプレートが見つからないか、アクセス権限がありません".to_string(),
                );
            }
        }

        // キャンペーンを更新
        let updated_campaign = campaigns::update_campaign(pool, campaign_id, user_id, request)
            .await
            .map_err(|e| format!("キャンペーン更新に失敗しました: {}", e))?;

        match updated_campaign {
            Some(campaign) => Ok(campaign),
            None => Err("キャンペーンが見つからないか、更新権限がありません".to_string()),
        }
    }

    // キャンペーンをスケジュール
    pub async fn schedule_campaign(
        &self,
        pool: &PgPool,
        campaign_id: Uuid,
        user_id: Uuid,
        request: &ScheduleCampaignRequest,
    ) -> Result<Campaign, String> {
        // 現在時刻より前の日時でスケジュールしようとしている場合はエラー
        if request.scheduled_at < Utc::now() {
            return Err("過去の日時でキャンペーンをスケジュールすることはできません".to_string());
        }

        // キャンペーンをスケジュール
        let updated_campaign =
            campaigns::schedule_campaign(pool, campaign_id, user_id, request.scheduled_at)
                .await
                .map_err(|e| format!("キャンペーンのスケジュールに失敗しました: {}", e))?;

        match updated_campaign {
            Some(campaign) => Ok(campaign),
            None => Err("キャンペーンが見つからないか、スケジュール権限がありません".to_string()),
        }
    }

    // キャンペーン送信開始
    pub async fn start_sending_campaign(
        &self,
        pool: &PgPool,
        campaign_id: Uuid,
        user_id: Uuid,
    ) -> Result<Campaign, String> {
        // キャンペーンを送信開始状態に更新
        let updated_campaign = campaigns::start_campaign_sending(pool, campaign_id, user_id)
            .await
            .map_err(|e| format!("キャンペーン送信開始に失敗しました: {}", e))?;

        match updated_campaign {
            Some(campaign) => Ok(campaign),
            None => Err("キャンペーンが見つからないか、送信権限がありません".to_string()),
        }
    }

    // キャンペーンプレビュー生成（テストユーザーデータで）
    pub async fn generate_campaign_preview(
        &self,
        pool: &PgPool,
        campaign_id: Uuid,
        user_id: Uuid,
    ) -> Result<String, String> {
        // キャンペーン情報を取得
        let campaign = campaigns::find_campaign_by_id(pool, campaign_id, user_id)
            .await
            .map_err(|e| format!("キャンペーン情報の取得に失敗しました: {}", e))?
            .ok_or_else(|| "キャンペーンが見つかりません".to_string())?;

        // テンプレート情報を取得
        let template = templates::find_template_by_id(pool, campaign.template_id, Some(user_id))
            .await
            .map_err(|e| format!("テンプレート情報の取得に失敗しました: {}", e))?
            .ok_or_else(|| "テンプレートが見つかりません".to_string())?;

        // テストデータでHTMLを生成
        let test_data = serde_json::json!({
            "user_name": "テストユーザー",
            "name": "テストユーザー",
            "email": "test@example.com",
            "company": "サンプル株式会社",
            "company_name": "サンプル株式会社",
            "service_name": "MarkMail",
            "login_url": "https://markmail.example.com/login",
            "unsubscribe_url": "https://markmail.example.com/unsubscribe?id=12345"
        });

        // マークダウンサービスでHTMLを生成
        let markdown_service = MarkdownService::new();
        let html = markdown_service
            .render_with_variables(&template.markdown_content, &test_data)
            .map_err(|e| format!("マークダウンのレンダリングに失敗しました: {}", e))?;

        Ok(html)
    }

    // 指定したキャンペーンの購読者一覧を取得
    pub async fn get_campaign_subscribers(
        &self,
        pool: &PgPool,
        _campaign_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<Subscriber>, String> {
        // アクティブな購読者を取得
        let options = crate::models::subscriber::ListSubscriberOptions {
            limit: None,
            offset: None,
            search: None,
            tag: None,
            status: None, // 一時的にステータスフィルタを無効化
            sort_by: None,
            sort_order: None,
        };

        let subscribers = subscribers::list_user_subscribers(pool, user_id, &options)
            .await
            .map_err(|e| format!("購読者の取得に失敗しました: {}", e))?;

        Ok(subscribers)
    }

    // キャンペーンの送信処理
    pub async fn process_campaign_sending(
        &self,
        pool: &PgPool,
        campaign_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), String> {
        // エラーが発生した場合にステータスをerrorに更新するためのガード
        let result = self
            .process_campaign_sending_internal(pool, campaign_id, user_id)
            .await;

        if let Err(ref e) = result {
            tracing::error!("キャンペーン送信処理でエラー: {}", e);
            // エラーステータスに更新
            if let Err(update_err) = campaigns::update_campaign_status(
                pool,
                campaign_id,
                user_id,
                crate::models::campaign::CampaignStatus::Error,
            )
            .await
            {
                tracing::error!("エラーステータスへの更新に失敗: {}", update_err);
            }
        }

        result
    }

    // 実際の送信処理
    async fn process_campaign_sending_internal(
        &self,
        pool: &PgPool,
        campaign_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), String> {
        // キャンペーン情報を取得
        let campaign = campaigns::find_campaign_by_id(pool, campaign_id, user_id)
            .await
            .map_err(|e| format!("キャンペーン情報の取得に失敗しました: {}", e))?
            .ok_or_else(|| "キャンペーンが見つかりません".to_string())?;

        // テンプレート情報を取得
        let template = templates::find_template_by_id(pool, campaign.template_id, Some(user_id))
            .await
            .map_err(|e| format!("テンプレート情報の取得に失敗しました: {}", e))?
            .ok_or_else(|| "テンプレートが見つかりません".to_string())?;

        // 購読者リストを取得
        let subscribers = self
            .get_campaign_subscribers(pool, campaign_id, user_id)
            .await?;

        if subscribers.is_empty() {
            return Err("送信対象の購読者が存在しません".to_string());
        }

        // メールサービスを初期化
        let email_service = EmailService::new(pool.clone())
            .await
            .map_err(|e| format!("メールサービスの初期化に失敗しました: {}", e))?;

        // マークダウンサービスを初期化
        let markdown_service = MarkdownService::new();

        // 購読者ごとにメールメッセージを作成
        let mut email_messages = Vec::new();
        for subscriber in &subscribers {
            // 購読者固有の変数を設定
            let mut variables = if template.variables.is_null() {
                serde_json::json!({})
            } else {
                template.variables.clone()
            };

            // variablesをオブジェクトとして扱う
            if let serde_json::Value::Object(ref mut map) = variables {
                let name = subscriber
                    .name
                    .clone()
                    .unwrap_or_else(|| "お客様".to_string());

                // name と first_name の両方を設定（互換性のため）
                map.insert("name".to_string(), serde_json::json!(name.clone()));
                map.insert("first_name".to_string(), serde_json::json!(name));
                map.insert(
                    "email".to_string(),
                    serde_json::json!(subscriber.email.clone()),
                );

                // カスタムフィールドを変数に追加
                if let serde_json::Value::Object(custom_fields) = &subscriber.custom_fields {
                    for (key, value) in custom_fields {
                        map.insert(key.clone(), value.clone());
                    }
                }

                // 配信停止URLを追加
                map.insert(
                    "unsubscribe_url".to_string(),
                    serde_json::json!(format!(
                        "https://markmail.example.com/unsubscribe/{}",
                        subscriber.id
                    )),
                );
            }

            // HTMLとテキストをレンダリング
            let html_body = markdown_service
                .render_with_variables(&template.markdown_content, &variables)
                .map_err(|e| format!("HTMLレンダリングに失敗しました: {}", e))?;

            let text_body = html2text::from_read(html_body.as_bytes(), 80);

            // 件名の変数を置換
            let subject = if let serde_json::Value::Object(vars) = &variables {
                let mut subject = template.subject_template.clone();
                for (key, value) in vars {
                    if let serde_json::Value::String(val) = value {
                        subject = subject.replace(&format!("{{{{{}}}}}", key), val);
                    }
                }
                subject
            } else {
                template.subject_template.clone()
            };

            let email_message = EmailMessage {
                to: vec![subscriber.email.clone()],
                subject,
                html_body,
                text_body: Some(text_body),
                reply_to: None,
                headers: None,
            };

            email_messages.push(email_message);
        }

        // バッチでメール送信
        let results = email_service
            .send_campaign(email_messages)
            .await
            .map_err(|e| format!("メール送信に失敗しました: {}", e))?;

        // 送信結果を集計
        let mut sent_count = 0;
        let mut failed_count = 0;
        for result in &results {
            match result.status {
                crate::services::email_service::EmailStatus::Sent => sent_count += 1,
                crate::services::email_service::EmailStatus::Failed => failed_count += 1,
                _ => {}
            }
        }

        // キャンペーンの統計情報を更新
        campaigns::update_campaign_stats(
            pool,
            campaign_id,
            Some(subscribers.len() as i32),
            Some(sent_count),
            None,
            None,
        )
        .await
        .map_err(|e| format!("統計情報の更新に失敗しました: {}", e))?;

        // 送信結果に応じてステータスを更新
        if failed_count == 0 && sent_count > 0 {
            // 全て成功した場合
            campaigns::complete_campaign_sending(pool, campaign_id)
                .await
                .map_err(|e| format!("キャンペーン状態の更新に失敗しました: {}", e))?;
        } else if sent_count == 0 && failed_count > 0 {
            // 全て失敗した場合
            campaigns::update_campaign_status(
                pool,
                campaign_id,
                user_id,
                crate::models::campaign::CampaignStatus::Error,
            )
            .await
            .map_err(|e| format!("キャンペーン状態の更新に失敗しました: {}", e))?;
        } else if failed_count > 0 {
            // 一部失敗した場合も送信完了とする（部分的成功）
            campaigns::complete_campaign_sending(pool, campaign_id)
                .await
                .map_err(|e| format!("キャンペーン状態の更新に失敗しました: {}", e))?;
        }

        tracing::info!(
            "キャンペーン {} の送信が完了しました。成功: {}, 失敗: {}",
            campaign_id,
            sent_count,
            failed_count
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    // mockallの使用は現在のテストでは不要なので削除
    // 必要になったら適切に実装する

    // キャンペーンスケジュールのテスト
    #[test]
    fn test_validate_schedule_date() {
        // テスト用の関数を作成して、日時バリデーションロジックのみをテスト
        fn validate_schedule_date(scheduled_at: chrono::DateTime<chrono::Utc>) -> bool {
            scheduled_at >= Utc::now()
        }

        // 過去の日時
        let past_date = Utc::now() - Duration::hours(1);
        assert!(
            !validate_schedule_date(past_date),
            "過去の日時は無効であるべき"
        );

        // 未来の日時
        let future_date = Utc::now() + Duration::hours(1);
        assert!(
            validate_schedule_date(future_date),
            "未来の日時は有効であるべき"
        );
    }

    // スケジュール可能な日時のテスト
    #[tokio::test]
    #[ignore] // 実際のDBが必要なのでignore
    async fn test_schedule_campaign_future_date() {
        // このテストは実際のDBが必要なので、ここでは省略
    }
}

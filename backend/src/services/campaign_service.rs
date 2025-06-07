use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    database::{campaigns, templates},
    models::{
        campaign::{
            Campaign, CreateCampaignRequest, ScheduleCampaignRequest, UpdateCampaignRequest,
        },
        subscriber::Subscriber,
    },
    services::markdown_service::MarkdownService,
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
            "name": "テストユーザー",
            "email": "test@example.com",
            "company": "サンプル株式会社",
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

    // 指定したキャンペーンの購読者一覧を取得（未実装）
    #[allow(dead_code)]
    pub async fn get_campaign_subscribers(
        &self,
        _pool: &PgPool,
        _campaign_id: Uuid,
        _user_id: Uuid,
    ) -> Result<Vec<Subscriber>, String> {
        // ここは今後実装予定
        // 現時点ではテスト用のデータを返す
        Ok(Vec::new())
    }

    // キャンペーンの送信処理（非同期処理で実行予定）
    #[allow(dead_code)]
    pub async fn process_campaign_sending(
        &self,
        _pool: &PgPool,
        _campaign_id: Uuid,
    ) -> Result<(), String> {
        // ここは今後実装予定
        // メールキュー、バッチ処理などを導入する
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use mockall::predicate::*;
    use mockall::*;

    // テスト用のモックを作成
    mock! {
        pub TemplateRepo {}
        impl TemplateRepo {
            pub async fn find_template_by_id(&self, id: Uuid, user_id: Option<Uuid>) -> Result<Option<crate::models::template::Template>, sqlx::Error>;
        }
    }

    mock! {
        pub CampaignRepo {}
        impl CampaignRepo {
            pub async fn create_campaign(&self, user_id: Uuid, request: &CreateCampaignRequest) -> Result<Campaign, sqlx::Error>;
            pub async fn update_campaign(&self, campaign_id: Uuid, user_id: Uuid, request: &UpdateCampaignRequest) -> Result<Option<Campaign>, sqlx::Error>;
            pub async fn schedule_campaign(&self, campaign_id: Uuid, user_id: Uuid, scheduled_at: chrono::DateTime<chrono::Utc>) -> Result<Option<Campaign>, sqlx::Error>;
            pub async fn start_campaign_sending(&self, campaign_id: Uuid, user_id: Uuid) -> Result<Option<Campaign>, sqlx::Error>;
        }
    }

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

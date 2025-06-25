use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::{PgPool, Result};
use uuid::Uuid;

use crate::models::crm::{CrmIntegrationSettings, CrmProviderType};

/// CRM統合作成パラメータ
pub struct CreateCrmIntegrationParams<'a> {
    pub user_id: Uuid,
    pub provider: CrmProviderType,
    pub org_id: &'a str,
    pub instance_url: &'a str,
    pub access_token: &'a str,
    pub refresh_token: Option<&'a str>,
    pub settings: &'a CrmIntegrationSettings,
}

/// CRM統合設定の作成
pub async fn create_crm_integration(
    pool: &PgPool,
    params: CreateCrmIntegrationParams<'_>,
) -> Result<Uuid> {
    // 認証情報をJSON形式で保存
    let credentials = serde_json::json!({
        "access_token": params.access_token,
        "refresh_token": params.refresh_token,
    });

    // 設定をJSON形式で保存
    let settings_json = serde_json::json!({
        "sync_enabled": params.settings.sync_enabled,
        "sync_interval_minutes": params.settings.sync_interval_minutes,
        "batch_size": params.settings.batch_size,
    });

    // Salesforce固有の設定
    let salesforce_settings = serde_json::json!({
        "api_version": "v60.0",
        "instance_url": params.instance_url,
        "org_id": params.org_id,
    });

    let integration_id = sqlx::query_scalar!(
        r#"
        INSERT INTO crm_integrations (
            user_id, provider, org_id, instance_url,
            credentials, settings, salesforce_settings,
            field_mappings
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (user_id, provider) 
        DO UPDATE SET
            org_id = EXCLUDED.org_id,
            instance_url = EXCLUDED.instance_url,
            credentials = EXCLUDED.credentials,
            settings = EXCLUDED.settings,
            salesforce_settings = EXCLUDED.salesforce_settings,
            field_mappings = EXCLUDED.field_mappings,
            sync_enabled = true,
            updated_at = NOW()
        RETURNING id
        "#,
        params.user_id,
        params.provider.as_str(),
        params.org_id,
        params.instance_url,
        credentials,
        settings_json,
        salesforce_settings,
        serde_json::to_value(&params.settings.field_mappings).unwrap_or(JsonValue::Array(vec![]))
    )
    .fetch_one(pool)
    .await?;

    Ok(integration_id)
}

/// ユーザーのCRM統合設定を取得
pub async fn get_user_crm_integration(
    pool: &PgPool,
    user_id: Uuid,
    provider: CrmProviderType,
) -> Result<Option<CrmIntegration>> {
    let integration = sqlx::query_as!(
        CrmIntegration,
        r#"
        SELECT 
            id,
            user_id,
            provider,
            org_id,
            instance_url,
            credentials,
            settings,
            salesforce_settings,
            field_mappings,
            sync_enabled,
            last_sync_at,
            created_at,
            updated_at
        FROM crm_integrations
        WHERE user_id = $1 AND provider = $2
        "#,
        user_id,
        provider.as_str()
    )
    .fetch_optional(pool)
    .await?;

    Ok(integration)
}

/// すべてのアクティブなCRM統合を取得
pub async fn get_active_crm_integrations(
    pool: &PgPool,
    provider: Option<CrmProviderType>,
) -> Result<Vec<CrmIntegration>> {
    let integrations = if let Some(provider) = provider {
        sqlx::query_as!(
            CrmIntegration,
            r#"
            SELECT 
                id,
                user_id,
                provider,
                org_id,
                instance_url,
                credentials,
                settings,
                salesforce_settings,
                field_mappings,
                sync_enabled,
                last_sync_at,
                created_at,
                updated_at
            FROM crm_integrations
            WHERE sync_enabled = true AND provider = $1
            "#,
            provider.as_str()
        )
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as!(
            CrmIntegration,
            r#"
            SELECT 
                id,
                user_id,
                provider,
                org_id,
                instance_url,
                credentials,
                settings,
                salesforce_settings,
                field_mappings,
                sync_enabled,
                last_sync_at,
                created_at,
                updated_at
            FROM crm_integrations
            WHERE sync_enabled = true
            "#
        )
        .fetch_all(pool)
        .await?
    };

    Ok(integrations)
}

/// CRM統合を無効化
pub async fn deactivate_crm_integration(
    pool: &PgPool,
    integration_id: Uuid,
    user_id: Uuid,
) -> Result<()> {
    sqlx::query!(
        r#"
        UPDATE crm_integrations
        SET sync_enabled = false, updated_at = NOW()
        WHERE id = $1 AND user_id = $2
        "#,
        integration_id,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// 同期ステータスを更新
pub async fn update_sync_status(
    pool: &PgPool,
    integration_id: Uuid,
    sync_at: DateTime<Utc>,
) -> Result<()> {
    sqlx::query!(
        r#"
        UPDATE crm_integrations
        SET 
            last_sync_at = $2,
            updated_at = NOW()
        WHERE id = $1
        "#,
        integration_id,
        sync_at
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// 同期ログ作成パラメータ
pub struct CreateSyncLogParams<'a> {
    pub integration_id: Uuid,
    pub sync_type: &'a str,
    pub entity_type: &'a str,
    pub entity_count: i32,
    pub success_count: i32,
    pub error_count: i32,
    pub error_details: Option<JsonValue>,
}

/// 同期ログを作成
pub async fn create_sync_log(pool: &PgPool, params: CreateSyncLogParams<'_>) -> Result<Uuid> {
    let log_id = sqlx::query_scalar!(
        r#"
        INSERT INTO crm_sync_logs (
            integration_id, sync_type, entity_type,
            entity_count, success_count, error_count,
            started_at, completed_at, error_details
        )
        VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW(), $7)
        RETURNING id
        "#,
        params.integration_id,
        params.sync_type,
        params.entity_type,
        params.entity_count,
        params.success_count,
        params.error_count,
        params.error_details
    )
    .fetch_one(pool)
    .await?;

    Ok(log_id)
}

/// 同期履歴を取得
pub async fn get_sync_logs(
    pool: &PgPool,
    integration_id: Uuid,
    entity_type: Option<&str>,
    limit: i64,
) -> Result<Vec<CrmSyncLog>> {
    let logs = if let Some(entity_type) = entity_type {
        sqlx::query_as!(
            CrmSyncLog,
            r#"
            SELECT 
                id,
                integration_id,
                sync_type,
                entity_type,
                entity_count,
                success_count,
                error_count,
                started_at,
                completed_at,
                error_details,
                created_at
            FROM crm_sync_logs
            WHERE integration_id = $1 AND entity_type = $2
            ORDER BY started_at DESC
            LIMIT $3
            "#,
            integration_id,
            entity_type,
            limit
        )
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as!(
            CrmSyncLog,
            r#"
            SELECT 
                id,
                integration_id,
                sync_type,
                entity_type,
                entity_count,
                success_count,
                error_count,
                started_at,
                completed_at,
                error_details,
                created_at
            FROM crm_sync_logs
            WHERE integration_id = $1
            ORDER BY started_at DESC
            LIMIT $2
            "#,
            integration_id,
            limit
        )
        .fetch_all(pool)
        .await?
    };

    Ok(logs)
}

/// CRM同期ステータスを作成または更新
pub async fn upsert_sync_status(
    pool: &PgPool,
    integration_id: Uuid,
    entity_type: &str,
    markmail_id: Uuid,
    crm_id: &str,
    sync_status: &str,
) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO crm_sync_status (
            integration_id, entity_type, markmail_id,
            crm_id, sync_status, sync_direction
        )
        VALUES ($1, $2, $3, $4, $5, 'to_crm')
        ON CONFLICT (integration_id, entity_type, markmail_id)
        DO UPDATE SET
            crm_id = EXCLUDED.crm_id,
            sync_status = EXCLUDED.sync_status,
            updated_at = NOW()
        "#,
        integration_id,
        entity_type,
        markmail_id,
        crm_id,
        sync_status
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// データベースモデル定義
#[derive(Debug)]
pub struct CrmIntegration {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub org_id: Option<String>,
    pub instance_url: Option<String>,
    pub credentials: JsonValue,
    pub settings: JsonValue,
    pub salesforce_settings: Option<JsonValue>,
    pub field_mappings: JsonValue,
    pub sync_enabled: bool,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CrmIntegration {
    /// アクセストークンを取得
    pub fn get_access_token(&self) -> Option<String> {
        self.credentials
            .get("access_token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// リフレッシュトークンを取得
    pub fn get_refresh_token(&self) -> Option<String> {
        self.credentials
            .get("refresh_token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// 同期が有効かどうか
    pub fn is_active(&self) -> bool {
        self.sync_enabled
    }

    /// 同期設定を取得
    pub fn get_sync_settings(&self) -> CrmIntegrationSettings {
        let sync_enabled = self
            .settings
            .get("sync_enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let sync_interval_minutes = self
            .settings
            .get("sync_interval_minutes")
            .and_then(|v| v.as_i64())
            .unwrap_or(60) as i32;

        let batch_size = self
            .settings
            .get("batch_size")
            .and_then(|v| v.as_u64())
            .unwrap_or(100) as usize;

        let field_mappings =
            serde_json::from_value(self.field_mappings.clone()).unwrap_or_default();

        CrmIntegrationSettings {
            sync_enabled,
            sync_interval_minutes,
            batch_size,
            field_mappings,
        }
    }
}

#[derive(Debug)]
pub struct CrmSyncLog {
    pub id: Uuid,
    pub integration_id: Uuid,
    pub sync_type: String,
    pub entity_type: String,
    pub entity_count: i32,
    pub success_count: i32,
    pub error_count: i32,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_details: Option<JsonValue>,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn test_create_and_get_crm_integration(pool: PgPool) -> Result<()> {
        // テストユーザーを作成
        let user_id = sqlx::query_scalar!(
            "INSERT INTO users (email, password_hash, name) VALUES ($1, $2, $3) RETURNING id",
            "test@example.com",
            "hash",
            "Test User"
        )
        .fetch_one(&pool)
        .await?;

        // CRM統合を作成
        let settings = CrmIntegrationSettings {
            sync_enabled: true,
            sync_interval_minutes: 60,
            batch_size: 100,
            field_mappings: Default::default(),
        };

        let params = CreateCrmIntegrationParams {
            user_id,
            provider: CrmProviderType::Salesforce,
            org_id: "test-org-id",
            instance_url: "https://test.salesforce.com",
            access_token: "test-token",
            refresh_token: Some("refresh-token"),
            settings: &settings,
        };
        let integration_id = create_crm_integration(&pool, params).await?;

        // 作成した統合を取得
        let integration = get_user_crm_integration(&pool, user_id, CrmProviderType::Salesforce)
            .await?
            .expect("Integration should exist");

        assert_eq!(integration.id, integration_id);
        assert_eq!(integration.user_id, user_id);
        assert_eq!(integration.provider, "salesforce");
        assert!(integration.sync_enabled);
        assert_eq!(integration.org_id, Some("test-org-id".to_string()));

        Ok(())
    }

    #[sqlx::test]
    async fn test_sync_log_creation(pool: PgPool) -> Result<()> {
        // テストデータをセットアップ
        let user_id = sqlx::query_scalar!(
            "INSERT INTO users (email, password_hash, name) VALUES ($1, $2, $3) RETURNING id",
            "test@example.com",
            "hash",
            "Test User"
        )
        .fetch_one(&pool)
        .await?;

        let settings = CrmIntegrationSettings::default();
        let params = CreateCrmIntegrationParams {
            user_id,
            provider: CrmProviderType::Salesforce,
            org_id: "test-org",
            instance_url: "https://test.salesforce.com",
            access_token: "test-token",
            refresh_token: None,
            settings: &settings,
        };
        let integration_id = create_crm_integration(&pool, params).await?;

        // 同期ログを作成
        let log_params = CreateSyncLogParams {
            integration_id,
            sync_type: "manual",
            entity_type: "contact",
            entity_count: 10,
            success_count: 8,
            error_count: 2,
            error_details: Some(serde_json::json!({"errors": ["Error 1", "Error 2"]})),
        };
        let log_id = create_sync_log(&pool, log_params).await?;

        // ログを取得
        let logs = get_sync_logs(&pool, integration_id, Some("contact"), 10).await?;
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].id, log_id);
        assert_eq!(logs[0].entity_count, 10);
        assert_eq!(logs[0].success_count, 8);
        assert_eq!(logs[0].error_count, 2);

        Ok(())
    }
}

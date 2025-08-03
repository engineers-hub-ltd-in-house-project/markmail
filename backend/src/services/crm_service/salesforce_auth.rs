use serde::{Deserialize, Serialize};
use std::process::Command;
use thiserror::Error;

/// Salesforce認証エラー
#[derive(Error, Debug)]
pub enum SalesforceAuthError {
    #[error("Salesforce CLIが見つかりません。'sf'コマンドがインストールされていることを確認してください")]
    CliNotFound,

    #[error("Salesforce CLIコマンドの実行に失敗しました: {0}")]
    CliExecutionError(String),

    #[error("Salesforceへのログインに失敗しました: {0}")]
    LoginFailed(String),

    #[error("組織情報の取得に失敗しました: {0}")]
    OrgInfoError(String),

    #[error("JSONパース エラー: {0}")]
    ParseError(#[from] serde_json::Error),
}

/// Salesforce組織情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesforceOrgInfo {
    pub org_id: String,
    pub username: String,
    pub instance_url: String,
    pub access_token: String,
    pub api_version: String,
    pub connected_status: String,
}

/// Salesforce認証結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesforceAuthResult {
    pub org_info: SalesforceOrgInfo,
    pub refresh_token: Option<String>,
}

/// Salesforce CLI認証ラッパー
pub struct SalesforceAuth;

impl SalesforceAuth {
    /// Salesforce CLIがインストールされているか確認
    pub fn check_cli_installed() -> Result<bool, SalesforceAuthError> {
        let output = Command::new("sf")
            .arg("--version")
            .output()
            .map_err(|_| SalesforceAuthError::CliNotFound)?;

        Ok(output.status.success())
    }

    /// Webブラウザを使用してSalesforceにログイン
    pub async fn login_web(alias: &str) -> Result<SalesforceAuthResult, SalesforceAuthError> {
        // sf org login web --alias myorg --set-default
        let output = Command::new("sf")
            .args([
                "org",
                "login",
                "web",
                "--alias",
                alias,
                "--set-default",
                "--json",
            ])
            .output()
            .map_err(|e| SalesforceAuthError::CliExecutionError(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SalesforceAuthError::LoginFailed(stderr.to_string()));
        }

        // ログイン成功後、組織情報を取得
        Self::get_org_info(alias).await
    }

    /// デバイスフローを使用してSalesforceにログイン（ヘッドレス環境用）
    pub async fn login_device(alias: &str) -> Result<SalesforceAuthResult, SalesforceAuthError> {
        // sf org login device --alias myorg --set-default
        let output = Command::new("sf")
            .args([
                "org",
                "login",
                "device",
                "--alias",
                alias,
                "--set-default",
                "--json",
            ])
            .output()
            .map_err(|e| SalesforceAuthError::CliExecutionError(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SalesforceAuthError::LoginFailed(stderr.to_string()));
        }

        // ログイン成功後、組織情報を取得
        Self::get_org_info(alias).await
    }

    /// 既存の認証情報から組織情報を取得
    pub async fn get_org_info(alias: &str) -> Result<SalesforceAuthResult, SalesforceAuthError> {
        // sf org display --target-org myorg --json
        let output = Command::new("sf")
            .args(["org", "display", "--target-org", alias, "--json"])
            .output()
            .map_err(|e| SalesforceAuthError::CliExecutionError(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SalesforceAuthError::OrgInfoError(stderr.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json_result: serde_json::Value = serde_json::from_str(&stdout)?;

        // 結果から必要な情報を抽出
        let result = json_result
            .get("result")
            .ok_or_else(|| SalesforceAuthError::OrgInfoError("結果が見つかりません".to_string()))?;

        let org_info = SalesforceOrgInfo {
            org_id: result
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            username: result
                .get("username")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            instance_url: result
                .get("instanceUrl")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            access_token: result
                .get("accessToken")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            api_version: result
                .get("apiVersion")
                .and_then(|v| v.as_str())
                .unwrap_or("v60.0")
                .to_string(),
            connected_status: result
                .get("connectedStatus")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        };

        Ok(SalesforceAuthResult {
            org_info,
            refresh_token: None, // CLIは内部で管理するため、直接取得できない
        })
    }

    /// アクセストークンをリフレッシュ
    pub async fn refresh_token(alias: &str) -> Result<String, SalesforceAuthError> {
        // sf org display --target-org myorg --json でアクセストークンを再取得
        let result = Self::get_org_info(alias).await?;
        Ok(result.org_info.access_token)
    }

    /// 組織一覧を取得
    pub async fn list_orgs() -> Result<Vec<String>, SalesforceAuthError> {
        let output = Command::new("sf")
            .args(["org", "list", "--json"])
            .output()
            .map_err(|e| SalesforceAuthError::CliExecutionError(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SalesforceAuthError::CliExecutionError(stderr.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json_result: serde_json::Value = serde_json::from_str(&stdout)?;

        // 組織のエイリアスを抽出
        let mut aliases = Vec::new();
        if let Some(orgs) = json_result
            .get("result")
            .and_then(|r| r.get("nonScratchOrgs"))
            .and_then(|o| o.as_array())
        {
            for org in orgs {
                if let Some(alias) = org.get("alias").and_then(|a| a.as_str()) {
                    aliases.push(alias.to_string());
                }
            }
        }

        Ok(aliases)
    }

    /// 組織からログアウト
    pub async fn logout(alias: &str) -> Result<(), SalesforceAuthError> {
        let output = Command::new("sf")
            .args(["org", "logout", "--target-org", alias, "--no-prompt"])
            .output()
            .map_err(|e| SalesforceAuthError::CliExecutionError(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SalesforceAuthError::CliExecutionError(stderr.to_string()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_cli_installed() {
        // このテストは実際のCLIインストール状況に依存するため、
        // 結果を確認するだけで、特定の値をアサートしない
        let result = SalesforceAuth::check_cli_installed();
        match result {
            Ok(installed) => {
                println!("Salesforce CLI installed: {installed}");
            }
            Err(e) => {
                println!("Error checking CLI: {e}");
            }
        }
    }
}

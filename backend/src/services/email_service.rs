use async_trait::async_trait;
use lettre::{
    message::Message, transport::smtp::AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

/// メール送信エラー
#[derive(Error, Debug)]
pub enum EmailError {
    #[error("メール送信に失敗しました: {0}")]
    Send(String),
    #[error("メールビルドに失敗しました: {0}")]
    Build(String),
    #[error("AWS SESエラー: {0}")]
    AwsSes(String),
    #[error("設定エラー: {0}")]
    Config(String),
}

/// メール送信リクエスト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMessage {
    pub to: Vec<String>,
    pub subject: String,
    pub html_body: String,
    pub text_body: Option<String>,
    pub reply_to: Option<String>,
    pub headers: Option<std::collections::HashMap<String, String>>,
}

/// メール送信結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailResult {
    pub message_id: String,
    pub status: EmailStatus,
    pub error: Option<String>,
}

/// メール送信ステータス
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EmailStatus {
    Sent,
    Failed,
    Queued,
}

/// メール送信プロバイダーのトレイト
#[async_trait]
pub trait EmailProvider: Send + Sync {
    /// 単一メール送信
    async fn send_email(&self, message: &EmailMessage) -> Result<EmailResult, EmailError>;

    /// バッチメール送信
    async fn send_batch(&self, messages: Vec<EmailMessage>)
        -> Result<Vec<EmailResult>, EmailError>;

    /// プロバイダー名を取得
    fn provider_name(&self) -> &str;
}

/// メール送信サービス設定
#[derive(Debug, Clone)]
pub struct EmailConfig {
    pub provider: EmailProviderType,
    pub from_email: String,
    pub from_name: Option<String>,
    pub smtp_config: Option<SmtpConfig>,
    pub aws_config: Option<AwsSesConfig>,
    pub rate_limit: u32,
    pub batch_size: usize,
}

/// メールプロバイダータイプ
#[derive(Debug, Clone, PartialEq)]
pub enum EmailProviderType {
    MailHog,
    AwsSes,
}

/// SMTP設定
#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub use_tls: bool,
}

/// AWS SES設定
#[derive(Debug, Clone)]
pub struct AwsSesConfig {
    pub region: String,
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
}

/// メール送信サービス
pub struct EmailService {
    provider: Arc<Box<dyn EmailProvider>>,
    config: EmailConfig,
}

impl EmailService {
    /// 新しいEmailServiceインスタンスを作成
    pub async fn new(config: EmailConfig) -> Result<Self, EmailError> {
        let provider: Box<dyn EmailProvider> = match &config.provider {
            EmailProviderType::MailHog => {
                let smtp_config = config
                    .smtp_config
                    .as_ref()
                    .ok_or_else(|| EmailError::Config("SMTP設定が必要です".to_string()))?;
                Box::new(MailHogProvider::new(
                    smtp_config.clone(),
                    config.from_email.clone(),
                )?)
            }
            EmailProviderType::AwsSes => {
                let aws_config = config
                    .aws_config
                    .as_ref()
                    .ok_or_else(|| EmailError::Config("AWS設定が必要です".to_string()))?;
                Box::new(AwsSesProvider::new(aws_config.clone(), config.from_email.clone()).await?)
            }
        };

        Ok(Self {
            provider: Arc::new(provider),
            config,
        })
    }

    /// 単一メール送信
    pub async fn send_email(&self, message: &EmailMessage) -> Result<EmailResult, EmailError> {
        self.provider.send_email(message).await
    }

    /// バッチメール送信（レート制限付き）
    pub async fn send_campaign(
        &self,
        recipients: Vec<EmailMessage>,
    ) -> Result<Vec<EmailResult>, EmailError> {
        let mut results = Vec::new();
        let chunks: Vec<_> = recipients.chunks(self.config.batch_size).collect();

        for chunk in chunks {
            let batch_results = self.provider.send_batch(chunk.to_vec()).await?;
            results.extend(batch_results);

            // レート制限のための待機
            if self.config.rate_limit > 0 {
                let delay = 1000 / self.config.rate_limit;
                tokio::time::sleep(tokio::time::Duration::from_millis(delay as u64)).await;
            }
        }

        Ok(results)
    }

    /// 環境変数から設定を読み込む
    pub fn from_env() -> Result<EmailConfig, EmailError> {
        let provider = match std::env::var("MAIL_PROVIDER")
            .unwrap_or_else(|_| "mailhog".to_string())
            .as_str()
        {
            "aws_ses" => EmailProviderType::AwsSes,
            _ => EmailProviderType::MailHog,
        };

        let from_email = std::env::var("SMTP_FROM")
            .or_else(|_| std::env::var("AWS_SES_FROM_EMAIL"))
            .map_err(|_| {
                EmailError::Config("送信元メールアドレスが設定されていません".to_string())
            })?;

        let smtp_config = if provider == EmailProviderType::MailHog {
            Some(SmtpConfig {
                host: std::env::var("SMTP_HOST").unwrap_or_else(|_| "localhost".to_string()),
                port: std::env::var("SMTP_PORT")
                    .unwrap_or_else(|_| "1025".to_string())
                    .parse()
                    .map_err(|_| EmailError::Config("無効なSMTPポート番号".to_string()))?,
                username: std::env::var("SMTP_USERNAME").ok(),
                password: std::env::var("SMTP_PASSWORD").ok(),
                use_tls: false,
            })
        } else {
            None
        };

        let aws_config = if provider == EmailProviderType::AwsSes {
            Some(AwsSesConfig {
                region: std::env::var("AWS_REGION")
                    .unwrap_or_else(|_| "ap-northeast-1".to_string()),
                access_key_id: std::env::var("AWS_ACCESS_KEY_ID").ok(),
                secret_access_key: std::env::var("AWS_SECRET_ACCESS_KEY").ok(),
            })
        } else {
            None
        };

        let rate_limit = std::env::var("EMAIL_RATE_LIMIT")
            .unwrap_or_else(|_| "14".to_string())
            .parse()
            .unwrap_or(14);

        let batch_size = std::env::var("EMAIL_BATCH_SIZE")
            .unwrap_or_else(|_| "50".to_string())
            .parse()
            .unwrap_or(50);

        Ok(EmailConfig {
            provider,
            from_email,
            from_name: None,
            smtp_config,
            aws_config,
            rate_limit,
            batch_size,
        })
    }
}

/// MailHogプロバイダー（開発環境用）
struct MailHogProvider {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from_email: String,
}

impl MailHogProvider {
    fn new(config: SmtpConfig, from_email: String) -> Result<Self, EmailError> {
        let transport = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.host)
            .port(config.port)
            .build();

        Ok(Self {
            transport,
            from_email,
        })
    }
}

#[async_trait]
impl EmailProvider for MailHogProvider {
    async fn send_email(&self, message: &EmailMessage) -> Result<EmailResult, EmailError> {
        if message.to.is_empty() {
            return Err(EmailError::Build("宛先が指定されていません".to_string()));
        }

        let to_address = message.to.first().unwrap();

        let mut email_builder = Message::builder()
            .from(
                self.from_email
                    .parse()
                    .map_err(|e| EmailError::Build(format!("無効な送信元アドレス: {}", e)))?,
            )
            .to(to_address
                .parse()
                .map_err(|e| EmailError::Build(format!("無効な宛先アドレス: {}", e)))?)
            .subject(&message.subject);

        if let Some(reply_to) = &message.reply_to {
            email_builder = email_builder.reply_to(
                reply_to
                    .parse()
                    .map_err(|e| EmailError::Build(format!("無効な返信先アドレス: {}", e)))?,
            );
        }

        let email = email_builder
            .multipart(
                lettre::message::MultiPart::alternative()
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(lettre::message::header::ContentType::TEXT_PLAIN)
                            .body(message.text_body.clone().unwrap_or_else(|| {
                                html2text::from_read(message.html_body.as_bytes(), 80)
                            })),
                    )
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(lettre::message::header::ContentType::TEXT_HTML)
                            .body(message.html_body.clone()),
                    ),
            )
            .map_err(|e| EmailError::Build(format!("メールビルドエラー: {}", e)))?;

        match self.transport.send(email).await {
            Ok(_response) => {
                // MailHogではメッセージIDは生成されない
                let message_id = uuid::Uuid::new_v4().to_string();
                Ok(EmailResult {
                    message_id,
                    status: EmailStatus::Sent,
                    error: None,
                })
            }
            Err(e) => Ok(EmailResult {
                message_id: "".to_string(),
                status: EmailStatus::Failed,
                error: Some(format!("送信エラー: {}", e)),
            }),
        }
    }

    async fn send_batch(
        &self,
        messages: Vec<EmailMessage>,
    ) -> Result<Vec<EmailResult>, EmailError> {
        let mut results = Vec::new();
        for message in messages {
            results.push(self.send_email(&message).await?);
        }
        Ok(results)
    }

    fn provider_name(&self) -> &str {
        "MailHog"
    }
}

/// AWS SESプロバイダー（本番環境用）
struct AwsSesProvider {
    client: aws_sdk_sesv2::Client,
    from_email: String,
}

impl AwsSesProvider {
    async fn new(config: AwsSesConfig, from_email: String) -> Result<Self, EmailError> {
        let aws_config = if let (Some(_access_key), Some(_secret_key)) = (
            config.access_key_id.as_ref(),
            config.secret_access_key.as_ref(),
        ) {
            aws_config::defaults(aws_config::BehaviorVersion::latest())
                .region(aws_config::Region::new(config.region.clone()))
                .load()
                .await
        } else {
            aws_config::defaults(aws_config::BehaviorVersion::latest())
                .region(aws_config::Region::new(config.region.clone()))
                .load()
                .await
        };

        let client = aws_sdk_sesv2::Client::new(&aws_config);

        Ok(Self { client, from_email })
    }
}

#[async_trait]
impl EmailProvider for AwsSesProvider {
    async fn send_email(&self, message: &EmailMessage) -> Result<EmailResult, EmailError> {
        use aws_sdk_sesv2::types::{
            Body, Content, Destination, EmailContent, Message as SesMessage,
        };

        if message.to.is_empty() {
            return Err(EmailError::Build("宛先が指定されていません".to_string()));
        }

        let destination = Destination::builder()
            .set_to_addresses(Some(message.to.clone()))
            .build();

        let subject_content = Content::builder()
            .data(&message.subject)
            .charset("UTF-8")
            .build()
            .map_err(|e| EmailError::Build(format!("件名ビルドエラー: {}", e)))?;

        let html_content = Content::builder()
            .data(&message.html_body)
            .charset("UTF-8")
            .build()
            .map_err(|e| EmailError::Build(format!("HTMLボディビルドエラー: {}", e)))?;

        let text_content = Content::builder()
            .data(message.text_body.as_deref().unwrap_or(""))
            .charset("UTF-8")
            .build()
            .map_err(|e| EmailError::Build(format!("テキストボディビルドエラー: {}", e)))?;

        let body = Body::builder()
            .html(html_content)
            .text(text_content)
            .build();

        let ses_message = SesMessage::builder()
            .subject(subject_content)
            .body(body)
            .build();

        let email_content = EmailContent::builder().simple(ses_message).build();

        let mut request = self
            .client
            .send_email()
            .from_email_address(&self.from_email)
            .destination(destination)
            .content(email_content);

        if let Some(reply_to) = &message.reply_to {
            request = request.reply_to_addresses(reply_to.clone());
        }

        match request.send().await {
            Ok(output) => {
                let message_id = output.message_id().unwrap_or("unknown").to_string();
                Ok(EmailResult {
                    message_id,
                    status: EmailStatus::Sent,
                    error: None,
                })
            }
            Err(e) => Ok(EmailResult {
                message_id: "".to_string(),
                status: EmailStatus::Failed,
                error: Some(format!("AWS SESエラー: {}", e)),
            }),
        }
    }

    async fn send_batch(
        &self,
        messages: Vec<EmailMessage>,
    ) -> Result<Vec<EmailResult>, EmailError> {
        // AWS SESのバッチ送信は別途実装が必要
        // 現在は単一送信を繰り返す
        let mut results = Vec::new();
        for message in messages {
            results.push(self.send_email(&message).await?);
        }
        Ok(results)
    }

    fn provider_name(&self) -> &str {
        "AWS SES"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_config_from_env() {
        std::env::set_var("MAIL_PROVIDER", "mailhog");
        std::env::set_var("SMTP_FROM", "test@example.com");
        std::env::set_var("SMTP_HOST", "localhost");
        std::env::set_var("SMTP_PORT", "1025");

        let config = EmailService::from_env().unwrap();
        assert_eq!(config.provider, EmailProviderType::MailHog);
        assert_eq!(config.from_email, "test@example.com");
        assert!(config.smtp_config.is_some());
    }

    #[tokio::test]
    async fn test_email_message_builder() {
        let message = EmailMessage {
            to: vec!["test@example.com".to_string()],
            subject: "テストメール".to_string(),
            html_body: "<h1>こんにちは</h1>".to_string(),
            text_body: Some("こんにちは".to_string()),
            reply_to: None,
            headers: None,
        };

        assert_eq!(message.to.len(), 1);
        assert_eq!(message.subject, "テストメール");
    }
}

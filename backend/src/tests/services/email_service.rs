use crate::services::email_service::{
    EmailConfig, EmailMessage, EmailProvider, EmailProviderType, EmailService, EmailStatus,
    SmtpConfig,
};

#[tokio::test]
async fn test_email_service_creation() {
    let config = EmailConfig {
        provider: EmailProviderType::MailHog,
        from_email: "test@example.com".to_string(),
        from_name: Some("Test Sender".to_string()),
        smtp_config: Some(SmtpConfig {
            host: "localhost".to_string(),
            port: 1025,
            username: None,
            password: None,
            use_tls: false,
        }),
        aws_config: None,
        rate_limit: 10,
        batch_size: 50,
    };

    let service = EmailService::new(config).await;
    assert!(service.is_ok());
}

#[tokio::test]
async fn test_email_message_validation() {
    let valid_message = EmailMessage {
        to: vec!["recipient@example.com".to_string()],
        subject: "Test Email".to_string(),
        html_body: "<h1>Hello</h1>".to_string(),
        text_body: Some("Hello".to_string()),
        reply_to: None,
        headers: None,
    };

    assert!(!valid_message.to.is_empty());
    assert!(!valid_message.subject.is_empty());
}

#[tokio::test]
async fn test_batch_email_creation() {
    let messages: Vec<EmailMessage> = (0..5)
        .map(|i| EmailMessage {
            to: vec![format!("user{}@example.com", i)],
            subject: format!("Test Email {}", i),
            html_body: format!("<h1>Hello User {}</h1>", i),
            text_body: Some(format!("Hello User {}", i)),
            reply_to: None,
            headers: None,
        })
        .collect();

    assert_eq!(messages.len(), 5);
    assert_eq!(messages[0].to[0], "user0@example.com");
    assert_eq!(messages[4].to[0], "user4@example.com");
}

#[test]
fn test_email_config_from_env() {
    // 環境変数を設定
    std::env::set_var("MAIL_PROVIDER", "mailhog");
    std::env::set_var("SMTP_FROM", "test@example.com");
    std::env::set_var("SMTP_HOST", "localhost");
    std::env::set_var("SMTP_PORT", "1025");
    std::env::set_var("EMAIL_RATE_LIMIT", "20");
    std::env::set_var("EMAIL_BATCH_SIZE", "100");

    let config = EmailService::from_env().unwrap();
    assert_eq!(config.provider, EmailProviderType::MailHog);
    assert_eq!(config.from_email, "test@example.com");
    assert_eq!(config.rate_limit, 20);
    assert_eq!(config.batch_size, 100);
}

#[test]
fn test_aws_ses_config_from_env() {
    // 前のテストの環境変数をクリア
    std::env::remove_var("SMTP_FROM");

    // AWS SES用の環境変数を設定
    std::env::set_var("MAIL_PROVIDER", "aws_ses");
    std::env::set_var("AWS_SES_FROM_EMAIL", "noreply@example.com");
    std::env::set_var("AWS_REGION", "us-east-1");

    let config = EmailService::from_env().unwrap();
    assert_eq!(config.provider, EmailProviderType::AwsSes);
    assert_eq!(config.from_email, "noreply@example.com");
    assert!(config.aws_config.is_some());

    let aws_config = config.aws_config.unwrap();
    assert_eq!(aws_config.region, "us-east-1");
}

// モックプロバイダーを使用した送信テスト
#[cfg(test)]
mod mock_tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    // テスト用のモックプロバイダー
    struct MockEmailProvider {
        sent_messages: Arc<Mutex<Vec<EmailMessage>>>,
    }

    impl MockEmailProvider {
        fn new() -> Self {
            Self {
                sent_messages: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn get_sent_messages(&self) -> Vec<EmailMessage> {
            self.sent_messages.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl EmailProvider for MockEmailProvider {
        async fn send_email(
            &self,
            message: &EmailMessage,
        ) -> Result<
            crate::services::email_service::EmailResult,
            crate::services::email_service::EmailError,
        > {
            self.sent_messages.lock().unwrap().push(message.clone());
            Ok(crate::services::email_service::EmailResult {
                message_id: uuid::Uuid::new_v4().to_string(),
                status: EmailStatus::Sent,
                error: None,
            })
        }

        async fn send_batch(
            &self,
            messages: Vec<EmailMessage>,
        ) -> Result<
            Vec<crate::services::email_service::EmailResult>,
            crate::services::email_service::EmailError,
        > {
            let mut results = Vec::new();
            for message in messages {
                results.push(self.send_email(&message).await?);
            }
            Ok(results)
        }

        fn provider_name(&self) -> &str {
            "Mock"
        }
    }

    #[tokio::test]
    async fn test_mock_email_sending() {
        let provider = MockEmailProvider::new();
        let message = EmailMessage {
            to: vec!["test@example.com".to_string()],
            subject: "Test".to_string(),
            html_body: "<p>Test</p>".to_string(),
            text_body: Some("Test".to_string()),
            reply_to: None,
            headers: None,
        };

        let result = provider.send_email(&message).await.unwrap();
        assert_eq!(result.status, EmailStatus::Sent);
        assert_eq!(provider.get_sent_messages().len(), 1);
    }
}

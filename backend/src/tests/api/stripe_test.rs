use crate::services::stripe_service::StripeService;
use stripe::SubscriptionStatus;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stripe_service_creation() {
        let service = StripeService::new(
            "sk_test_dummy_key".to_string(),
            "whsec_test_dummy_secret".to_string(),
        );

        // webhook_secretフィールドがprivateなので、
        // サービスが正常に作成されることのみを確認
        let _ = service; // 使用されていない警告を回避
    }

    #[test]
    fn test_subscription_status_conversion() {
        // 全てのサブスクリプションステータスが正しく変換されることを確認
        let test_cases = vec![
            (SubscriptionStatus::Active, "active"),
            (SubscriptionStatus::Canceled, "canceled"),
            (SubscriptionStatus::Incomplete, "incomplete"),
            (SubscriptionStatus::IncompleteExpired, "expired"),
            (SubscriptionStatus::PastDue, "past_due"),
            (SubscriptionStatus::Trialing, "trialing"),
            (SubscriptionStatus::Unpaid, "unpaid"),
            (SubscriptionStatus::Paused, "paused"),
        ];

        for (status, expected) in test_cases {
            assert_eq!(
                StripeService::convert_subscription_status(&status),
                expected,
                "Status {status:?} should convert to {expected}"
            );
        }
    }

    // Note: 実際のStripe APIを呼び出すテストは、
    // テスト用のStripeアカウントとAPIキーが必要です。
    // これらは統合テストまたはE2Eテストで実施すべきです。
}

#[cfg(test)]
mod api_endpoint_tests {

    #[tokio::test]
    async fn test_checkout_endpoint_requires_authentication() {
        // TODO: アプリケーションのテストヘルパーを使用して
        // 認証なしでcheckoutエンドポイントにアクセスした場合、
        // 401 Unauthorizedが返されることを確認
    }

    #[tokio::test]
    async fn test_webhook_endpoint_requires_signature() {
        // TODO: Stripe署名なしでwebhookエンドポイントにアクセスした場合、
        // 400 Bad Requestが返されることを確認
    }
}

use crate::ai::models::{
    ai_responses::{ContentContext, GenerateContentRequest, OptimizeSubjectRequest},
    GenerateScenarioRequest, Language,
};

#[cfg(test)]
mod tests {
    use super::*;

    // Note: 現時点では、AI API は外部依存があるため、
    // モックプロバイダーを使用した単体テストとして実装すべきです。
    // 統合テストは、実際のAPIキーが設定されている場合のみ実行されます。

    #[test]
    fn test_scenario_request_construction() {
        let request = GenerateScenarioRequest {
            industry: "SaaS".to_string(),
            target_audience: "スタートアップ企業".to_string(),
            goal: "無料トライアルから有料プランへの転換".to_string(),
            additional_context: Some("B2B向けのプロジェクト管理ツール".to_string()),
            language: Some(Language::Japanese),
        };

        assert_eq!(request.industry, "SaaS");
        assert_eq!(request.target_audience, "スタートアップ企業");
        assert!(request.additional_context.is_some());
    }

    #[test]
    fn test_content_request_construction() {
        let request = GenerateContentRequest {
            content_type: "email".to_string(),
            context: ContentContext {
                industry: Some("SaaS".to_string()),
                target_audience: Some("エンジニア".to_string()),
                tone: None,
                language: Some(Language::Japanese),
                existing_content: None,
            },
            options: None,
        };

        assert_eq!(request.content_type, "email");
        assert_eq!(request.context.industry, Some("SaaS".to_string()));
        assert_eq!(request.context.language, Some(Language::Japanese));
    }

    #[test]
    fn test_optimize_subject_request_construction() {
        let request = OptimizeSubjectRequest {
            original_subject: "新機能のお知らせ".to_string(),
            target_audience: "既存ユーザー".to_string(),
            campaign_goal: Some("エンゲージメント向上".to_string()),
            variations_count: Some(5),
            language: Some(Language::Japanese),
        };

        assert_eq!(request.original_subject, "新機能のお知らせ");
        assert_eq!(request.variations_count, Some(5));
    }
}

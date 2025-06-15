#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::ai::models::Language;

    #[test]
    fn test_get_scenario_system_prompt() {
        let ja_prompt = get_scenario_system_prompt(&Language::Japanese);
        assert!(ja_prompt.contains("マーケティング戦略家"));
        assert!(ja_prompt.contains("JSONのみで応答"));

        let en_prompt = get_scenario_system_prompt(&Language::English);
        assert!(en_prompt.contains("marketing strategist"));
        assert!(en_prompt.contains("valid JSON only"));
    }

    #[test]
    fn test_get_content_generation_system_prompt() {
        let ja_prompt = get_content_generation_system_prompt(&Language::Japanese);
        assert!(ja_prompt.contains("コピーライター"));
        assert!(ja_prompt.contains("マークダウン形式"));

        let en_prompt = get_content_generation_system_prompt(&Language::English);
        assert!(en_prompt.contains("copywriter"));
        assert!(en_prompt.contains("markdown format"));
    }

    #[test]
    fn test_generate_scenario_user_prompt() {
        let ja_prompt = generate_scenario_user_prompt(
            "SaaS",
            "スタートアップ",
            "リード獲得",
            Some("B2B向け"),
            &Language::Japanese,
        );
        assert!(ja_prompt.contains("業界: SaaS"));
        assert!(ja_prompt.contains("ターゲット層: スタートアップ"));
        assert!(ja_prompt.contains("目標: リード獲得"));
        assert!(ja_prompt.contains("追加の文脈: B2B向け"));

        let en_prompt = generate_scenario_user_prompt(
            "SaaS",
            "Startups",
            "Lead Generation",
            Some("B2B focused"),
            &Language::English,
        );
        assert!(en_prompt.contains("Industry: SaaS"));
        assert!(en_prompt.contains("Target Audience: Startups"));
        assert!(en_prompt.contains("Goal: Lead Generation"));
        assert!(en_prompt.contains("Additional Context: B2B focused"));
    }

    #[test]
    fn test_generate_subject_optimization_prompt() {
        let ja_prompt = generate_subject_optimization_prompt(
            "新製品のお知らせ",
            "エンジニア",
            &Language::Japanese,
        );
        assert!(ja_prompt.contains("エンジニア向けに最適化"));
        assert!(ja_prompt.contains("元の件名: 新製品のお知らせ"));

        let en_prompt = generate_subject_optimization_prompt(
            "New Product Announcement",
            "Engineers",
            &Language::English,
        );
        assert!(en_prompt.contains("optimize the following subject line for Engineers"));
        assert!(en_prompt.contains("Original subject: New Product Announcement"));
    }
}

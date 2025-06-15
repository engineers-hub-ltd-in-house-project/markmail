#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_language_from_code() {
        assert_eq!(Language::from_code("ja"), Some(Language::Japanese));
        assert_eq!(Language::from_code("JA"), Some(Language::Japanese));
        assert_eq!(Language::from_code("japanese"), Some(Language::Japanese));
        assert_eq!(Language::from_code("en"), Some(Language::English));
        assert_eq!(Language::from_code("EN"), Some(Language::English));
        assert_eq!(Language::from_code("english"), Some(Language::English));
        assert_eq!(Language::from_code("invalid"), None);
    }

    #[test]
    fn test_language_to_code() {
        assert_eq!(Language::Japanese.to_code(), "ja");
        assert_eq!(Language::English.to_code(), "en");
    }

    #[test]
    fn test_language_default() {
        assert_eq!(Language::default(), Language::Japanese);
    }

    #[test]
    fn test_language_serialization() {
        use serde_json;

        let ja = Language::Japanese;
        let json = serde_json::to_string(&ja).unwrap();
        assert_eq!(json, r#""ja""#);

        let en = Language::English;
        let json = serde_json::to_string(&en).unwrap();
        assert_eq!(json, r#""en""#);
    }

    #[test]
    fn test_language_deserialization() {
        use serde_json;

        let ja: Language = serde_json::from_str(r#""ja""#).unwrap();
        assert_eq!(ja, Language::Japanese);

        let en: Language = serde_json::from_str(r#""en""#).unwrap();
        assert_eq!(en, Language::English);
    }
}

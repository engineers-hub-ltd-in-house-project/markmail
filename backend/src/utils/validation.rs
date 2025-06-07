use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref EMAIL_REGEX: Regex = Regex::new(
        r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})"
    )
    .unwrap();
}

#[allow(dead_code)]
pub fn is_valid_email(email: &str) -> bool {
    EMAIL_REGEX.is_match(email)
}

#[allow(dead_code)]
pub fn is_strong_password(password: &str) -> bool {
    // 8文字以上であること
    if password.len() < 8 {
        return false;
    }

    // 最低限の複雑性チェック（文字種の混在）
    let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    // パスワードは少なくとも3種類の文字タイプを含む必要がある
    let char_types = [has_lowercase, has_uppercase, has_digit, has_special]
        .iter()
        .filter(|&&x| x)
        .count();

    char_types >= 3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_emails() {
        let valid_emails = vec![
            "test@example.com",
            "john.doe@example.co.jp",
            "info+123@engineers-hub.com",
            "admin@localhost.dev",
        ];

        for email in valid_emails {
            assert!(is_valid_email(email), "Email {} should be valid", email);
        }
    }

    #[test]
    fn test_invalid_emails() {
        let invalid_emails = vec![
            "",
            "plaintext",
            "@example.com",
            "user@",
            "user@domain",
            "user@.com",
            "user@domain..com",
            "<script>alert(1)</script>@example.com",
        ];

        for email in invalid_emails {
            assert!(!is_valid_email(email), "Email {} should be invalid", email);
        }
    }

    #[test]
    fn test_strong_passwords() {
        let strong_passwords = vec!["Passw0rd!", "StrongP@ss123", "C0mplex!Password", "8charsA+"];

        for password in strong_passwords {
            assert!(
                is_strong_password(password),
                "Password '{}' should be strong",
                password
            );
        }
    }

    #[test]
    fn test_weak_passwords() {
        let weak_passwords = vec![
            "",
            "short",
            "onlyletters",
            "ONLYUPPERCASE",
            "12345678",
            "!@#$%^&*()",
        ];

        for password in weak_passwords {
            assert!(
                !is_strong_password(password),
                "Password '{}' should be weak",
                password
            );
        }
    }
}

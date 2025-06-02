use bcrypt::{hash, verify, DEFAULT_COST};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PasswordError {
    #[error("パスワードのハッシュ化に失敗しました")]
    HashError(#[from] bcrypt::BcryptError),
}

/// パスワードをハッシュ化する
pub fn hash_password(password: &str) -> Result<String, PasswordError> {
    let hashed = hash(password, DEFAULT_COST)?;
    Ok(hashed)
}

/// パスワードを検証する
pub fn verify_password(password: &str, hash: &str) -> Result<bool, PasswordError> {
    let is_valid = verify(password, hash)?;
    Ok(is_valid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let password = "test_password123";
        let hashed = hash_password(password).unwrap();

        // ハッシュが生成されることを確認
        assert!(!hashed.is_empty());

        // 同じパスワードでも異なるハッシュが生成される
        let hashed2 = hash_password(password).unwrap();
        assert_ne!(hashed, hashed2);
    }

    #[test]
    fn test_verify_password() {
        let password = "test_password123";
        let hashed = hash_password(password).unwrap();

        // 正しいパスワードの検証
        assert!(verify_password(password, &hashed).unwrap());

        // 間違ったパスワードの検証
        assert!(!verify_password("wrong_password", &hashed).unwrap());
    }
}

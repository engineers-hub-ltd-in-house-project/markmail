use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum JwtError {
    #[error("JWTエンコードエラー: {0}")]
    EncodeError(#[from] jsonwebtoken::errors::Error),

    #[error("JWT秘密鍵が設定されていません")]
    SecretNotSet,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // ユーザーID
    pub email: String,
    pub name: String,
    pub exp: i64, // 有効期限
    pub iat: i64, // 発行日時
}

impl Claims {
    pub fn new(user_id: Uuid, email: String, name: String, expiration_hours: i64) -> Self {
        let now = Utc::now();
        let exp = now + Duration::hours(expiration_hours);

        Self {
            sub: user_id.to_string(),
            email,
            name,
            exp: exp.timestamp(),
            iat: now.timestamp(),
        }
    }
}

/// JWTトークンを生成する
pub fn generate_token(claims: &Claims) -> Result<String, JwtError> {
    let secret = match std::env::var("JWT_SECRET") {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("JWT_SECRET環境変数が設定されていません: {:?}", e);
            return Err(JwtError::SecretNotSet);
        }
    };

    tracing::debug!("JWTトークンをエンコードします");
    let token = match encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(secret.as_ref()),
    ) {
        Ok(token) => token,
        Err(e) => {
            tracing::error!("JWTエンコードエラー: {:?}", e);
            return Err(JwtError::EncodeError(e));
        }
    };

    tracing::debug!("JWTトークンエンコード成功");
    Ok(token)
}

/// JWTトークンを検証し、クレームを取得する
pub fn verify_token(token: &str) -> Result<TokenData<Claims>, JwtError> {
    let secret = std::env::var("JWT_SECRET").map_err(|_| JwtError::SecretNotSet)?;
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )?;
    Ok(token_data)
}

/// リフレッシュトークンを生成する（単純なランダム文字列）
pub fn generate_refresh_token() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    let mut rng = rand::thread_rng();

    (0..64)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_refresh_token() {
        let token1 = generate_refresh_token();
        let token2 = generate_refresh_token();

        // トークンが64文字であることを確認
        assert_eq!(token1.len(), 64);
        assert_eq!(token2.len(), 64);

        // 異なるトークンが生成されることを確認
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_jwt_generation_and_verification() {
        // テスト用のJWT秘密鍵を設定
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_only");

        let user_id = Uuid::new_v4();
        let claims = Claims::new(
            user_id,
            "test@example.com".to_string(),
            "Test User".to_string(),
            24, // 24時間
        );

        let token = generate_token(&claims).unwrap();
        assert!(!token.is_empty());

        let verified = verify_token(&token).unwrap();
        assert_eq!(verified.claims.sub, user_id.to_string());
        assert_eq!(verified.claims.email, "test@example.com");
    }
}

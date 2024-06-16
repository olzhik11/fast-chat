use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
    http::{HeaderName, HeaderValue},
    RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use chrono;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::{
    errors::{AppError, AppErrorType},
    graphql::user::schema::SessionUser,
    startup::AppState,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
    pub email: String,
}

impl Claims {
    // time to live, in minutes
    pub fn new(session_user: &SessionUser, ttl: i64) -> Self {
        let now = chrono::Utc::now();

        Self {
            sub: session_user.id.into(),
            exp: (now + chrono::Duration::minutes(ttl)).timestamp(),
            iat: now.timestamp(),
            email: session_user.email.to_string(),
        }
    }
}

pub fn encode_token(claims: &Claims) -> Result<String, AppError> {
    let encoding_key = &EncodingKey::from_rsa_pem(include_bytes!("../../keys/private.pem"))
        .map_err(|e| {
            AppError::new(
                "Key encoding error.".to_string(),
                AppErrorType::TokenEncodingError(e),
            )
        })?;

    encode(&Header::new(Algorithm::RS256), claims, &encoding_key).map_err(|e| {
        AppError::new(
            "Token encoding error.".to_string(),
            AppErrorType::TokenEncodingError(e),
        )
    })
}

pub fn decode_token(token: &str) -> Result<Claims, AppError> {
    let decoding_key =
        DecodingKey::from_rsa_pem(include_bytes!("../../keys/public.pem")).map_err(|e| {
            AppError::new(
                "Key decoding error.".to_string(),
                AppErrorType::TokenEncodingError(e),
            )
        })?;

    decode::<Claims>(token, &decoding_key, &Validation::new(Algorithm::RS256))
        .map(|v| v.claims)
        .map_err(|e| {
            AppError::new(
                "Token decoding error.".to_string(),
                AppErrorType::TokenEncodingError(e),
            )
        })
}

pub fn verify_token(token: Option<&str>) -> Result<Claims, AppError> {
    decode_token(token.unwrap_or_default())
}

pub fn get_auth_header_pair(token: String) -> (HeaderName, HeaderValue) {
    let bt = format!("Bearer {token}");
    (
        HeaderName::from_lowercase(b"authorization").unwrap(),
        HeaderValue::from_str(&bt).unwrap(),
    )
}

#[async_trait]
impl FromRequestParts<AppState> for Claims {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        match parts.extract::<TypedHeader<Authorization<Bearer>>>().await {
            Ok(TypedHeader(Authorization(bearer))) => decode_token(bearer.token()),
            Err(e) => Err(AppError::new(
                e.to_string(),
                AppErrorType::AuthorizationError(format!(
                    "No authorization header is present: {}",
                    e.to_string()
                )),
            )),
        }
    }
}

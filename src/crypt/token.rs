use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header, request::Parts, HeaderName, HeaderValue},
    RequestPartsExt,
};

use axum_extra::{headers::Cookie, TypedHeader};
use chrono;
use cookie;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppErrorType},
    sql::user::SessionUser,
    startup::AppState,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,
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
    let cookie = cookie::Cookie::build(("auth_token", token))
        .http_only(true)
        .same_site(cookie::SameSite::Lax)
        .secure(true)
        .path("/")
        .build();

    let cookie_str = cookie.to_string();

    (
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie_str).unwrap(),
    )
}

#[async_trait]
impl FromRequestParts<AppState> for Claims {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        match parts.extract::<TypedHeader<Cookie>>().await {
            Ok(TypedHeader(cookie)) => {
                info!("{:?}", cookie);
                if let Some(token) = cookie.get("auth_token") {
                    decode_token(token)
                } else {
                    Err(AppError::new(
                        "No auth_token cookie is present".to_string(),
                        AppErrorType::AuthorizationError(
                            "No auth_token cookie is present".to_string(),
                        ),
                    ))
                }
            }
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

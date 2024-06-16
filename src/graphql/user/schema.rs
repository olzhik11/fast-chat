use crate::{
    crypt::hash::hash_password,
    errors::AppError,
    graphql::user::validators::{UserEmail, UserName},
};
use derivative::{self, Derivative};
use juniper::{GraphQLInputObject, GraphQLObject};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct SessionUser<'a> {
    pub id: Uuid,
    pub email: &'a str,
}

#[derive(GraphQLInputObject, Debug)]
pub struct UserUpdate {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, GraphQLObject, FromRow, Derivative)]
#[derivative(Default)]
pub struct User {
    #[derivative(Default(value = "Uuid::new_v4()"))]
    pub id: Uuid,
    pub name: String,
    pub email: String,
    #[graphql(ignore)]
    pub password: String,
    #[derivative(Default(value = "chrono::Utc::now()"))]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[derivative(Default(value = "chrono::Utc::now()"))]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize, Serialize, GraphQLInputObject, Debug)]
pub struct UserInput {
    pub name: String,
    pub email: String,
    pub password: String,
}

impl UserInput {
    pub fn validate_user_input(self) -> Result<Self, AppError> {
        let name = UserName::parse(self.name)?;
        let email = UserEmail::parse(self.email)?;

        let hash = hash_password(self.password).expect("Failed to hash password.");

        Ok(UserInput {
            name: name.inner(),
            email: email.inner(),
            password: hash,
        })
    }
}

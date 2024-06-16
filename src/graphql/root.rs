use crate::{
    crypt::token::Claims,
    graphql::user::schema::{User, UserUpdate},
    sql::user::{get_user, update_user},
};
use juniper::{Context, EmptySubscription, FieldResult, IntoFieldError, RootNode};
use sqlx::PgPool;
use uuid::timestamp::context;

use super::room::schema::Room;

pub struct GraphQLContext {
    pub pool: PgPool,
    pub claims: Claims,
}

impl Context for GraphQLContext {}

impl GraphQLContext {
    pub fn new(pool: PgPool, claims: Claims) -> Self {
        GraphQLContext { pool, claims }
    }
}

pub struct QueryRoot;

#[juniper::graphql_object(Context = GraphQLContext, name = "Query")]
impl QueryRoot {
    #[graphql(description = "Getting a single user based on id.")]
    async fn user(context: &GraphQLContext) -> FieldResult<User> {
        get_user(&context.pool, &context.claims.email)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[graphql(description = "Getting user rooms.")]
    async fn rooms(context: &GraphQLContext) -> FieldResult<Vec<Room>> {
        todo!()
    }
}

pub struct MutationRoot;

#[juniper::graphql_object(Context = GraphQLContext, name = "Mutation")]
impl MutationRoot {
    #[graphql(description = "Updating user name based on id.")]
    pub async fn update_user(context: &GraphQLContext, user: UserUpdate) -> FieldResult<User> {
        update_user(&context.pool, user)
            .await
            .map_err(|e| e.into_field_error())
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot, EmptySubscription<GraphQLContext>>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot, MutationRoot, EmptySubscription::new())
}

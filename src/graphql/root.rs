use crate::{
    crypt::token::Claims,
    graphql::{room::schema::Room, user::schema::{User, UserUpdate}},
    sql::{room::{create_room, get_rooms}, user::{get_user, update_user}},
};
use juniper::{Context, EmptySubscription, FieldResult, IntoFieldError, RootNode};
use sqlx::PgPool;
use uuid::Uuid;

use super::room::schema::RoomInput;


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
    #[graphql(description = "Get a single user based on id.")]
    async fn user(context: &GraphQLContext) -> FieldResult<User> {
        get_user(&context.pool, &context.claims.email)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[graphql(description = "Get user rooms.")]
    async fn rooms(context: &GraphQLContext, id: Uuid) -> FieldResult<Vec<Room>> {
        get_rooms(&context.pool, id).await.map_err(|e| e.into_field_error())
    }
}

pub struct MutationRoot;

#[juniper::graphql_object(Context = GraphQLContext, name = "Mutation")]
impl MutationRoot {
    #[graphql(description = "Update user name based on id.")]
    pub async fn update_user(context: &GraphQLContext, user: UserUpdate) -> FieldResult<User> {
        update_user(&context.pool, user)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[graphql(description = "Create room.")]
    pub async fn create_room(context: &GraphQLContext, room: RoomInput, user_id: Uuid) -> FieldResult<Room> {
        create_room(&context.pool, room, user_id).await.map_err(|e| e.into_field_error())
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot, EmptySubscription<GraphQLContext>>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot, MutationRoot, EmptySubscription::new())
}

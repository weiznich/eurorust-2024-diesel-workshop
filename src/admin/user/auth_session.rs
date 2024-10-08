//! Authentication setup for our application
use super::Credentials;
use crate::database::schema::users;
use crate::database::Id;
use crate::errors::Result;
use argon2::Argon2;
use argon2::PasswordHash;
use argon2::PasswordVerifier;
use axum_login::AuthUser;
use axum_login::AuthnBackend;
use axum_login::UserId;
use diesel::prelude::*;
use diesel::sqlite::Sqlite;

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(check_for_backend(Sqlite))]
pub struct User {
    pub(crate) id: Id,
    pub(crate) password: String,
}

impl AuthUser for User {
    type Id = Id;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password.as_bytes()
    }
}

#[derive(Clone)]
pub struct LoginBackend {
    pub(crate) pool: deadpool_diesel::sqlite::Pool,
}

impl LoginBackend {
    pub fn new(pool: deadpool_diesel::sqlite::Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl AuthnBackend for LoginBackend {
    type User = User;
    type Credentials = Credentials;
    type Error = crate::errors::Error;

    async fn authenticate(
        &self,
        Credentials { name, password }: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let user = self
            .pool
            .get()
            .await?
            .interact(|conn| {
                users::table
                    .filter(users::name.eq(name))
                    .select(User::as_select())
                    .first(conn)
                    .optional()
            })
            .await??;
        if let Some(user) = user {
            if Argon2::default()
                .verify_password(password.as_bytes(), &PasswordHash::new(&user.password)?)
                .is_ok()
            {
                Ok(Some(user))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let user_id = *user_id;
        Ok(self
            .pool
            .get()
            .await?
            .interact(move |conn| {
                users::table
                    .find(user_id)
                    .select(User::as_select())
                    .first(conn)
                    .optional()
            })
            .await??)
    }
}

pub(crate) type AuthSession = axum_login::AuthSession<LoginBackend>;

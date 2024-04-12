use super::checker::Team;
use crate::{checker::Config, ConfigState};
use async_trait::async_trait;
use axum_login::*;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Default)]
pub struct Auth {
    users: HashMap<Uuid, TeamUser>,
}

#[derive(Debug, Clone)]
pub struct TeamUser(pub Uuid,pub String);

static ADMIN_ID: Uuid = Uuid::from_u128(0x14298410567319418293721489124109);

impl TeamUser {
    pub fn admin() -> Self {
        TeamUser(
            ADMIN_ID,
            "admin".to_string()
        )
    }
    pub fn is_admin(&self) -> bool {
        self.0 == ADMIN_ID
    }
}

impl AuthUser for TeamUser {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.0
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl From<&Team> for TeamUser {
    fn from(team: &Team) -> Self {
        Self(team.id,team.name.clone())
    }
}

#[derive(Clone)]
pub struct TeamCredentials {
    pub config: ConfigState,
    pub name: String,
    pub password: String,
}

#[async_trait]
impl AuthnBackend for Auth {
    type User = TeamUser;
    type Credentials = TeamCredentials;
    type Error = std::convert::Infallible;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        if creds.name == "admin" {
            if Config::check_admin_password(&creds.password) {
                return Ok(Some(TeamUser::admin()));
            }
            return Ok(None);
        }
        let conf = creds.config.read().await;
        Ok(conf
            .get_team_with_password(&creds.name, &creds.password)
            .map(|t| t.into()))
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        Ok(self.users.get(user_id).cloned())
    }
}

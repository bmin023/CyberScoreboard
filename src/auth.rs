use super::checker::Team;
use crate::ConfigState;
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
    config: ConfigState,
    name: String,
    password: String,
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
        let conf = creds.config.read().await;
        Ok(conf
            .get_team_with_password(&creds.name, &creds.password)
            .map(|t| t.into()))
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        Ok(self.users.get(user_id).cloned())
    }
}

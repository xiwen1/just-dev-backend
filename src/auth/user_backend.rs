use axum::async_trait;
use axum_login::{AuthUser, AuthnBackend, AuthzBackend, UserId};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

use super::User;

#[derive(Clone, Default)]
pub struct UserBackend {
    users: HashMap<i64, User>,
}

#[derive(Deserialize, Clone)]
pub struct Credentials {
    pub user_id: i64,
    pub password: String,
}

#[async_trait]
impl AuthnBackend for UserBackend {
    type User = User;
    type Credentials = Credentials;
    type Error = std::convert::Infallible;

    async fn authenticate(
        &self,
        Credentials { user_id, password }: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        match self.users.get(&user_id) {
            Some(user) => {
                if password.as_bytes() == user.session_auth_hash() {
                    Ok(Some(user.clone()))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        Ok(self.users.get(user_id).cloned())
    }
}

#[async_trait]
impl AuthzBackend for UserBackend {
    type Permission = String;

    async fn get_user_permissions(
        &self,
        _user: &Self::User,
    ) -> Result<HashSet<Self::Permission>, Self::Error> {
        let permissions = HashSet::from_iter(_user.doc_ids.iter().map(|id| id.to_string()));
        Ok(permissions)
    }
}

impl UserBackend {
    pub fn new() -> Self {
        let mut users = HashMap::new();
        users.insert(1, User::new(1, &[1]));
        users.insert(2, User::new(2, &[2]));
        users.insert(3, User::new(3, &[1, 2]));
        users.insert(4, User::new(4, &[]));
        UserBackend { users }
    }
}

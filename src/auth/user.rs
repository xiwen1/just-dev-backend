use axum_login::AuthUser;
use serde::Deserialize;



#[derive(Deserialize, Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub doc_ids: Vec<i64>,
    pub pw_hash: Vec<u8>,
}

impl AuthUser for User {
    type Id = i64;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        &self.pw_hash
    }
}

impl User {
    pub fn new(id: i64, doc_ids:&[i64]) -> Self {
        User {
            id, 
            doc_ids: Vec::from(doc_ids),
            pw_hash: "password".as_bytes().to_vec(),
        }
    }
}

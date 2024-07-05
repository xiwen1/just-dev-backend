use std::{collections::HashMap, sync::Arc};


use axum_ycrdt_websocket::{broadcast::BroadcastGroup, AwarenessRef};
use tokio::sync::RwLock;
use yrs::{sync::Awareness, Doc, Text, Transact};

#[derive(Clone, Default)]
pub struct DocumentRepository {
    docs: HashMap<i64, String>,
}

#[derive(Clone)]
pub struct BroadcastMap {
    rooms: HashMap<i64, Arc<BroadcastGroup>>,
}

impl BroadcastMap {
    pub async fn get_room(
        &mut self,
        doc_id: i64,
        doc_repo: Arc<DocumentRepository>,
    ) -> Option<Arc<BroadcastGroup>> {
        match self.rooms.get(&doc_id) {
            Some(bcast) => Some(bcast.clone()),
            None => match doc_repo.docs.get(&doc_id) {
                Some(doc) => {
                    let bcast = new_bcast(doc_id, doc).await;
                    self.rooms.insert(doc_id, bcast.clone());
                    Some(bcast)
                }
                None => None,
            },
        }
    }

    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
        }
    }
}

async fn new_bcast(doc_id: i64, content: &str) -> Arc<BroadcastGroup> {
    let awareness: AwarenessRef = {
        let doc = Doc::new();
        {
            let txt = doc.get_or_insert_text(format!("{}", doc_id));
            let mut txn = doc.transact_mut();
            txt.push(&mut txn, content);
        }
        Arc::new(RwLock::new(Awareness::new(doc)))
    };
    Arc::new(BroadcastGroup::new(awareness.clone(), 32).await)
}

impl DocumentRepository {
    pub fn new() -> Self {
        let mut docs = HashMap::new();
        docs.insert(1, "hello".into());
        docs.insert(2, "world".into());
        DocumentRepository { docs }
    }
}

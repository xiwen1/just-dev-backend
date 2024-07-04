use std::collections::HashMap;

use yrs::Doc;


#[derive(Clone, Default)]
pub struct DocBackend {
    docs: HashMap<i64, Doc>,
}

impl DocBackend {
    fn new() -> Self {
        let mut docs = HashMap::new();
        docs.insert(1, Doc::new());
        docs.insert(2, Doc::new());
        DocBackend {docs}
    }
}
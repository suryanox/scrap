use std::collections::HashMap;
use crate::trash::TrashItem;

pub struct Scrap {
    pub items: HashMap<String, TrashItem>
}

impl Scrap {
    pub fn new() -> Scrap {
        Scrap { items: HashMap::new() }
    }

    pub fn add_item(&mut self, item: TrashItem) {
        self.items.insert(item.name.clone(), item);
    }
}

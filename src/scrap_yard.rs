use std::collections::HashMap;
use std::os::unix::fs::MetadataExt;
use crate::item::ItemType;
use crate::scrap::Scrap;
use crate::trash::TrashItem;
pub struct ScrapYard {
    junk: HashMap<ItemType, Scrap>
}

impl ScrapYard {
    fn add_junk(&mut self, item: TrashItem) {
        self.junk
            .entry(item.item_type.clone())
            .or_insert_with(Scrap::new)
            .add_item(item);
    }

    pub fn get_type_count(&self, item_type: &ItemType) -> usize {
        self.junk
            .get(item_type)
            .map(|scrap| scrap.items.len())
            .unwrap_or(0)
    }

    pub fn get_item_at(&self, item_type: &ItemType, index: usize) -> Option<&TrashItem> {
        self.junk
            .get(item_type)
            .and_then(|scrap| scrap.items.values().nth(index))
    }

    pub fn iter_items(&self, item_type: &ItemType) -> impl Iterator<Item = &TrashItem> {
        self.junk
            .get(item_type)
            .into_iter()
            .flat_map(|scrap| scrap.items.values())
    }


    pub fn build() -> ScrapYard {
        let mut scrapy =  ScrapYard { junk: HashMap::new() };
        let home = std::env::var("HOME").unwrap_or_default();
        let trash_path = format!("{}/.Trash", home);

        if let Ok(entries) = std::fs::read_dir(&trash_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                if name.starts_with('.') {
                    continue;
                }

                let is_dir = path.is_dir();

                let ext = path.extension()
                    .map(|e| e.to_string_lossy().to_string())
                    .unwrap_or_default();

                let item_type = ItemType::from_extension(&ext, is_dir);

                let deleted_at = entry.metadata()
                    .and_then(|m| m.modified())
                    .map(|t| {
                        let datetime: chrono::DateTime<chrono::Local> = t.into();
                        datetime.format("%Y-%m-%d %H:%M").to_string()
                    })
                    .unwrap_or_else(|_| "Unknown".to_string());

                scrapy.add_junk(TrashItem {
                    name,
                    original_path: path.to_string_lossy().to_string(),
                    deleted_at,
                    item_type,
                    size: entry.metadata().unwrap().size()
                });
            }
        }
        scrapy
    }
}
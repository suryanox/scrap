use crate::item::ItemType;

#[derive(Debug, Clone)]
pub struct TrashItem {
    pub name: String,
    pub original_path: String,
    pub deleted_at: String,
    pub item_type: ItemType,
    pub size: u64
}
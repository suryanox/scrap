use std::os::unix::fs::MetadataExt;
use ratatui::widgets::{ListState, ScrollbarState};
use crate::item::ItemType;
use crate::trash::TrashItem;

pub struct App {
    pub items: Vec<TrashItem>,
    pub filtered_items: Vec<TrashItem>,
    pub list_state: ListState,
    pub selected_tab: usize,
    pub tabs: Vec<ItemType>,
    #[allow(dead_code)]
    pub query: String,
    pub should_quit: bool,
    pub scroll_state: ScrollbarState,
}

impl App {
    pub fn new() -> Self {
        let items = Self::load_trash_items();
        let tabs = vec![
            ItemType::Folder,
            ItemType::Code,
            ItemType::Document,
            ItemType::Image,
            ItemType::Video,
            ItemType::Audio,
            ItemType::Archive,
            ItemType::Other,
        ];

        let mut app = App {
            items,
            filtered_items: Vec::new(),
            list_state: ListState::default(),
            selected_tab: 0,
            tabs,
            query: String::new(),
            should_quit: false,
            scroll_state: ScrollbarState::default(),
        };
        app.filter_items();
        if !app.filtered_items.is_empty() {
            app.list_state.select(Some(0));
        }
        app
    }

    pub fn load_trash_items() -> Vec<TrashItem> {
        let mut items = Vec::new();

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

                items.push(TrashItem {
                    name,
                    original_path: path.to_string_lossy().to_string(),
                    deleted_at,
                    item_type,
                    size: entry.metadata().unwrap().size()
                });
            }
        }

        items.sort_by(|a, b| b.deleted_at.cmp(&a.deleted_at));
        items
    }

    pub fn get_type_count(&self, item_type: &ItemType) -> usize {
        self.items
            .iter()
            .filter(|item| &item.item_type == item_type)
            .count()
    }

    pub fn filter_items(&mut self) {
        let selected_type = &self.tabs[self.selected_tab];
        self.filtered_items = self.items
            .iter()
            .filter(|item| &item.item_type == selected_type)
            .cloned()
            .collect();
        self.scroll_state = self.scroll_state.content_length(self.filtered_items.len());
    }

    pub fn next_tab(&mut self) {
        self.selected_tab = (self.selected_tab + 1) % self.tabs.len();
        self.filter_items();
        self.list_state.select(if self.filtered_items.is_empty() { None } else { Some(0) });
        self.scroll_state = self.scroll_state.position(0);
    }

    pub fn prev_tab(&mut self) {
        self.selected_tab = if self.selected_tab == 0 {
            self.tabs.len() - 1
        } else {
            self.selected_tab - 1
        };
        self.filter_items();
        self.list_state.select(if self.filtered_items.is_empty() { None } else { Some(0) });
        self.scroll_state = self.scroll_state.position(0);
    }

    pub fn next_item(&mut self) {
        if self.filtered_items.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.filtered_items.len() - 1 { 0 } else { i + 1 }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i);
    }

    pub fn prev_item(&mut self) {
        if self.filtered_items.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 { self.filtered_items.len() - 1 } else { i - 1 }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i);
    }
}
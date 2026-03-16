use ratatui::widgets::{ListState, ScrollbarState};
use crate::item::ItemType;
use crate::scrap_yard::ScrapYard;
use crate::trash::TrashItem;

pub struct App {
    pub scrap_yard: ScrapYard,
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
            scrap_yard: ScrapYard::build(),
            list_state: ListState::default(),
            selected_tab: 0,
            tabs,
            query: String::new(),
            should_quit: false,
            scroll_state: ScrollbarState::default(),
        };
        app.reset_selection();
        app
    }

    pub fn current_item_type(&self) -> &ItemType {
        &self.tabs[self.selected_tab]
    }

    pub fn current_items_count(&self) -> usize {
        self.scrap_yard.get_type_count(self.current_item_type())
    }

    pub fn get_item_at(&self, index: usize) -> Option<&TrashItem> {
        self.scrap_yard.get_item_at(self.current_item_type(), index)
    }

    pub fn get_selected_item(&self) -> Option<&TrashItem> {
        self.list_state.selected().and_then(|i| self.get_item_at(i))
    }

    fn reset_selection(&mut self) {
        let count = self.current_items_count();
        self.list_state.select(if count == 0 { None } else { Some(0) });
        self.scroll_state = self.scroll_state.position(0);
    }

    pub fn next_tab(&mut self) {
        self.selected_tab = (self.selected_tab + 1) % self.tabs.len();
        self.reset_selection();
    }

    pub fn prev_tab(&mut self) {
        self.selected_tab = if self.selected_tab == 0 {
            self.tabs.len() - 1
        } else {
            self.selected_tab - 1
        };
        self.reset_selection();
    }

    pub fn next_item(&mut self) {
        let count = self.current_items_count();
        if count == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => if i >= count - 1 { 0 } else { i + 1 },
            None => 0,
        };
        self.list_state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i);
    }

    pub fn prev_item(&mut self) {
        let count = self.current_items_count();
        if count == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => if i == 0 { count - 1 } else { i - 1 },
            None => 0,
        };
        self.list_state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i);
    }
}
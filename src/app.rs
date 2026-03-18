use ratatui::widgets::{ListState, ScrollbarState};
use crate::item::ItemType;
use crate::scrap_yard::ScrapYard;
use crate::trash::TrashItem;

#[derive(Debug, Clone, PartialEq)]
pub enum QueryCommand {
    DeleteAll,
    DeleteByType(ItemType),
    Invalid,
    Empty,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Query,
}

pub struct App {
    pub scrap_yard: ScrapYard,
    pub list_state: ListState,
    pub selected_tab: usize,
    pub tabs: Vec<ItemType>,
    pub query: String,
    pub query_command: QueryCommand,
    pub input_mode: InputMode,
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
            query_command: QueryCommand::Empty,
            input_mode: InputMode::Normal,
            should_quit: false,
            scroll_state: ScrollbarState::default(),
        };
        app.reset_selection();
        app
    }

    pub fn enter_query_mode(&mut self) {
        self.input_mode = InputMode::Query;
    }

    pub fn exit_query_mode(&mut self) {
        self.input_mode = InputMode::Normal;
        self.query.clear();
        self.query_command = QueryCommand::Empty;
    }

    pub fn query_push(&mut self, c: char) {
        self.query.push(c);
        self.validate_query();
    }

    pub fn query_pop(&mut self) {
        self.query.pop();
        self.validate_query();
    }

    fn validate_query(&mut self) {
        let q = self.query.trim().to_lowercase();
        
        if q.is_empty() {
            self.query_command = QueryCommand::Empty;
            return;
        }

        if q == "delete all" {
            self.query_command = QueryCommand::DeleteAll;
            return;
        }

        if q.starts_with("delete ") {
            let type_str = q.strip_prefix("delete ").unwrap_or("");
            if let Some(item_type) = Self::parse_item_type(type_str) {
                self.query_command = QueryCommand::DeleteByType(item_type);
                return;
            }
        }

        self.query_command = QueryCommand::Invalid;
    }

    fn parse_item_type(s: &str) -> Option<ItemType> {
        match s.trim().to_lowercase().as_str() {
            "folders" | "folder" => Some(ItemType::Folder),
            "code" => Some(ItemType::Code),
            "docs" | "documents" | "document" => Some(ItemType::Document),
            "images" | "image" => Some(ItemType::Image),
            "video" | "videos" => Some(ItemType::Video),
            "audio" => Some(ItemType::Audio),
            "archives" | "archive" => Some(ItemType::Archive),
            "other" => Some(ItemType::Other),
            _ => None,
        }
    }

    pub fn is_query_valid(&self) -> bool {
        matches!(self.query_command, QueryCommand::DeleteAll | QueryCommand::DeleteByType(_))
    }

    pub fn run_query(&mut self) {
        match &self.query_command {
            QueryCommand::DeleteAll => {
                self.scrap_yard.delete_all();
                self.reset_selection();
            }
            QueryCommand::DeleteByType(item_type) => {
                self.scrap_yard.delete_by_type(item_type);
                self.reset_selection();
            }
            _ => {}
        }
        self.exit_query_mode();
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
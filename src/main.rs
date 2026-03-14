use std::io;
use std::os::unix::prelude::MetadataExt;
use std::time::Duration;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame, Terminal,
};

const BG: Color = Color::Rgb(13, 17, 23);
const SURFACE: Color = Color::Rgb(22, 27, 34);
const SURFACE_BRIGHT: Color = Color::Rgb(33, 38, 45);
const BORDER: Color = Color::Rgb(48, 54, 61);
const BORDER_HIGHLIGHT: Color = Color::Rgb(88, 166, 255);
const TEXT: Color = Color::Rgb(230, 237, 243);
const TEXT_DIM: Color = Color::Rgb(125, 133, 144);
const ACCENT: Color = Color::Rgb(88, 166, 255);
const GREEN: Color = Color::Rgb(63, 185, 80);
const YELLOW: Color = Color::Rgb(210, 153, 34);
const PURPLE: Color = Color::Rgb(163, 113, 247);
const CYAN: Color = Color::Rgb(57, 211, 211);
const ORANGE: Color = Color::Rgb(219, 109, 40);
const PINK: Color = Color::Rgb(219, 97, 162);
const CYBER_PUNK: Color = Color::Rgb(30, 240, 201);

#[derive(Debug, Clone)]
struct TrashItem {
    name: String,
    original_path: String,
    deleted_at: String,
    item_type: ItemType,
    size: u64
}

#[derive(Debug, Clone, PartialEq)]
enum ItemType {
    Document,
    Image,
    Video,
    Audio,
    Archive,
    Code,
    Folder,
    Other,
}

impl ItemType {
    fn icon(&self) -> &str {
        match self {
            ItemType::Folder   => "󰉋 ",
            ItemType::Code     => "󰅩 ",
            ItemType::Document => "󰈙 ",
            ItemType::Image    => "󰈟 ",
            ItemType::Video    => "󰈫 ",
            ItemType::Audio    => "󰈣 ",
            ItemType::Archive  => "󰿺 ",
            ItemType::Other    => "󰈚 ",
        }
    }

    fn color(&self) -> Color {
        match self {
            ItemType::Folder => ACCENT,
            ItemType::Code => GREEN,
            ItemType::Document => CYAN,
            ItemType::Image => YELLOW,
            ItemType::Video => PURPLE,
            ItemType::Audio => PINK,
            ItemType::Archive => ORANGE,
            ItemType::Other => TEXT_DIM,
        }
    }

    fn from_extension(ext: &str, is_dir: bool) -> Self {
        if is_dir {
            return ItemType::Folder;
        }
        match ext.to_lowercase().as_str() {
            "pdf" | "doc" | "docx" | "txt" | "md" | "rtf" | "odt" => ItemType::Document,
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg" | "webp" | "ico" => ItemType::Image,
            "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" => ItemType::Video,
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" | "m4a" => ItemType::Audio,
            "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" => ItemType::Archive,
            "rs" | "py" | "js" | "ts" | "java" | "c" | "cpp" | "h" | "go" | "rb" | "php"
            | "swift" | "kt" | "scala" | "html" | "css" | "json" | "yaml" | "toml" | "xml" | "sh" => ItemType::Code,
            _ => ItemType::Other,
        }
    }

    fn label(&self) -> &str {
        match self {
            ItemType::Folder => "Folders",
            ItemType::Code => "Code",
            ItemType::Document => "Docs",
            ItemType::Image => "Images",
            ItemType::Video => "Video",
            ItemType::Audio => "Audio",
            ItemType::Archive => "Archives",
            ItemType::Other => "Other",
        }
    }
}

struct App {
    items: Vec<TrashItem>,
    filtered_items: Vec<TrashItem>,
    list_state: ListState,
    selected_tab: usize,
    tabs: Vec<ItemType>,
    #[allow(dead_code)]
    query: String,
    should_quit: bool,
    scroll_state: ScrollbarState,
}

impl App {
    fn new() -> Self {
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

    fn load_trash_items() -> Vec<TrashItem> {
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

    fn get_type_count(&self, item_type: &ItemType) -> usize {
        self.items
            .iter()
            .filter(|item| &item.item_type == item_type)
            .count()
    }

    fn filter_items(&mut self) {
        let selected_type = &self.tabs[self.selected_tab];
        self.filtered_items = self.items
            .iter()
            .filter(|item| &item.item_type == selected_type)
            .cloned()
            .collect();
        self.scroll_state = self.scroll_state.content_length(self.filtered_items.len());
    }

    fn next_tab(&mut self) {
        self.selected_tab = (self.selected_tab + 1) % self.tabs.len();
        self.filter_items();
        self.list_state.select(if self.filtered_items.is_empty() { None } else { Some(0) });
        self.scroll_state = self.scroll_state.position(0);
    }

    fn prev_tab(&mut self) {
        self.selected_tab = if self.selected_tab == 0 {
            self.tabs.len() - 1
        } else {
            self.selected_tab - 1
        };
        self.filter_items();
        self.list_state.select(if self.filtered_items.is_empty() { None } else { Some(0) });
        self.scroll_state = self.scroll_state.position(0);
    }

    fn next_item(&mut self) {
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

    fn prev_item(&mut self) {
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

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();

    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new();
    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
                        KeyCode::Tab | KeyCode::Right => app.next_tab(),
                        KeyCode::BackTab | KeyCode::Left => app.prev_tab(),
                        KeyCode::Down => app.next_item(),
                        KeyCode::Up => app.prev_item(),
                        _ => {}
                    }
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let area = f.area();

    f.render_widget(Block::default().style(Style::default().bg(BG)), area);

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(area);

    render_header(f, app, main_layout[0]);
    render_content(f, app, main_layout[1]);
    render_query(f, main_layout[2]);
    render_footer(f, main_layout[3]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let total_count = app.items.len();

    let mut spans = vec![
        Span::styled("  ", Style::default().fg(CYBER_PUNK)),
        Span::styled("󰩹 ", Style::default().fg(CYBER_PUNK)),
        Span::styled("Scrap", Style::default().fg(CYBER_PUNK).add_modifier(Modifier::BOLD)),
        Span::styled("  ", Style::default()),
    ];

    for (i, tab) in app.tabs.iter().enumerate() {
        let count = app.get_type_count(tab);
        if count == 0 {
            continue;
        }

        let is_selected = i == app.selected_tab;

        if is_selected {
            spans.push(Span::styled(" ", Style::default().bg(SURFACE_BRIGHT)));
            spans.push(Span::styled(
                tab.icon(),
                Style::default().fg(tab.color()).bg(SURFACE_BRIGHT)
            ));
            spans.push(Span::styled(
                tab.label(),
                Style::default()
                    .fg(TEXT)
                    .bg(SURFACE_BRIGHT)
                    .add_modifier(Modifier::BOLD)
            ));
            spans.push(Span::styled(
                format!(" {}", count),
                Style::default().fg(tab.color()).bg(SURFACE_BRIGHT)
            ));
            spans.push(Span::styled(" ", Style::default().bg(SURFACE_BRIGHT)));
        } else {
            spans.push(Span::styled(" ", Style::default()));
            spans.push(Span::styled(
                tab.icon(),
                Style::default().fg(TEXT_DIM)
            ));
            spans.push(Span::styled(
                tab.label(),
                Style::default().fg(TEXT_DIM)
            ));
            spans.push(Span::styled(
                format!(" {}", count),
                Style::default().fg(TEXT_DIM)
            ));
        }

        spans.push(Span::styled(" ", Style::default()));
    }

    spans.push(Span::styled(
        format!(" {} items", total_count),
        Style::default().fg(TEXT_DIM)
    ));

    let header = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(SURFACE))
        .alignment(Alignment::Left)
        .block(Block::default()
            .borders(Borders::BOTTOM)
            .border_type(BorderType::Plain)
            .border_style(Style::default().fg(BORDER)));

    f.render_widget(header, area);
}

fn render_content(f: &mut Frame, app: &mut App, area: Rect) {
    let content_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .horizontal_margin(1)
        .split(area);

    render_file_list(f, app, content_layout[0]);
    render_details(f, app, content_layout[1]);
}

fn render_file_list(f: &mut Frame, app: &mut App, area: Rect) {
    let max_name_len = area.width.saturating_sub(20) as usize;

    let items: Vec<ListItem> = app.filtered_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_selected = app.list_state.selected() == Some(i);

            let line = Line::from(vec![
                Span::styled(
                    if is_selected { " ▸ " } else { "   " },
                    Style::default().fg(ACCENT)
                ),
                Span::styled(item.item_type.icon(), Style::default().fg(item.item_type.color())),
                Span::styled(
                    truncate_str(&item.name, max_name_len),
                    Style::default()
                        .fg(if is_selected { TEXT } else { TEXT_DIM })
                        .add_modifier(if is_selected { Modifier::BOLD } else { Modifier::empty() })
                ),
            ]);

            ListItem::new(line).style(
                if is_selected {
                    Style::default().bg(SURFACE_BRIGHT)
                } else {
                    Style::default().bg(SURFACE)
                }
            )
        })
        .collect();

    let current_tab = &app.tabs[app.selected_tab];
    let title = format!(" {} ", current_tab.label());

    let list = List::new(items)
        .block(Block::default()
            .title(Span::styled(title, Style::default().fg(current_tab.color()).add_modifier(Modifier::BOLD)))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(BORDER_HIGHLIGHT))
            .style(Style::default().bg(SURFACE)));

    f.render_stateful_widget(list, area, &mut app.list_state);

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(None)
        .end_symbol(None)
        .track_symbol(Some("│"))
        .thumb_symbol("┃")
        .track_style(Style::default().fg(BORDER))
        .thumb_style(Style::default().fg(ACCENT));

    let scrollbar_area = Rect {
        x: area.x + area.width - 1,
        y: area.y + 1,
        width: 1,
        height: area.height.saturating_sub(2),
    };

    f.render_stateful_widget(scrollbar, scrollbar_area, &mut app.scroll_state);
}

fn render_details(f: &mut Frame, app: &App, area: Rect) {
    let max_path_len = area.width.saturating_sub(10) as usize;

    let content = if let Some(selected) = app.list_state.selected() {
        if let Some(item) = app.filtered_items.get(selected) {
            vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("   name ", Style::default().fg(TEXT_DIM)),
                    Span::styled(truncate_str(&item.name, max_path_len), Style::default().fg(TEXT).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("   type ", Style::default().fg(TEXT_DIM)),
                    Span::styled(item.item_type.icon(), Style::default().fg(item.item_type.color())),
                    Span::styled(item.item_type.label(), Style::default().fg(item.item_type.color())),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("   time ", Style::default().fg(TEXT_DIM)),
                    Span::styled(&item.deleted_at, Style::default().fg(YELLOW)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("   Size ", Style::default().fg(TEXT_DIM)),
                    Span::styled(format_bytes(item.size), Style::default().fg(YELLOW)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("   path ", Style::default().fg(TEXT_DIM)),
                    Span::styled(
                        truncate_str(&item.original_path, max_path_len),
                        Style::default().fg(TEXT_DIM)
                    ),
                ]),
            ]
        } else {
            empty_state()
        }
    } else {
        empty_state()
    };

    let details = Paragraph::new(content)
        .block(Block::default()
            .title(Span::styled(" info ", Style::default().fg(TEXT_DIM)))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(BORDER))
            .style(Style::default().bg(SURFACE)));

    f.render_widget(details, area);
}

fn render_query(f: &mut Frame, area: Rect) {
    let inner_area = Rect {
        x: area.x + 1,
        y: area.y,
        width: area.width.saturating_sub(2),
        height: area.height,
    };

    let query_block = Block::default()
        .title(Span::styled("  Query ", Style::default().fg(ACCENT)))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER))
        .style(Style::default().bg(SURFACE));

    f.render_widget(query_block, inner_area);
}

fn render_footer(f: &mut Frame, area: Rect) {
    let footer = Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled("q", Style::default().fg(ACCENT)),
        Span::styled(" quit  ", Style::default().fg(TEXT_DIM)),
        Span::styled("↑↓", Style::default().fg(ACCENT)),
        Span::styled(" navigate  ", Style::default().fg(TEXT_DIM)),
        Span::styled("tab or ←→", Style::default().fg(ACCENT)),
        Span::styled(" switch category", Style::default().fg(TEXT_DIM)),
    ]);

    let footer_widget = Paragraph::new(footer)
        .style(Style::default().bg(BG));

    f.render_widget(footer_widget, area);
}

fn empty_state() -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("      ", Style::default()),
            Span::styled("no item selected", Style::default().fg(TEXT_DIM)),
        ]),
    ]
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if max_len == 0 {
        return String::new();
    }

    let char_count = s.chars().count();
    if char_count <= max_len {
        s.to_string()
    } else if max_len > 3 {
        let truncated: String = s.chars().take(max_len - 3).collect();
        format!("{}...", truncated)
    } else {
        s.chars().take(max_len).collect()
    }
}

fn format_bytes(bytes: u64) -> String {
    let kb = 1024f64;
    let mb = kb * 1024.0;
    let gb = mb * 1024.0;
    let tb = gb * 1024.0;

    let b = bytes as f64;

    if b >= tb {
        format!("{:.2}TB", b / tb)
    } else if b >= gb {
        format!("{:.2}GB", b / gb)
    } else if b >= mb {
        format!("{:.2}MB", b / mb)
    } else if b >= kb {
        format!("{:.2}KB", b / kb)
    } else {
        format!("{}B", bytes)
    }
}

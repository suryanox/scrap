use std::io;
use std::time::Duration;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
    Frame, Terminal,
};

#[derive(Debug, Clone)]
struct TrashItem {
    name: String,
    original_path: String,
    deleted_at: String,
    item_type: ItemType,
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
            ItemType::Document => "󰈙 ",
            ItemType::Image => "󰋩 ",
            ItemType::Video => "󰕧 ",
            ItemType::Audio => "󰎆 ",
            ItemType::Archive => "󰀼 ",
            ItemType::Code => "󰅩 ",
            ItemType::Folder => "󰉋 ",
            ItemType::Other => "󰈔 ",
        }
    }

    fn color(&self) -> Color {
        match self {
            ItemType::Document => Color::Rgb(137, 180, 250),
            ItemType::Image => Color::Rgb(249, 226, 175),
            ItemType::Video => Color::Rgb(203, 166, 247),
            ItemType::Audio => Color::Rgb(166, 227, 161),
            ItemType::Code => Color::Rgb(148, 226, 213),
            ItemType::Archive => Color::Rgb(250, 179, 135),
            ItemType::Folder => Color::Rgb(180, 190, 254),
            ItemType::Other => Color::Rgb(166, 173, 200),
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
            | "swift" | "kt" | "scala" | "html" | "css" | "json" | "yaml" | "toml" | "xml" => ItemType::Code,
            _ => ItemType::Other,
        }
    }

    fn label(&self) -> &str {
        match self {
            ItemType::Document => "Documents",
            ItemType::Image => "Images",
            ItemType::Video => "Videos",
            ItemType::Audio => "Audio",
            ItemType::Archive => "Archives",
            ItemType::Code => "Code",
            ItemType::Folder => "Folders",
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
                });
            }
        }
        
        items.sort_by(|a, b| b.deleted_at.cmp(&a.deleted_at));
        items
    }

    fn filter_items(&mut self) {
        let selected_type = &self.tabs[self.selected_tab];
        self.filtered_items = self.items
            .iter()
            .filter(|item| &item.item_type == selected_type)
            .cloned()
            .collect();
    }

    fn next_tab(&mut self) {
        self.selected_tab = (self.selected_tab + 1) % self.tabs.len();
        self.filter_items();
        self.list_state.select(if self.filtered_items.is_empty() { None } else { Some(0) });
    }

    fn prev_tab(&mut self) {
        self.selected_tab = if self.selected_tab == 0 {
            self.tabs.len() - 1
        } else {
            self.selected_tab - 1
        };
        self.filter_items();
        self.list_state.select(if self.filtered_items.is_empty() { None } else { Some(0) });
    }

    fn next_item(&mut self) {
        if self.filtered_items.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.filtered_items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn prev_item(&mut self) {
        if self.filtered_items.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
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

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app)).unwrap();

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => app.should_quit = true,
                        KeyCode::Tab => app.next_tab(),
                        KeyCode::BackTab => app.prev_tab(),
                        KeyCode::Down | KeyCode::Char('j') => app.next_item(),
                        KeyCode::Up | KeyCode::Char('k') => app.prev_item(),
                        KeyCode::Left | KeyCode::Char('h') => app.prev_tab(),
                        KeyCode::Right | KeyCode::Char('l') => app.next_tab(),
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
    let bg_color = Color::Rgb(30, 30, 46);
    let surface_color = Color::Rgb(49, 50, 68);
    let text_color = Color::Rgb(205, 214, 244);
    let accent_color = Color::Rgb(137, 180, 250);
    let border_color = Color::Rgb(88, 91, 112);
    let dim_text = Color::Rgb(108, 112, 134);

    let main_block = Block::default()
        .style(Style::default().bg(bg_color));
    f.render_widget(main_block, f.area());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(5),
        ])
        .split(f.area());

    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(12), Constraint::Min(0)])
        .split(chunks[0]);

    let title = Paragraph::new(Text::from(vec![
        Line::from(vec![
            Span::styled("󰩹 ", Style::default().fg(Color::Rgb(243, 139, 168))),
            Span::styled("scrap", Style::default().fg(accent_color).bold()),
        ]),
    ]))
    .style(Style::default().bg(bg_color))
    .block(Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(surface_color)));
    f.render_widget(title, header_chunks[0]);

    let tab_titles: Vec<Line> = app.tabs
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let count = app.items.iter().filter(|item| &item.item_type == t).count();
            let style = if i == app.selected_tab {
                Style::default().fg(t.color()).bold()
            } else {
                Style::default().fg(dim_text)
            };
            Line::from(vec![
                Span::styled(t.icon(), style),
                Span::styled(format!("{} ", t.label()), style),
                Span::styled(format!("({})", count), Style::default().fg(dim_text)),
            ])
        })
        .collect();

    let tabs = Tabs::new(tab_titles)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(surface_color)))
        .highlight_style(Style::default().fg(accent_color))
        .select(app.selected_tab)
        .divider(Span::styled(" │ ", Style::default().fg(border_color)));
    f.render_widget(tabs, header_chunks[1]);

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[1]);

    let items: Vec<ListItem> = app.filtered_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_selected = app.list_state.selected() == Some(i);
            let style = if is_selected {
                Style::default()
                    .fg(item.item_type.color())
                    .bg(Color::Rgb(69, 71, 90))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(text_color)
            };

            let content = Line::from(vec![
                Span::styled(item.item_type.icon(), Style::default().fg(item.item_type.color())),
                Span::styled(&item.name, style),
            ]);
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .title(Span::styled(
                format!(" {} ({}) ", app.tabs[app.selected_tab].label(), app.filtered_items.len()),
                Style::default().fg(accent_color).bold()
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(surface_color)))
        .highlight_style(Style::default().bg(Color::Rgb(69, 71, 90)))
        .highlight_symbol("▸ ");
    f.render_stateful_widget(list, content_chunks[0], &mut app.list_state);

    let details = if let Some(selected) = app.list_state.selected() {
        if let Some(item) = app.filtered_items.get(selected) {
            vec![
                Line::from(vec![
                    Span::styled("Name: ", Style::default().fg(dim_text)),
                    Span::styled(&item.name, Style::default().fg(text_color)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Type: ", Style::default().fg(dim_text)),
                    Span::styled(item.item_type.label(), Style::default().fg(item.item_type.color())),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Deleted: ", Style::default().fg(dim_text)),
                    Span::styled(&item.deleted_at, Style::default().fg(Color::Rgb(245, 194, 231))),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Path: ", Style::default().fg(dim_text)),
                ]),
                Line::from(vec![
                    Span::styled(&item.original_path, Style::default().fg(dim_text).italic()),
                ]),
            ]
        } else {
            vec![Line::from(Span::styled("No item selected", Style::default().fg(dim_text)))]
        }
    } else {
        vec![Line::from(Span::styled("No item selected", Style::default().fg(dim_text)))]
    };

    let details_widget = Paragraph::new(details)
        .block(Block::default()
            .title(Span::styled(" Details ", Style::default().fg(accent_color).bold()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(surface_color)))
        .style(Style::default().bg(surface_color));
    f.render_widget(details_widget, content_chunks[1]);

    let query_block = Block::default()
        .title(Span::styled(" Query ", Style::default().fg(accent_color).bold()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(surface_color));

    let query_inner = query_block.inner(chunks[2]);
    f.render_widget(query_block, chunks[2]);

    let help_text = Line::from(vec![
        Span::styled("  q", Style::default().fg(Color::Rgb(243, 139, 168)).bold()),
        Span::styled(" quit  ", Style::default().fg(dim_text)),
        Span::styled("↑↓/jk", Style::default().fg(Color::Rgb(166, 227, 161)).bold()),
        Span::styled(" navigate  ", Style::default().fg(dim_text)),
        Span::styled("←→/hl/Tab", Style::default().fg(Color::Rgb(249, 226, 175)).bold()),
        Span::styled(" switch tab", Style::default().fg(dim_text)),
    ]);

    let help_widget = Paragraph::new(help_text)
        .style(Style::default().bg(surface_color));
    f.render_widget(help_widget, query_inner);
}

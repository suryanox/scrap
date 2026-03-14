use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Line, Modifier, Span, Style};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation};
use crate::app::App;
use crate::color::{ACCENT, BG, BORDER, BORDER_HIGHLIGHT, CYBER_PUNK, INFO_BLUE, LAVENDER, MINT, SURFACE, SURFACE_BRIGHT, TEXT, TEXT_DIM};

pub fn ui(f: &mut Frame, app: &mut App) {
    let area = f.area();

    f.render_widget(Block::default().style(Style::default().bg(BG)), area);

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(7),
            Constraint::Length(4),
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
    let max_path_len = area.width.saturating_sub(5) as usize;

    let content = if let Some(selected) = app.list_state.selected() {
        if let Some(item) = app.filtered_items.get(selected) {
            vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("   Name: ", Style::default().fg(MINT)),
                    Span::styled(truncate_str(&item.name, max_path_len), Style::default().fg(LAVENDER).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::styled("   Type: ", Style::default().fg(MINT)),
                    Span::styled(item.item_type.icon(), Style::default().fg(item.item_type.color())),
                    Span::styled(item.item_type.label(), Style::default().fg(item.item_type.color())),
                ]),
                Line::from(vec![
                    Span::styled("   Date Modified: ", Style::default().fg(MINT)),
                    Span::styled(&item.deleted_at, Style::default().fg(LAVENDER)),
                ]),
                Line::from(vec![
                    Span::styled("   Size: ", Style::default().fg(MINT)),
                    Span::styled(format_bytes(item.size), Style::default().fg(LAVENDER)),
                ]),
                Line::from(vec![
                    Span::styled("   Path: ", Style::default().fg(MINT)),
                    Span::styled(
                        truncate_str(&item.original_path, max_path_len),
                        Style::default().fg(LAVENDER)
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
            .title(Span::styled("󰋽 Info ", Style::default().fg(INFO_BLUE)))
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
            Span::styled("No item selected", Style::default().fg(TEXT_DIM)),
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
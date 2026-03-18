use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Line, Modifier, Span, Style};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, Wrap};
use crate::app::{App, InputMode, QueryCommand};
use crate::color::{ACCENT, BG, BORDER, BORDER_HIGHLIGHT, CYBER_PUNK, GREEN, INFO_BLUE, LAVENDER, MINT, ORANGE, SURFACE, SURFACE_BRIGHT, TEXT, TEXT_DIM};

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
    render_query(f, app, main_layout[2]);
    render_footer(f, app, main_layout[3]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let mut spans = vec![
        Span::styled("  ", Style::default().fg(CYBER_PUNK)),
        Span::styled("󰩹 ", Style::default().fg(CYBER_PUNK)),
        Span::styled("Scrap", Style::default().fg(CYBER_PUNK).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" {} ", format_bytes(app.scrap_yard.size())),
            Style::default().fg(CYBER_PUNK).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ", Style::default()),
    ];

    for (i, tab) in app.tabs.iter().enumerate() {
        let count = app.scrap_yard.get_type_count(tab);
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
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .horizontal_margin(1)
        .split(area);

    render_file_list(f, app, content_layout[0]);
    render_details(f, app, content_layout[1]);
}

fn render_file_list(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app.scrap_yard
        .iter_items(app.current_item_type())
        .enumerate()
        .map(|(i, item)| {
            let is_selected = app.list_state.selected() == Some(i);

            let line = Line::from(vec![
                Span::styled(
                    if is_selected { " ▸ " } else { "   " },
                    Style::default().fg(ACCENT)
                ),
                Span::styled(item.item_type.icon(), Style::default().fg(item.item_type.color())),
                Span::styled(&item.name,
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
    let content = if let Some(item) = app.get_selected_item() {
        vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("   Name: ", Style::default().fg(MINT)),
                Span::styled(&item.name, Style::default().fg(LAVENDER).add_modifier(Modifier::BOLD)),
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
                    &item.original_path,
                    Style::default().fg(LAVENDER)
                ),
            ]),
        ]
    } else {
        empty_state()
    };

    let details = Paragraph::new(content).wrap(Wrap { trim: false })
        .block(Block::default()
            .title(Span::styled("󰋽 Info ", Style::default().fg(INFO_BLUE)))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(BORDER))
            .style(Style::default().bg(SURFACE)));

    f.render_widget(details, area);
}

fn render_query(f: &mut Frame, app: &App, area: Rect) {
    let inner_area = Rect {
        x: area.x + 1,
        y: area.y,
        width: area.width.saturating_sub(2),
        height: area.height,
    };

    let is_query_mode = app.input_mode == InputMode::Query;
    let border_color = if is_query_mode { BORDER_HIGHLIGHT } else { BORDER };

    let query_block = Block::default()
        .title(Span::styled("  Query ", Style::default().fg(ACCENT)))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(SURFACE));

    f.render_widget(query_block, inner_area);

    let content_area = Rect {
        x: inner_area.x + 2,
        y: inner_area.y + 1,
        width: inner_area.width.saturating_sub(4),
        height: inner_area.height.saturating_sub(2),
    };

    let query_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(20), Constraint::Length(12)])
        .split(content_area);

    let (status_icon, status_color) = match &app.query_command {
        QueryCommand::Empty => ("", TEXT_DIM),
        QueryCommand::DeleteAll => ("󰄬 ", GREEN),
        QueryCommand::DeleteByType(_) => ("󰄬 ", GREEN),
        QueryCommand::Invalid => ("󰅙 ", ORANGE),
    };

    let cursor = if is_query_mode { "▎" } else { "" };
    
    let input_spans = vec![
        Span::styled(status_icon, Style::default().fg(status_color)),
        Span::styled(&app.query, Style::default().fg(TEXT)),
        Span::styled(cursor, Style::default().fg(ACCENT)),
    ];

    let hint = if app.query.is_empty() && is_query_mode {
        " delete all | delete <type>"
    } else {
        ""
    };

    let input_line = if app.query.is_empty() && is_query_mode {
        Line::from(vec![
            Span::styled(hint, Style::default().fg(TEXT_DIM)),
            Span::styled(cursor, Style::default().fg(ACCENT)),
        ])
    } else {
        Line::from(input_spans)
    };

    let input = Paragraph::new(input_line);
    f.render_widget(input, query_layout[0]);

    let run_button = if app.is_query_valid() {
        Line::from(vec![
            Span::styled(" ", Style::default().bg(GREEN)),
            Span::styled(" Run ", Style::default().fg(BG).bg(GREEN).add_modifier(Modifier::BOLD)),
            Span::styled("↵", Style::default().fg(BG).bg(GREEN)),
            Span::styled(" ", Style::default().bg(GREEN)),
        ])
    } else {
        Line::from(vec![
            Span::styled(" ", Style::default().bg(SURFACE_BRIGHT)),
            Span::styled(" Run ", Style::default().fg(TEXT_DIM).bg(SURFACE_BRIGHT)),
            Span::styled("↵", Style::default().fg(TEXT_DIM).bg(SURFACE_BRIGHT)),
            Span::styled(" ", Style::default().bg(SURFACE_BRIGHT)),
        ])
    };

    let button = Paragraph::new(run_button).alignment(Alignment::Right);
    f.render_widget(button, query_layout[1]);
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let footer = match app.input_mode {
        InputMode::Query => Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled("esc", Style::default().fg(ACCENT)),
            Span::styled(" cancel  ", Style::default().fg(TEXT_DIM)),
            Span::styled("enter", Style::default().fg(ACCENT)),
            Span::styled(" run query", Style::default().fg(TEXT_DIM)),
        ]),
        InputMode::Normal => Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled("q", Style::default().fg(ACCENT)),
            Span::styled(" quit  ", Style::default().fg(TEXT_DIM)),
            Span::styled("↑↓", Style::default().fg(ACCENT)),
            Span::styled(" navigate  ", Style::default().fg(TEXT_DIM)),
            Span::styled("tab or ←→", Style::default().fg(ACCENT)),
            Span::styled(" switch category  ", Style::default().fg(TEXT_DIM)),
            Span::styled("/", Style::default().fg(ACCENT)),
            Span::styled(" query", Style::default().fg(TEXT_DIM)),
        ]),
    };

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
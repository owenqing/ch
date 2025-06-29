use tui::backend::Backend;
use tui::widgets::{List, ListItem, Block, Borders, Paragraph, ListState};
use tui::style::{Style, Color, Modifier};
use tui::layout::{Constraint, Rect, Layout, Direction};
use tui::text::{Span, Spans};
use std::fmt::Write as FmtWrite;
use crate::config::Config;
use crate::app::AppState;
use anyhow::Result;

pub fn render_ui<B: Backend>(
    terminal: &mut tui::Terminal<B>,
    config: &Config,
    app: &AppState,
) -> Result<()> {
    let group_names: Vec<_> = config.groups.keys().cloned().collect();
    let filtered_connections = get_filtered_connections(config, &group_names, app);

    let groups: Vec<ListItem> = group_names.iter().map(|k| ListItem::new(k.as_str())).collect();
    let connection_items: Vec<ListItem> = filtered_connections.iter().map(|(group_name, k)| {
        if let Some(group) = config.groups.get(group_name) {
            if let Some(desc) = group.connections.get(k.as_str()) {
                if app.search_mode && !app.search_query.is_empty() {
                    highlight_search(k, desc, &app.search_query)
                } else {
                    ListItem::new(Spans::from(vec![
                        Span::styled(k.to_string(), Style::default().fg(Color::Yellow)),
                        Span::raw(" - " ),
                        Span::styled(desc.to_string(), Style::default().fg(Color::Green)),
                    ]))
                }
            } else {
                ListItem::new(k.as_str())
            }
        } else {
            ListItem::new(k.as_str())
        }
    }).collect();

    let mut group_state = ListState::default();
    let filtered_has_match = !filtered_connections.is_empty();
    let group_highlight = if app.search_mode && !app.search_query.is_empty() && filtered_has_match {
        filtered_connections.get(app.current_selection).map(|(g, _)| {
            config.groups.keys().position(|k| k == g)
        }).flatten()
    } else if !app.search_mode {
        Some(app.selected_group)
    } else {
        None
    };
    if let Some(idx) = group_highlight {
        group_state.select(Some(idx));
    }
    let mut connection_state = ListState::default();
    if app.focus == 1 && (!app.search_mode || !app.search_query.is_empty()) {
        connection_state.select(Some(app.current_selection));
    }

    terminal.draw(|f| {
        let layout_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(3),
                Constraint::Percentage(95),
                Constraint::Percentage(2),
            ].as_ref())
            .split(f.size());
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
            .split(layout_chunks[1]);
        let group_list = List::new(groups.clone())
            .block(Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(" Groups ", Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD)))
                .border_style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::SLOW_BLINK));
        let connection_list = List::new(connection_items.clone())
            .block(Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    if app.search_mode { " Search Result " } else { " Commands " },
                    Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD)
                ))
                .border_style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::SLOW_BLINK));
        let status_text = {
            let mut text = String::new();
            FmtWrite::write_fmt(&mut text, format_args!("Menu: ← → | Option: ↑ ↓ | Exec: Enter | Exit: q | Search: / ")).unwrap();
            text
        };
        let top_status_bar = Paragraph::new(Spans::from(vec![
            Span::styled("  Operation: ", Style::default().fg(Color::Green)),
            Span::raw(&status_text),
        ]))
            .style(Style::default().bg(Color::Black).fg(Color::White).add_modifier(Modifier::BOLD))
            .alignment(tui::layout::Alignment::Left);
        let (bottom_left_text, bottom_right_text) = if app.search_mode {
            (
                format!("Press ESC exit search mode | Search: {}", app.search_query),
                format!("Menu: {}", if app.focus == 0 { "Groups" } else { "Commands" })
            )
        } else {
            (
                format!("Press / into search mode"),
                format!("Menu: {}", if app.focus == 0 { "Groups" } else { "Commands" })
            )
        };
        let bottom_left_info = Paragraph::new(bottom_left_text)
            .style(Style::default().bg(Color::Black).fg(Color::White).add_modifier(Modifier::BOLD))
            .alignment(tui::layout::Alignment::Left)
            .wrap(tui::widgets::Wrap { trim: false });
        let bottom_right_info = Paragraph::new(bottom_right_text)
            .style(Style::default().bg(Color::Black).fg(Color::LightYellow).add_modifier(Modifier::BOLD))
            .alignment(tui::layout::Alignment::Right)
            .wrap(tui::widgets::Wrap { trim: true });
        f.render_widget(Block::default().style(Style::default().bg(Color::Reset)), f.size());
        let group_area = Rect::new(
            main_chunks[0].x + 1,
            main_chunks[0].y + 1,
            main_chunks[0].width - 1,
            main_chunks[0].height - 1
        );
        let connection_area = Rect::new(
            main_chunks[1].x + 1,
            main_chunks[1].y + 1,
            main_chunks[1].width - 1,
            main_chunks[1].height - 1
        );
        f.render_stateful_widget(group_list, group_area, &mut group_state);
        f.render_stateful_widget(connection_list, connection_area, &mut connection_state);
        f.render_widget(top_status_bar, layout_chunks[0]);
        let bottom_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
            .split(Rect::new(
                layout_chunks[2].x + 2,
                layout_chunks[2].y,
                layout_chunks[2].width - 3,
                layout_chunks[2].height
            ));
        f.render_widget(bottom_left_info, bottom_chunks[0]);
        f.render_widget(bottom_right_info, bottom_chunks[1]);
    })?;
    Ok(())
}

pub fn get_filtered_connections(
    config: &Config,
    group_names: &[String],
    app: &AppState,
) -> Vec<(String, String)> {
    let all_commands: Vec<(String, String)> = config.groups.iter()
        .flat_map(|(group_name, group)| {
            group.connections.iter()
                .map(move |(cmd, _)| (group_name.clone(), cmd.clone()))
        })
        .collect();
    if app.search_mode {
        all_commands
            .iter()
            .filter(|(_, cmd)| cmd.to_lowercase().contains(&app.search_query.to_lowercase()))
            .cloned()
            .collect()
    } else {
        if let Some(group_name) = group_names.get(app.selected_group).cloned() {
            if let Some(_group) = config.groups.get(&group_name) {
                _group.connections.keys().map(|s| (group_name.clone(), s.to_string())).collect()
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }
}

fn highlight_search<'a>(cmd: &'a str, desc: &'a str, query: &'a str) -> ListItem<'a> {
    let lower_k = cmd.to_lowercase();
    let lower_query = query.to_lowercase();
    let mut spans = Vec::new();
    let mut last = 0;
    let mut search_start = 0;
    while let Some(pos) = lower_k[search_start..].find(&lower_query) {
        let match_start = search_start + pos;
        let match_end = match_start + lower_query.len();
        let orig_match_start = cmd.char_indices().nth(match_start).map(|(i, _)| i).unwrap_or(match_start);
        let orig_match_end = cmd.char_indices().nth(match_end).map(|(i, _)| i).unwrap_or(match_end);
        if last < orig_match_start {
            spans.push(Span::raw(&cmd[last..orig_match_start]));
        }
        spans.push(Span::styled(
            &cmd[orig_match_start..orig_match_end],
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        ));
        last = orig_match_end;
        search_start = match_end;
        if match_end >= lower_k.len() {
            break;
        }
    }
    if last < cmd.len() {
        spans.push(Span::raw(&cmd[last..]));
    }
    spans.push(Span::raw(" - "));
    spans.push(Span::styled(desc.to_string(), Style::default().fg(Color::Green)));
    ListItem::new(Spans::from(spans))
} 
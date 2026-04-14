//! UI widgets for the skill management TUI.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Borders, Paragraph, Row as TuiRow, Table},
    Frame,
};

use super::state::{PendingAction, SkillRow};

/// Truncates text to fit within max_width characters.
fn truncate(text: &str, max_width: usize) -> String {
    let t = text.trim();
    if t.chars().count() <= max_width {
        return t.to_string();
    }
    let mut out = String::new();
    for ch in t.chars().take(max_width.saturating_sub(1)) {
        out.push(ch);
    }
    out.push('…');
    out
}

/// Renders the skill description in a dedicated container.
fn render_description(area: Rect, selected_row: Option<&SkillRow>, frame: &mut Frame) {
    let Some(row) = selected_row else {
        let block = Block::default()
            .borders(Borders::BOTTOM)
            .style(Style::default().fg(Color::DarkGray));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        frame.render_widget(Paragraph::new(""), inner);
        return;
    };

    let Some(skill) = &row.catalog_skill else {
        let block = Block::default()
            .title(" skill ")
            .borders(Borders::BOTTOM)
            .style(Style::default().fg(Color::DarkGray));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        frame.render_widget(
            Paragraph::new("Skill fora do catálogo — descrição não disponível")
                .style(Style::default().fg(Color::DarkGray)),
            inner,
        );
        return;
    };

    let desc = &skill.description;
    let name = truncate(&skill.name, 26);
    let cat = truncate(skill.catalog_label.as_str(), 20);
    let header = format!(" skill: {name} · {cat} ");

    let block = Block::default()
        .title(header)
        .borders(Borders::BOTTOM)
        .style(Style::default().fg(Color::White));

    let inner = block.inner(area);
    let width = inner.width as usize;
    let wrapped = wrap_text(desc, width);
    let lines: Vec<Line> = wrapped
        .into_iter()
        .take(area.height as usize)
        .map(Line::from)
        .collect();

    let paragraph = Paragraph::new(lines).style(Style::default().fg(Color::White));
    frame.render_widget(block, area);
    frame.render_widget(paragraph, inner);
}

/// Wraps text to fit within specified width.
fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

/// Renders the help bar at the bottom of the screen.
fn render_help_bar(area: Rect, selected_row: Option<&SkillRow>, frame: &mut Frame) {
    let (can_install, can_update, can_remove) = match selected_row {
        Some(row) => (
            row.catalog_skill.is_some(),
            row.has_any_update(),
            row.local.is_some() || row.global.is_some(),
        ),
        None => (false, false, false),
    };

    let mut parts: Vec<String> = Vec::new();
    parts.push("↑↓ navegar".to_string());

    if can_install {
        parts.push("p instalar (P)".to_string());
        parts.push("g instalar (G)".to_string());
    }
    if can_update {
        parts.push("u atualizar".to_string());
    }
    if can_remove {
        parts.push("r remover".to_string());
    }

    parts.push("c limpar".to_string());
    parts.push("Enter aplicar".to_string());
    parts.push("q sair".to_string());

    // Build spans with separators between items
    let spans: Vec<ratatui::text::Span> = parts
        .iter()
        .enumerate()
        .flat_map(|(i, part)| {
            if i > 0 {
                vec![
                    ratatui::text::Span::raw(" | "),
                    ratatui::text::Span::raw(part),
                ]
            } else {
                vec![ratatui::text::Span::raw(part)]
            }
        })
        .collect();

    let line = Line::from(spans).style(Style::default().fg(Color::DarkGray));
    let paragraph = Paragraph::new(line).alignment(Alignment::Left);
    frame.render_widget(paragraph, area);
}

/// Renders the skill list as a table within the given area.
fn render_skill_list(
    rows: &[SkillRow],
    cursor: usize,
    scroll: usize,
    area: Rect,
    frame: &mut Frame,
) {
    if rows.is_empty() {
        let paragraph = Paragraph::new("(catálogo vazio)")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
        return;
    }

    // Cabeçalho da tabela + margem consomem ~2 linhas dentro da área.
    let max_body_rows = (area.height.saturating_sub(2)).max(1) as usize;

    let visible_rows: Vec<TuiRow> = rows
        .iter()
        .skip(scroll)
        .take(max_body_rows)
        .enumerate()
        .map(|(i, row)| {
            let idx = scroll + i;
            let selected = idx == cursor;

            let name = if row.is_orphan() {
                format!("{} (órfã)", truncate(&row.name, 18))
            } else {
                truncate(&row.name, 18)
            };

            let cat_label = truncate(row.catalog_list_label(), 14);
            let pending = truncate(&row.pending_display_label(), 18);

            let style = if selected {
                Style::default().fg(Color::Yellow).bg(Color::DarkGray)
            } else if row.is_orphan() {
                Style::default().fg(Color::DarkGray)
            } else if row.pending != PendingAction::None {
                Style::default().fg(Color::LightGreen)
            } else {
                Style::default()
            };

            TuiRow::new(vec![
                if selected { ">" } else { " " }.to_string(),
                name,
                row.status_compact(),
                pending,
                cat_label,
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        visible_rows,
        [
            Constraint::Length(2),  // Selection indicator
            Constraint::Length(20), // Name
            Constraint::Length(14), // Status
            Constraint::Length(18), // Pending
            Constraint::Length(16), // Catalog (last)
        ],
    )
    .header(
        TuiRow::new(vec![" ", "Nome", "Estado", "Pendente", "Catálogo"])
            .style(Style::default().fg(Color::Cyan).bold())
            .bottom_margin(1),
    )
    .block(
        Block::default()
            .title("ai-workflow-skills — gestão de skills")
            .borders(Borders::ALL),
    )
    .style(Style::default());

    frame.render_widget(table, area);
}

/// Renders a flash message (temporary notification) within the given area.
fn render_flash(flash: &str, area: Rect, frame: &mut Frame) {
    if flash.is_empty() {
        return;
    }

    let paragraph = Paragraph::new(flash)
        .style(Style::default().fg(Color::LightYellow))
        .alignment(Alignment::Center);

    let block = Block::default().borders(Borders::BOTTOM);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(paragraph, inner);
}

/// Partitions the terminal into non-overlapping regions and renders all UI elements.
pub fn render(frame: &mut Frame, rows: &[SkillRow], cursor: usize, scroll: usize, flash: &str) {
    let area = frame.area();

    // Partition: [ flash (1) | skill list | description (3) | help bar (1) ]
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // flash message
            Constraint::Min(1),    // skill list
            Constraint::Length(3), // description
            Constraint::Length(1), // help bar
        ])
        .split(area);

    let [flash_area, list_area, desc_area, help_area] = chunks.as_ref() else {
        return;
    };

    let selected_row = rows.get(cursor).map(|r| r as &SkillRow);
    render_flash(flash, *flash_area, frame);
    render_skill_list(rows, cursor, scroll, *list_area, frame);
    render_description(*desc_area, selected_row, frame);
    render_help_bar(*help_area, selected_row, frame);
}

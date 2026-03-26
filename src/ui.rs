use crate::app::{scroll_offset, App, LoginField, Screen, SearchMode, Tab};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table, Tabs, Wrap,
};
use ratatui::Frame;

// Color palette
const BLUE: Color = Color::Rgb(100, 149, 237);
const CYAN: Color = Color::Rgb(0, 210, 211);
const GOLD: Color = Color::Rgb(255, 200, 60);
const GREEN: Color = Color::Rgb(80, 220, 120);
const RED: Color = Color::Rgb(255, 100, 100);
const DIM: Color = Color::Rgb(100, 100, 110);
const SURFACE: Color = Color::Rgb(30, 30, 40);
const ROW_ALT: Color = Color::Rgb(22, 22, 32);
const WHITE: Color = Color::Rgb(220, 220, 230);
const MUTED: Color = Color::Rgb(150, 150, 165);

pub fn render(f: &mut Frame, app: &mut App) {
    let bg = Block::default().style(Style::default().bg(SURFACE));
    f.render_widget(bg, f.area());

    match app.screen {
        Screen::Login => render_login(f, app),
        Screen::Main => render_main(f, app),
        Screen::AlbumView => render_album(f, app),
    }
}

// ── Login ────────────────────────────────────────────────────────────────────

fn render_login(f: &mut Frame, app: &App) {
    let area = f.area();
    let form_width = 56u16.min(area.width.saturating_sub(4));
    let form_height = 18u16.min(area.height.saturating_sub(2));
    let form_area = centered_rect(form_width, form_height, area);

    let form_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BLUE))
        .style(Style::default().bg(SURFACE));
    f.render_widget(Clear, form_area);
    f.render_widget(form_block, form_area);

    let inner = inner_rect(form_area, 3, 1);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Q", Style::default().fg(BLUE).add_modifier(Modifier::BOLD)),
            Span::styled("obuz ", Style::default().fg(WHITE).add_modifier(Modifier::BOLD)),
            Span::styled("TUI", Style::default().fg(CYAN)),
        ]))
        .alignment(Alignment::Center),
        chunks[0],
    );

    f.render_widget(
        Paragraph::new("Sign in to your account")
            .style(Style::default().fg(DIM))
            .alignment(Alignment::Center),
        chunks[1],
    );

    let labels = ["Email", "Password"];
    let fields = [LoginField::Email, LoginField::Password];
    for (i, (label, field)) in labels.iter().zip(fields.iter()).enumerate() {
        let label_idx = 3 + i * 3;
        let input_idx = 4 + i * 3;
        let is_focused = app.login_focus == *field;
        let value = &app.login_fields[i];
        let display_value = if i == 1 { "\u{2022}".repeat(value.len()) } else { value.clone() };

        let label_style = if is_focused {
            Style::default().fg(BLUE).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(MUTED)
        };
        f.render_widget(Paragraph::new(format!("  {}", label)).style(label_style), chunks[label_idx]);

        let (border_color, cursor) = if is_focused { (CYAN, "\u{2588}") } else { (DIM, "") };
        f.render_widget(
            Block::default().style(Style::default().bg(ROW_ALT)).borders(Borders::BOTTOM).border_style(Style::default().fg(border_color)),
            chunks[input_idx],
        );
        f.render_widget(
            Paragraph::new(format!(" {}{}", display_value, cursor)).style(Style::default().fg(WHITE).bg(ROW_ALT)),
            chunks[input_idx],
        );
    }

    let submit_style = if app.login_focus == LoginField::Submit {
        Style::default().fg(SURFACE).bg(BLUE).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(BLUE).add_modifier(Modifier::BOLD)
    };
    let submit_text = if app.login_loading { "  Connecting...  " } else { "  Sign In  " };
    f.render_widget(Paragraph::new(submit_text).style(submit_style).alignment(Alignment::Center), chunks[9]);

    if let Some(status) = &app.login_status {
        f.render_widget(Paragraph::new(status.as_str()).style(Style::default().fg(GOLD)).alignment(Alignment::Center), chunks[11]);
    }
    if let Some(err) = &app.login_error {
        f.render_widget(Paragraph::new(err.as_str()).style(Style::default().fg(RED)).alignment(Alignment::Center).wrap(Wrap { trim: true }), chunks[12]);
    }
}

// ── Main screen ──────────────────────────────────────────────────────────────

fn render_main(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(4),
            Constraint::Length(1),
        ])
        .split(area);

    render_header(f, app, main_chunks[0]);

    match app.tab {
        Tab::Search => render_search_content(f, app, main_chunks[1]),
        Tab::Favorites => render_favorites(f, app, main_chunks[1]),
    }

    render_player_bar(f, app, main_chunks[2]);
    render_help_bar(f, app, main_chunks[3]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(10)])
        .split(area);

    let tab_titles = vec![
        Line::from(vec![Span::styled("F1", Style::default().fg(DIM)), Span::raw(" Search")]),
        Line::from(vec![Span::styled("F2", Style::default().fg(DIM)), Span::raw(" Albums")]),
    ];
    let selected = match app.tab { Tab::Search => 0, Tab::Favorites => 1 };
    f.render_widget(
        Tabs::new(tab_titles)
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(DIM)).style(Style::default().bg(SURFACE)))
            .select(selected)
            .style(Style::default().fg(MUTED))
            .highlight_style(Style::default().fg(BLUE).add_modifier(Modifier::BOLD))
            .divider(Span::styled(" | ", Style::default().fg(DIM))),
        header_chunks[0],
    );

    if app.tab == Tab::Search {
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(" \u{1F50D} ", Style::default().fg(DIM)),
                Span::styled(&app.search_query, Style::default().fg(WHITE)),
                Span::styled("\u{2588}", Style::default().fg(CYAN)),
            ]))
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(BLUE)).style(Style::default().bg(SURFACE))),
            header_chunks[1],
        );
    } else {
        let count = app.favorite_albums.len();
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(" \u{2665} ", Style::default().fg(RED)),
                Span::styled(format!("{} favorite album{}", count, if count != 1 { "s" } else { "" }), Style::default().fg(MUTED)),
            ]))
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(DIM)).style(Style::default().bg(SURFACE))),
            header_chunks[1],
        );
    }
}

fn render_search_content(f: &mut Frame, app: &mut App, area: Rect) {
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(3)])
        .split(area);

    let mut spans = vec![
        Span::styled(" ", Style::default()),
        mode_span(" Tracks ", app.search_mode == SearchMode::Tracks),
        Span::styled(" ", Style::default()),
        mode_span(" Albums ", app.search_mode == SearchMode::Albums),
    ];
    if let Some(status) = &app.status_message {
        let style = if status.contains("rror") { Style::default().fg(RED) } else { Style::default().fg(GOLD) };
        spans.push(Span::styled("   ", Style::default()));
        spans.push(Span::styled(status, style));
    }
    f.render_widget(Paragraph::new(Line::from(spans)), content_chunks[0]);

    match app.search_mode {
        SearchMode::Tracks => {
            let visible = list_visible_height(content_chunks[1]);
            app.search_scroll = scroll_offset(app.search_selected, app.search_scroll, visible);
            render_track_list(f, &app.search_tracks, app.search_selected, app.search_scroll, content_chunks[1]);
        }
        SearchMode::Albums => {
            let visible = list_visible_height(content_chunks[1]);
            app.search_scroll = scroll_offset(app.search_selected, app.search_scroll, visible);
            render_album_list(f, &app.search_albums, app.search_selected, app.search_scroll, content_chunks[1]);
        }
    }
}

// ── Track list (search) ──────────────────────────────────────────────────────

fn render_track_list(f: &mut Frame, tracks: &[crate::api::Track], selected: usize, offset: usize, area: Rect) {
    if tracks.is_empty() {
        f.render_widget(
            Paragraph::new("Type a query and press Enter to search").style(Style::default().fg(DIM)).alignment(Alignment::Center).block(table_block(" Tracks ".into())),
            area,
        );
        return;
    }

    let visible = list_visible_height(area);
    let end = (offset + visible).min(tracks.len());
    let header = Row::new(vec![
        Cell::from("  # "), Cell::from("Title"), Cell::from("Artist"), Cell::from("Album"), Cell::from("  Time"),
    ]).style(Style::default().fg(BLUE).add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = tracks[offset..end]
        .iter()
        .enumerate()
        .map(|(vi, track)| {
            let i = offset + vi;
            let sel = i == selected;
            let bg = row_bg(sel, i);
            let fg = if sel { GOLD } else { WHITE };
            let marker = if sel { " \u{25B6} " } else { "   " };
            Row::new(vec![
                Cell::from(format!("{}{}", marker, i + 1)).style(Style::default().fg(if sel { GOLD } else { DIM }).bg(bg)),
                Cell::from(track.title.as_str()),
                Cell::from(track.artist_name()).style(Style::default().fg(if sel { GOLD } else { MUTED }).bg(bg)),
                Cell::from(track.album_title()).style(Style::default().fg(if sel { GOLD } else { DIM }).bg(bg)),
                Cell::from(format!("  {}", track.format_duration())).style(Style::default().fg(if sel { GOLD } else { DIM }).bg(bg)),
            ]).style(Style::default().fg(fg).bg(bg))
        })
        .collect();

    let title = format!(" Tracks ({}) ", tracks.len());
    let scroll_info = if tracks.len() > visible { format!(" [{}-{}/{}] ", offset + 1, end, tracks.len()) } else { String::new() };
    f.render_widget(
        Table::new(rows, [Constraint::Length(6), Constraint::Percentage(35), Constraint::Percentage(25), Constraint::Percentage(25), Constraint::Length(8)])
            .header(header)
            .block(table_block_with_scroll(title, scroll_info)),
        area,
    );
}

// ── Album track list ─────────────────────────────────────────────────────────

fn render_album_track_list(f: &mut Frame, tracks: &[crate::api::Track], selected: usize, offset: usize, area: Rect) {
    if tracks.is_empty() {
        f.render_widget(Paragraph::new("No tracks.").style(Style::default().fg(DIM)).alignment(Alignment::Center).block(table_block(" Tracks ".into())), area);
        return;
    }

    let visible = list_visible_height(area);
    let end = (offset + visible).min(tracks.len());
    let header = Row::new(vec![
        Cell::from("  # "), Cell::from("Title"), Cell::from("Artist"), Cell::from("  Time"),
    ]).style(Style::default().fg(BLUE).add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = tracks[offset..end]
        .iter()
        .enumerate()
        .map(|(vi, track)| {
            let i = offset + vi;
            let sel = i == selected;
            let bg = row_bg(sel, i);
            let fg = if sel { GOLD } else { WHITE };
            let marker = if sel { " \u{25B6} " } else { "   " };
            let num = track.track_number.unwrap_or((i + 1) as u32);
            Row::new(vec![
                Cell::from(format!("{}{}", marker, num)).style(Style::default().fg(if sel { GOLD } else { DIM }).bg(bg)),
                Cell::from(track.title.as_str()),
                Cell::from(track.artist_name()).style(Style::default().fg(if sel { GOLD } else { MUTED }).bg(bg)),
                Cell::from(format!("  {}", track.format_duration())).style(Style::default().fg(if sel { GOLD } else { DIM }).bg(bg)),
            ]).style(Style::default().fg(fg).bg(bg))
        })
        .collect();

    let title = format!(" Tracks ({}) ", tracks.len());
    let scroll_info = if tracks.len() > visible { format!(" [{}-{}/{}] ", offset + 1, end, tracks.len()) } else { String::new() };
    f.render_widget(
        Table::new(rows, [Constraint::Length(6), Constraint::Percentage(50), Constraint::Percentage(35), Constraint::Length(8)])
            .header(header)
            .block(table_block_with_scroll(title, scroll_info)),
        area,
    );
}

// ── Album list ───────────────────────────────────────────────────────────────

fn render_album_list(f: &mut Frame, albums: &[crate::api::Album], selected: usize, offset: usize, area: Rect) {
    if albums.is_empty() {
        f.render_widget(Paragraph::new("No albums.").style(Style::default().fg(DIM)).alignment(Alignment::Center).block(table_block(" Albums ".into())), area);
        return;
    }

    let visible = list_visible_height(area);
    let end = (offset + visible).min(albums.len());
    let header = Row::new(vec![
        Cell::from("  # "), Cell::from("Title"), Cell::from("Artist"), Cell::from("Tracks"), Cell::from("Year"),
    ]).style(Style::default().fg(BLUE).add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = albums[offset..end]
        .iter()
        .enumerate()
        .map(|(vi, album)| {
            let i = offset + vi;
            let sel = i == selected;
            let bg = row_bg(sel, i);
            let fg = if sel { GOLD } else { WHITE };
            let marker = if sel { " \u{25B6} " } else { "   " };
            let artist = album.artist.as_ref().map(|a| a.name.as_str()).unwrap_or("Unknown");
            let tracks = album.tracks_count.map(|c| c.to_string()).unwrap_or_else(|| "-".to_string());
            let year = album.release_date_original.as_ref().and_then(|d| d.get(..4)).unwrap_or("-");
            Row::new(vec![
                Cell::from(format!("{}{}", marker, i + 1)).style(Style::default().fg(if sel { GOLD } else { DIM }).bg(bg)),
                Cell::from(album.title.as_str()),
                Cell::from(artist).style(Style::default().fg(if sel { GOLD } else { MUTED }).bg(bg)),
                Cell::from(tracks).style(Style::default().fg(if sel { GOLD } else { DIM }).bg(bg)),
                Cell::from(year).style(Style::default().fg(if sel { GOLD } else { DIM }).bg(bg)),
            ]).style(Style::default().fg(fg).bg(bg))
        })
        .collect();

    let title = format!(" Albums ({}) ", albums.len());
    let scroll_info = if albums.len() > visible { format!(" [{}-{}/{}] ", offset + 1, end, albums.len()) } else { String::new() };
    f.render_widget(
        Table::new(rows, [Constraint::Length(6), Constraint::Percentage(40), Constraint::Percentage(30), Constraint::Length(8), Constraint::Length(6)])
            .header(header)
            .block(table_block_with_scroll(title, scroll_info)),
        area,
    );
}

fn render_favorites(f: &mut Frame, app: &mut App, area: Rect) {
    if !app.favorites_loaded {
        f.render_widget(
            Paragraph::new("Loading favorite albums...").style(Style::default().fg(GOLD)).alignment(Alignment::Center).block(table_block(" Favorite Albums ".into())),
            area,
        );
        return;
    }
    let visible = list_visible_height(area);
    app.favorites_scroll = scroll_offset(app.favorites_selected, app.favorites_scroll, visible);
    render_album_list(f, &app.favorite_albums, app.favorites_selected, app.favorites_scroll, area);
}

// ── Album detail view ────────────────────────────────────────────────────────

fn render_album(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let has_status = app.status_message.is_some();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(if has_status { 1 } else { 0 }),
            Constraint::Min(5),
            Constraint::Length(4),
            Constraint::Length(1),
        ])
        .split(area);

    if let Some(album) = &app.album {
        let artist = album.artist.as_ref().map(|a| a.name.as_str()).unwrap_or("Unknown");
        let year = album.release_date_original.as_ref().and_then(|d| d.get(..4)).unwrap_or("");
        let track_count = album.tracks_count.unwrap_or(0);
        f.render_widget(
            Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(&album.title, Style::default().fg(WHITE).add_modifier(Modifier::BOLD))),
                Line::from(vec![
                    Span::styled(artist, Style::default().fg(BLUE)),
                    Span::styled(format!("  \u{2022}  {} tracks  \u{2022}  {}", track_count, year), Style::default().fg(DIM)),
                ]),
            ])
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(BLUE)).style(Style::default().bg(SURFACE))
                .title(Span::styled(" \u{25C0} Backspace ", Style::default().fg(DIM)))),
            chunks[0],
        );
    }

    // Status message (download progress, errors, etc.)
    if let Some(status) = &app.status_message {
        let style = if status.contains("rror") { Style::default().fg(RED) } else { Style::default().fg(GOLD) };
        f.render_widget(Paragraph::new(format!(" {}", status)).style(style), chunks[1]);
    }

    let visible = list_visible_height(chunks[2]);
    app.album_scroll = scroll_offset(app.album_selected, app.album_scroll, visible);
    render_album_track_list(f, &app.album_tracks, app.album_selected, app.album_scroll, chunks[2]);
    render_player_bar(f, app, chunks[3]);
    render_help_bar(f, app, chunks[4]);
}

// ── Player bar with progress ─────────────────────────────────────────────────

fn render_player_bar(f: &mut Frame, app: &App, area: Rect) {
    let bar_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(20), Constraint::Length(22)])
        .split(area);

    // Now playing + progress
    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(1)])
        .split(inner_rect(bar_chunks[0], 1, 1));

    let (np_icon, np_style) = if app.player.is_loading {
        ("\u{21BB} ", Style::default().fg(GOLD))
    } else if app.player.is_playing {
        ("\u{25B6} ", Style::default().fg(GREEN))
    } else if app.player.current_track_title.is_some() {
        ("\u{23F8} ", Style::default().fg(MUTED))
    } else {
        ("  ", Style::default().fg(DIM))
    };

    let np_text = if let Some(title) = &app.player.current_track_title {
        let artist = app.player.current_track_artist.as_deref().unwrap_or("");
        let queue_info = if app.queue.len() > 1 {
            format!("  [{}/{}]", app.queue_index + 1, app.queue.len())
        } else {
            String::new()
        };
        let loop_label = app.loop_mode.label();
        let loop_str = if loop_label.is_empty() {
            String::new()
        } else {
            format!("  {}", loop_label)
        };
        if app.player.is_loading {
            format!("{}Loading {} - {}", np_icon, title, artist)
        } else {
            format!("{}{} \u{2022} {}{}{}", np_icon, title, artist, queue_info, loop_str)
        }
    } else {
        format!("{}Nothing playing", np_icon)
    };

    let border_color = if app.player.is_playing { GREEN } else { DIM };
    f.render_widget(
        Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(border_color)).style(Style::default().bg(SURFACE))
            .title(Span::styled(" Now Playing ", Style::default().fg(MUTED))),
        bar_chunks[0],
    );
    f.render_widget(Paragraph::new(np_text).style(np_style), inner_chunks[0]);

    // Progress bar
    if app.player.current_track_title.is_some() && !app.player.is_loading {
        let elapsed = app.player.elapsed_secs();
        let total = app.player.current_track_duration;
        let progress = app.player.progress();
        let bar_width = inner_chunks[1].width as usize;
        let time_str = format!(" {}/{} ", format_duration(elapsed), format_duration(total));
        let bar_avail = bar_width.saturating_sub(time_str.len());
        let filled = (progress * bar_avail as f64) as usize;
        let empty = bar_avail.saturating_sub(filled);

        let progress_line = Line::from(vec![
            Span::styled("\u{2501}".repeat(filled), Style::default().fg(CYAN)),
            Span::styled("\u{2500}".repeat(empty), Style::default().fg(DIM)),
            Span::styled(time_str, Style::default().fg(MUTED)),
        ]);
        f.render_widget(Paragraph::new(progress_line), inner_chunks[1]);
    }

    // Volume
    let vol_pct = (app.player.volume * 100.0) as u32;
    let vol_filled = (app.player.volume * 14.0) as usize;
    let vol_empty = 14usize.saturating_sub(vol_filled);
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("\u{2588}".repeat(vol_filled), Style::default().fg(CYAN)),
            Span::styled("\u{2591}".repeat(vol_empty), Style::default().fg(DIM)),
            Span::styled(format!(" {}%", vol_pct), Style::default().fg(MUTED)),
        ]))
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(DIM)).style(Style::default().bg(SURFACE))
            .title(Span::styled(" Vol \u{25C0}\u{25B6} ", Style::default().fg(MUTED)))),
        bar_chunks[1],
    );
}

// ── Help bar ─────────────────────────────────────────────────────────────────

fn render_help_bar(f: &mut Frame, app: &App, area: Rect) {
    let hints: Vec<(&str, &str)> = match app.screen {
        Screen::Main => match app.tab {
            Tab::Search => vec![("Enter", "Search/Play"), ("Tab", "Tracks/Albums"), ("\u{2191}\u{2193}", "Navigate"), ("p", "Pause"), ("n/N", "Next/Prev"), ("r", "Loop"), ("Esc", "Quit")],
            Tab::Favorites => vec![("Enter", "Open"), ("x", "Unfavorite"), ("\u{2191}\u{2193}", "Navigate"), ("F1", "Search"), ("p", "Pause"), ("r", "Loop"), ("Esc", "Quit")],
        },
        Screen::AlbumView => vec![("Enter", "Play"), ("d", "Download"), ("f", "Favorite"), ("\u{2191}\u{2193}", "Nav"), ("Bksp", "Back"), ("p", "Pause"), ("n/N", "Next/Prev"), ("r", "Loop"), ("Esc", "Quit")],
        Screen::Login => return,
    };
    let spans: Vec<Span> = hints.iter().enumerate().flat_map(|(i, (key, action))| {
        let mut s = vec![
            Span::styled(format!(" {} ", key), Style::default().fg(SURFACE).bg(DIM)),
            Span::styled(format!(" {} ", action), Style::default().fg(MUTED)),
        ];
        if i < hints.len() - 1 { s.push(Span::styled(" ", Style::default())); }
        s
    }).collect();
    f.render_widget(Paragraph::new(Line::from(spans)).style(Style::default().bg(SURFACE)), area);
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn mode_span(text: &str, active: bool) -> Span<'_> {
    if active {
        Span::styled(text, Style::default().fg(SURFACE).bg(BLUE).add_modifier(Modifier::BOLD))
    } else {
        Span::styled(text, Style::default().fg(MUTED))
    }
}

fn row_bg(selected: bool, index: usize) -> Color {
    if selected { Color::Rgb(40, 45, 65) } else if index.is_multiple_of(2) { SURFACE } else { ROW_ALT }
}

fn list_visible_height(area: Rect) -> usize {
    // area height minus borders (2) minus header row (1)
    (area.height as usize).saturating_sub(3)
}

fn table_block(title: String) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(DIM))
        .style(Style::default().bg(SURFACE))
        .title(Span::styled(title, Style::default().fg(MUTED)))
}

fn table_block_with_scroll(title: String, scroll_info: String) -> Block<'static> {
    let mut block = Block::default()
        .borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(DIM))
        .style(Style::default().bg(SURFACE))
        .title(Span::styled(title, Style::default().fg(MUTED)));
    if !scroll_info.is_empty() {
        block = block.title_bottom(Span::styled(scroll_info, Style::default().fg(DIM)));
    }
    block
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

fn inner_rect(area: Rect, h_margin: u16, v_margin: u16) -> Rect {
    Rect::new(
        area.x + h_margin, area.y + v_margin,
        area.width.saturating_sub(h_margin * 2), area.height.saturating_sub(v_margin * 2),
    )
}

fn format_duration(secs: u64) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

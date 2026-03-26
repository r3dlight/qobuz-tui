// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 r3dlight
use crate::app::{scroll_offset, App, LoginField, Screen, SearchMode, Tab};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table, Tabs, Wrap,
};
use ratatui::Frame;

// ── Color palette ────────────────────────────────────────────────────────────
// A rich, modern dark theme inspired by music player aesthetics

const BRAND: Color = Color::Rgb(88, 140, 236);    // Primary brand blue
const ACCENT: Color = Color::Rgb(0, 200, 200);     // Teal accent
const GOLD: Color = Color::Rgb(255, 195, 50);      // Warm gold for selection
const GREEN: Color = Color::Rgb(72, 210, 120);     // Playing indicator
const RED: Color = Color::Rgb(240, 85, 85);        // Errors / hearts
const PURPLE: Color = Color::Rgb(170, 120, 255);   // Loop / special indicators
const ORANGE: Color = Color::Rgb(255, 150, 50);    // Loading

const SURFACE: Color = Color::Rgb(18, 18, 26);     // Deep background
const SURFACE_1: Color = Color::Rgb(25, 25, 36);   // Slightly elevated
const SURFACE_2: Color = Color::Rgb(32, 32, 46);   // Cards / panels
const ROW_ALT: Color = Color::Rgb(22, 22, 33);     // Alternating rows
const ROW_SEL: Color = Color::Rgb(35, 40, 60);     // Selected row

const WHITE: Color = Color::Rgb(230, 230, 240);    // Primary text
const TEXT: Color = Color::Rgb(190, 190, 205);      // Secondary text
const MUTED: Color = Color::Rgb(120, 120, 140);    // Tertiary text
const DIM: Color = Color::Rgb(70, 70, 85);         // Borders / disabled

// ── Unicode icons ────────────────────────────────────────────────────────────

const ICON_PLAY: &str = "\u{25B6}";       // ▶
const ICON_PAUSE: &str = "\u{23F8}";      // ⏸
const ICON_LOADING: &str = "\u{25CC}";    // ◌
const ICON_SEARCH: &str = "\u{25CE}";     // ◎
const ICON_HEART: &str = "\u{2665}";      // ♥
const ICON_MUSIC: &str = "\u{266B}";      // ♫
const ICON_QUEUE: &str = "\u{2261}";      // ≡
const ICON_VOL: &str = "\u{266A}";        // ♪
const ICON_CURSOR: &str = "\u{2588}";     // █

pub fn render(f: &mut Frame, app: &mut App) {
    let bg = Block::default().style(Style::default().bg(SURFACE));
    f.render_widget(bg, f.area());

    match app.screen {
        Screen::Login => render_login(f, app),
        Screen::Main => render_main(f, app),
        Screen::AlbumView => render_album(f, app),
        Screen::PlaylistView => render_playlist_view(f, app),
    }
}

// ── Login ────────────────────────────────────────────────────────────────────

fn render_login(f: &mut Frame, app: &App) {
    let area = f.area();
    let form_width = 52u16.min(area.width.saturating_sub(4));
    let form_height = 17u16.min(area.height.saturating_sub(2));
    let form_area = centered_rect(form_width, form_height, area);

    f.render_widget(Clear, form_area);
    f.render_widget(
        Block::default()
            .borders(Borders::ALL).border_type(BorderType::Rounded)
            .border_style(Style::default().fg(BRAND))
            .style(Style::default().bg(SURFACE_2)),
        form_area,
    );

    let inner = inner_rect(form_area, 3, 1);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // spacing
            Constraint::Length(1), // logo
            Constraint::Length(1), // subtitle
            Constraint::Length(1), // spacing
            Constraint::Length(1), // email label
            Constraint::Length(1), // email input
            Constraint::Length(1), // spacing
            Constraint::Length(1), // password label
            Constraint::Length(1), // password input
            Constraint::Length(1), // spacing
            Constraint::Length(1), // submit
            Constraint::Length(1), // spacing
            Constraint::Length(1), // status
            Constraint::Min(0),   // error
        ])
        .split(inner);

    // Logo
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" \u{266B} ", Style::default().fg(ACCENT)),
            Span::styled("Qobuz", Style::default().fg(WHITE).add_modifier(Modifier::BOLD)),
            Span::styled(" TUI", Style::default().fg(BRAND)),
        ])).alignment(Alignment::Center),
        chunks[1],
    );

    f.render_widget(
        Paragraph::new("\u{2500}\u{2500}\u{2500} Sign in to your account \u{2500}\u{2500}\u{2500}")
            .style(Style::default().fg(DIM)).alignment(Alignment::Center),
        chunks[2],
    );

    let labels = ["Email", "Password"];
    let fields = [LoginField::Email, LoginField::Password];
    for (i, (label, field)) in labels.iter().zip(fields.iter()).enumerate() {
        let label_idx = 4 + i * 3;
        let input_idx = 5 + i * 3;
        let is_focused = app.login_focus == *field;
        let value = &app.login_fields[i];
        let display = if i == 1 { "\u{2022}".repeat(value.len()) } else { value.clone() };

        let lbl_style = if is_focused {
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(MUTED)
        };
        f.render_widget(Paragraph::new(format!("  {}", label)).style(lbl_style), chunks[label_idx]);

        let border_c = if is_focused { ACCENT } else { DIM };
        let input_bg = if is_focused { SURFACE_1 } else { ROW_ALT };
        let cursor = if is_focused { ICON_CURSOR } else { "" };
        f.render_widget(Block::default().style(Style::default().bg(input_bg)).borders(Borders::BOTTOM).border_style(Style::default().fg(border_c)), chunks[input_idx]);
        f.render_widget(Paragraph::new(format!(" {}{}", display, cursor)).style(Style::default().fg(WHITE).bg(input_bg)), chunks[input_idx]);
    }

    // Submit
    let (sub_fg, sub_bg) = if app.login_focus == LoginField::Submit { (SURFACE, BRAND) } else { (BRAND, SURFACE_2) };
    let sub_text = if app.login_loading { format!(" {} Connecting... ", ICON_LOADING) } else { "  \u{2192} Sign In  ".to_string() };
    f.render_widget(Paragraph::new(sub_text).style(Style::default().fg(sub_fg).bg(sub_bg).add_modifier(Modifier::BOLD)).alignment(Alignment::Center), chunks[10]);

    if let Some(s) = &app.login_status {
        f.render_widget(Paragraph::new(s.as_str()).style(Style::default().fg(GOLD)).alignment(Alignment::Center), chunks[12]);
    }
    if let Some(e) = &app.login_error {
        f.render_widget(Paragraph::new(e.as_str()).style(Style::default().fg(RED)).alignment(Alignment::Center).wrap(Wrap { trim: true }), chunks[13]);
    }
}

// ── Main screen ──────────────────────────────────────────────────────────────

fn render_main(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header
            Constraint::Min(5),   // content
            Constraint::Length(5), // player bar (taller for better look)
            Constraint::Length(1), // help bar
        ])
        .split(area);

    render_header(f, app, chunks[0]);
    match app.tab {
        Tab::Search => render_search(f, app, chunks[1]),
        Tab::Favorites => render_favorites(f, app, chunks[1]),
        Tab::Playlists => render_playlists(f, app, chunks[1]),
    }
    render_player_bar(f, app, chunks[2]);
    render_help_bar(f, app, chunks[3]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(56), Constraint::Min(10)])
        .split(area);

    // Tabs with icons (Tab key cycles)
    let tab_titles = vec![
        Line::from(format!(" {} Search ", ICON_SEARCH)),
        Line::from(format!(" {} Albums ", ICON_HEART)),
        Line::from(format!(" {} Playlists ", ICON_MUSIC)),
    ];
    let sel = match app.tab { Tab::Search => 0, Tab::Favorites => 1, Tab::Playlists => 2 };
    f.render_widget(
        Tabs::new(tab_titles)
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(DIM)).style(Style::default().bg(SURFACE_1))
                .title(Span::styled(" \u{266B} QOBUZ TUI ", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))))
            .select(sel)
            .style(Style::default().fg(MUTED))
            .highlight_style(Style::default().fg(BRAND).add_modifier(Modifier::BOLD))
            .divider(Span::styled(" \u{2502} ", Style::default().fg(DIM))),
        cols[0],
    );

    // Right panel
    match app.tab {
        Tab::Search => {
            f.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(format!(" {} ", ICON_SEARCH), Style::default().fg(BRAND)),
                    Span::styled(&app.search_query, Style::default().fg(WHITE)),
                    Span::styled(ICON_CURSOR, Style::default().fg(ACCENT)),
                ]))
                .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(BRAND)).style(Style::default().bg(SURFACE_1))),
                cols[1],
            );
        }
        Tab::Favorites => {
            let n = app.favorite_albums.len();
            f.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(format!(" {} ", ICON_HEART), Style::default().fg(RED)),
                    Span::styled(format!("{} album{}", n, if n != 1 { "s" } else { "" }), Style::default().fg(TEXT)),
                ]))
                .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(DIM)).style(Style::default().bg(SURFACE_1))),
                cols[1],
            );
        }
        Tab::Playlists => {
            let n = app.playlists.len();
            f.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(format!(" {} ", ICON_MUSIC), Style::default().fg(PURPLE)),
                    Span::styled(format!("{} playlist{}", n, if n != 1 { "s" } else { "" }), Style::default().fg(TEXT)),
                ]))
                .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(DIM)).style(Style::default().bg(SURFACE_1))),
                cols[1],
            );
        }
    }
}

// ── Search ───────────────────────────────────────────────────────────────────

fn render_search(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(3)])
        .split(area);

    // Mode switcher + status
    let mut spans = vec![
        Span::styled(" ", Style::default()),
        pill(" Tracks ", app.search_mode == SearchMode::Tracks, BRAND),
        Span::styled("  ", Style::default()),
        pill(" Albums ", app.search_mode == SearchMode::Albums, BRAND),
    ];
    if let Some(status) = &app.status_message {
        let c = if status.contains("rror") { RED } else { GOLD };
        spans.extend([Span::styled("   ", Style::default()), Span::styled(status, Style::default().fg(c))]);
    }
    f.render_widget(Paragraph::new(Line::from(spans)), chunks[0]);

    match app.search_mode {
        SearchMode::Tracks => {
            let v = list_visible_height(chunks[1]);
            app.search_scroll = scroll_offset(app.search_selected, app.search_scroll, v);
            render_track_list(f, &app.search_tracks, app.search_selected, app.search_scroll, chunks[1], true);
        }
        SearchMode::Albums => {
            let v = list_visible_height(chunks[1]);
            app.search_scroll = scroll_offset(app.search_selected, app.search_scroll, v);
            render_album_list(f, &app.search_albums, app.search_selected, app.search_scroll, chunks[1]);
        }
    }
}

// ── Track list ───────────────────────────────────────────────────────────────

fn render_track_list(f: &mut Frame, tracks: &[qobuz_lib::api::Track], selected: usize, offset: usize, area: Rect, show_album: bool) {
    if tracks.is_empty() {
        f.render_widget(
            Paragraph::new("Type a query and press Enter").style(Style::default().fg(DIM)).alignment(Alignment::Center).block(panel(" Tracks ")),
            area,
        );
        return;
    }

    let visible = list_visible_height(area);
    let end = (offset + visible).min(tracks.len());

    let header_cells = if show_album {
        vec![Cell::from(""), Cell::from("  Title"), Cell::from("Artist"), Cell::from("Album"), Cell::from("Time")]
    } else {
        vec![Cell::from(""), Cell::from("  Title"), Cell::from("Artist"), Cell::from("Time")]
    };
    let header = Row::new(header_cells)
        .style(Style::default().fg(BRAND).add_modifier(Modifier::BOLD))
        .bottom_margin(0);

    let rows: Vec<Row> = tracks[offset..end].iter().enumerate().map(|(vi, track)| {
        let i = offset + vi;
        let sel = i == selected;
        let bg = row_bg(sel, i);
        let fg = if sel { GOLD } else { TEXT };
        let num_fg = if sel { GOLD } else { DIM };
        let sub_fg = if sel { GOLD } else { MUTED };
        let marker = if sel { format!(" {} ", ICON_PLAY) } else { format!(" {:2} ", i + 1) };

        let mut cells = vec![
            Cell::from(marker).style(Style::default().fg(num_fg).bg(bg)),
            Cell::from(format!("  {}", track.title)).style(Style::default().fg(fg).bg(bg)),
            Cell::from(track.artist_name()).style(Style::default().fg(sub_fg).bg(bg)),
        ];
        if show_album {
            cells.push(Cell::from(track.album_title()).style(Style::default().fg(if sel { GOLD } else { DIM }).bg(bg)));
        }
        cells.push(Cell::from(track.format_duration()).style(Style::default().fg(if sel { GOLD } else { DIM }).bg(bg)));
        Row::new(cells).style(Style::default().bg(bg))
    }).collect();

    let widths = if show_album {
        vec![Constraint::Length(5), Constraint::Percentage(32), Constraint::Percentage(23), Constraint::Percentage(23), Constraint::Length(7)]
    } else {
        vec![Constraint::Length(5), Constraint::Percentage(48), Constraint::Percentage(34), Constraint::Length(7)]
    };
    let title = format!(" {} Tracks ({}) ", ICON_MUSIC, tracks.len());
    let scroll = if tracks.len() > visible { format!(" {}-{}/{} ", offset + 1, end, tracks.len()) } else { String::new() };
    f.render_widget(Table::new(rows, widths).header(header).block(panel_scroll(title, scroll)), area);
}

// ── Album list ───────────────────────────────────────────────────────────────

fn render_album_list(f: &mut Frame, albums: &[qobuz_lib::api::Album], selected: usize, offset: usize, area: Rect) {
    if albums.is_empty() {
        f.render_widget(Paragraph::new("No albums.").style(Style::default().fg(DIM)).alignment(Alignment::Center).block(panel(" Albums ")), area);
        return;
    }
    let visible = list_visible_height(area);
    let end = (offset + visible).min(albums.len());
    let header = Row::new(vec![Cell::from(""), Cell::from("  Title"), Cell::from("Artist"), Cell::from("Trk"), Cell::from("Year")])
        .style(Style::default().fg(BRAND).add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = albums[offset..end].iter().enumerate().map(|(vi, album)| {
        let i = offset + vi;
        let sel = i == selected;
        let bg = row_bg(sel, i);
        let fg = if sel { GOLD } else { TEXT };
        let sub = if sel { GOLD } else { MUTED };
        let dim = if sel { GOLD } else { DIM };
        let marker = if sel { format!(" {} ", ICON_PLAY) } else { format!(" {:2} ", i + 1) };
        let artist = album.artist.as_ref().map(|a| a.name.as_str()).unwrap_or("?");
        let trk = album.tracks_count.map(|c| c.to_string()).unwrap_or_else(|| "-".into());
        let year = album.release_date_original.as_ref().and_then(|d| d.get(..4)).unwrap_or("-");
        Row::new(vec![
            Cell::from(marker).style(Style::default().fg(dim).bg(bg)),
            Cell::from(format!("  {}", album.title)).style(Style::default().fg(fg).bg(bg)),
            Cell::from(artist).style(Style::default().fg(sub).bg(bg)),
            Cell::from(trk).style(Style::default().fg(dim).bg(bg)),
            Cell::from(year).style(Style::default().fg(dim).bg(bg)),
        ]).style(Style::default().bg(bg))
    }).collect();

    let title = format!(" {} Albums ({}) ", ICON_HEART, albums.len());
    let scroll = if albums.len() > visible { format!(" {}-{}/{} ", offset + 1, end, albums.len()) } else { String::new() };
    f.render_widget(
        Table::new(rows, [Constraint::Length(5), Constraint::Percentage(38), Constraint::Percentage(28), Constraint::Length(5), Constraint::Length(6)])
            .header(header).block(panel_scroll(title, scroll)),
        area,
    );
}

fn render_favorites(f: &mut Frame, app: &mut App, area: Rect) {
    if !app.favorites_loaded {
        f.render_widget(Paragraph::new(format!(" {} Loading...", ICON_LOADING)).style(Style::default().fg(GOLD)).alignment(Alignment::Center).block(panel(" Favorites ")), area);
        return;
    }
    let v = list_visible_height(area);
    app.favorites_scroll = scroll_offset(app.favorites_selected, app.favorites_scroll, v);
    render_album_list(f, &app.favorite_albums, app.favorites_selected, app.favorites_scroll, area);
}

// ── Playlists ────────────────────────────────────────────────────────────────

fn render_playlists(f: &mut Frame, app: &mut App, area: Rect) {
    if !app.playlists_loaded {
        f.render_widget(Paragraph::new(format!(" {} Loading...", ICON_LOADING)).style(Style::default().fg(GOLD)).alignment(Alignment::Center).block(panel(" Playlists ")), area);
        return;
    }
    if app.playlists.is_empty() {
        f.render_widget(Paragraph::new("No playlists.").style(Style::default().fg(DIM)).alignment(Alignment::Center).block(panel(" Playlists ")), area);
        return;
    }
    let visible = list_visible_height(area);
    app.playlists_scroll = scroll_offset(app.playlists_selected, app.playlists_scroll, visible);
    let offset = app.playlists_scroll;
    let end = (offset + visible).min(app.playlists.len());

    let header = Row::new(vec![Cell::from(""), Cell::from("  Name"), Cell::from("Owner"), Cell::from("Trk")])
        .style(Style::default().fg(BRAND).add_modifier(Modifier::BOLD));
    let rows: Vec<Row> = app.playlists[offset..end].iter().enumerate().map(|(vi, pl)| {
        let i = offset + vi;
        let sel = i == app.playlists_selected;
        let bg = row_bg(sel, i);
        let fg = if sel { GOLD } else { TEXT };
        let sub = if sel { GOLD } else { MUTED };
        let dim = if sel { GOLD } else { DIM };
        let marker = if sel { format!(" {} ", ICON_PLAY) } else { format!(" {:2} ", i + 1) };
        let owner = pl.owner.as_ref().map(|o| o.name.as_str()).unwrap_or("");
        let trk = pl.tracks_count.map(|c| c.to_string()).unwrap_or_else(|| "-".into());
        Row::new(vec![
            Cell::from(marker).style(Style::default().fg(dim).bg(bg)),
            Cell::from(format!("  {}", pl.name)).style(Style::default().fg(fg).bg(bg)),
            Cell::from(owner).style(Style::default().fg(sub).bg(bg)),
            Cell::from(trk).style(Style::default().fg(dim).bg(bg)),
        ]).style(Style::default().bg(bg))
    }).collect();

    let title = format!(" {} Playlists ({}) ", ICON_MUSIC, app.playlists.len());
    f.render_widget(
        Table::new(rows, [Constraint::Length(5), Constraint::Percentage(50), Constraint::Percentage(30), Constraint::Length(5)])
            .header(header).block(panel(title)),
        area,
    );
}

// ── Detail views ─────────────────────────────────────────────────────────────

fn render_playlist_view(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let has_status = app.status_message.is_some();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(if has_status { 1 } else { 0 }), Constraint::Min(5), Constraint::Length(5), Constraint::Length(1)])
        .split(area);

    let name = app.playlist_name.as_deref().unwrap_or("Playlist");
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(format!(" {} ", ICON_MUSIC), Style::default().fg(PURPLE)),
            Span::styled(name, Style::default().fg(WHITE).add_modifier(Modifier::BOLD)),
        ]))
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(PURPLE)).style(Style::default().bg(SURFACE_2))
            .title(Span::styled(" \u{25C0} Backspace ", Style::default().fg(DIM)))),
        chunks[0],
    );
    render_status(f, app, chunks[1]);
    let v = list_visible_height(chunks[2]);
    app.playlist_scroll = scroll_offset(app.playlist_selected, app.playlist_scroll, v);
    render_track_list(f, &app.playlist_tracks, app.playlist_selected, app.playlist_scroll, chunks[2], false);
    render_player_bar(f, app, chunks[3]);
    render_help_bar(f, app, chunks[4]);
}

fn render_album(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let has_status = app.status_message.is_some();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Length(if has_status { 1 } else { 0 }), Constraint::Min(5), Constraint::Length(5), Constraint::Length(1)])
        .split(area);

    if let Some(album) = &app.album {
        let artist = album.artist.as_ref().map(|a| a.name.as_str()).unwrap_or("Unknown");
        let year = album.release_date_original.as_ref().and_then(|d| d.get(..4)).unwrap_or("");
        let tc = album.tracks_count.unwrap_or(0);
        f.render_widget(
            Paragraph::new(vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled(format!(" {} ", ICON_MUSIC), Style::default().fg(BRAND)),
                    Span::styled(&album.title, Style::default().fg(WHITE).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::styled(format!("   {}", artist), Style::default().fg(ACCENT)),
                    Span::styled(format!("  \u{2022}  {} tracks  \u{2022}  {}", tc, year), Style::default().fg(DIM)),
                ]),
            ])
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(BRAND)).style(Style::default().bg(SURFACE_2))
                .title(Span::styled(" \u{25C0} Backspace ", Style::default().fg(DIM)))),
            chunks[0],
        );
    }
    render_status(f, app, chunks[1]);
    let v = list_visible_height(chunks[2]);
    app.album_scroll = scroll_offset(app.album_selected, app.album_scroll, v);
    render_track_list(f, &app.album_tracks, app.album_selected, app.album_scroll, chunks[2], false);
    render_player_bar(f, app, chunks[3]);
    render_help_bar(f, app, chunks[4]);
}

// ── Player bar ───────────────────────────────────────────────────────────────

fn render_player_bar(f: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(20), Constraint::Length(20)])
        .split(area);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
        .split(inner_rect(cols[0], 1, 1));

    // Now Playing info
    let (icon, icon_style) = if app.player.is_loading {
        (ICON_LOADING, Style::default().fg(ORANGE))
    } else if app.player.is_playing {
        (ICON_PLAY, Style::default().fg(GREEN))
    } else if app.player.current_track_title.is_some() {
        (ICON_PAUSE, Style::default().fg(MUTED))
    } else {
        (ICON_VOL, Style::default().fg(DIM))
    };

    let np_line = if let Some(title) = &app.player.current_track_title {
        let artist = app.player.current_track_artist.as_deref().unwrap_or("");
        let queue_info = if app.queue.len() > 1 {
            format!("  {} {}/{}", ICON_QUEUE, app.queue_index + 1, app.queue.len())
        } else { String::new() };
        let loop_label = app.loop_mode.label();
        let loop_str = if loop_label.is_empty() { String::new() } else { format!("  {}", loop_label) };

        Line::from(vec![
            Span::styled(format!(" {} ", icon), icon_style),
            Span::styled(title, Style::default().fg(WHITE).add_modifier(Modifier::BOLD)),
            Span::styled(format!("  \u{2022}  {}", artist), Style::default().fg(TEXT)),
            Span::styled(queue_info, Style::default().fg(MUTED)),
            Span::styled(loop_str, Style::default().fg(PURPLE)),
        ])
    } else {
        Line::from(Span::styled(format!(" {} Nothing playing", icon), Style::default().fg(DIM)))
    };

    let border_c = if app.player.is_playing { GREEN } else { DIM };
    f.render_widget(
        Block::default().borders(Borders::ALL).border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_c)).style(Style::default().bg(SURFACE_1))
            .title(Span::styled(" Now Playing ", Style::default().fg(MUTED))),
        cols[0],
    );
    f.render_widget(Paragraph::new(np_line), inner[0]);

    // Progress bar
    if app.player.current_track_title.is_some() && !app.player.is_loading {
        let elapsed = app.player.elapsed_secs();
        let total = app.player.current_track_duration;
        let progress = app.player.progress();
        let width = inner[1].width as usize;
        let time = format!(" {}/{} ", fmt_dur(elapsed), fmt_dur(total));
        let bar_w = width.saturating_sub(time.len());
        let filled = (progress * bar_w as f64) as usize;
        let empty = bar_w.saturating_sub(filled);

        f.render_widget(Paragraph::new(Line::from(vec![
            Span::styled("\u{2501}".repeat(filled), Style::default().fg(ACCENT)),
            Span::styled("\u{2500}".repeat(empty), Style::default().fg(DIM)),
            Span::styled(time, Style::default().fg(MUTED)),
        ])), inner[1]);
    }

    // Seek hint
    if app.player.current_track_title.is_some() {
        let seekable = if app.player.cached_data.is_some() { "seekable" } else { "streaming" };
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(format!(" ,/; seek \u{2022} {}", seekable), Style::default().fg(DIM)))),
            inner[2],
        );
    }

    // Volume
    let vol_pct = (app.player.volume * 100.0) as u32;
    let vol_filled = (app.player.volume * 12.0) as usize;
    let vol_empty = 12usize.saturating_sub(vol_filled);
    f.render_widget(
        Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("\u{2588}".repeat(vol_filled), Style::default().fg(ACCENT)),
                Span::styled("\u{2591}".repeat(vol_empty), Style::default().fg(DIM)),
            ]),
            Line::from(Span::styled(format!("  {}%", vol_pct), Style::default().fg(MUTED))),
        ])
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded)
            .border_style(Style::default().fg(DIM)).style(Style::default().bg(SURFACE_1))
            .title(Span::styled(format!(" {} \u{25C0}\u{25B6} ", ICON_VOL), Style::default().fg(MUTED)))),
        cols[1],
    );
}

// ── Help bar ─────────────────────────────────────────────────────────────────

fn render_help_bar(f: &mut Frame, app: &App, area: Rect) {
    let h: Vec<(&str, &str)> = match app.screen {
        Screen::Main => match app.tab {
            Tab::Search => vec![("Enter", "Search/Play"), ("S-Tab", "Trk/Alb"), ("Tab", "Next tab"), ("\u{2191}\u{2193}", "Nav"), ("p", "Pause"), ("n/N", "Skip"), ("r", "Loop"), ("Esc", "Quit")],
            Tab::Favorites => vec![("Enter", "Open"), ("x", "Unfav"), ("Tab", "Next tab"), ("\u{2191}\u{2193}", "Nav"), ("p", "Pause"), ("n/N", "Skip"), ("r", "Loop"), ("Esc", "Quit")],
            Tab::Playlists => vec![("Enter", "Open"), ("Tab", "Next tab"), ("\u{2191}\u{2193}", "Nav"), ("p", "Pause"), ("n/N", "Skip"), ("r", "Loop"), ("Esc", "Quit")],
        },
        Screen::AlbumView => vec![("Enter", "Play"), ("d", "DL"), ("f", "Fav"), ("Bksp", "Back"), ("p", "Pause"), ("n/N", "Skip"), ("r", "Loop"), ("Esc", "Quit")],
        Screen::PlaylistView => vec![("Enter", "Play"), ("Bksp", "Back"), ("p", "Pause"), ("n/N", "Skip"), ("r", "Loop"), ("Esc", "Quit")],
        Screen::Login => return,
    };
    let spans: Vec<Span> = h.iter().enumerate().flat_map(|(i, (k, a))| {
        let mut s = vec![
            Span::styled(format!(" {} ", k), Style::default().fg(SURFACE).bg(MUTED)),
            Span::styled(format!(" {} ", a), Style::default().fg(DIM)),
        ];
        if i < h.len() - 1 { s.push(Span::raw(" ")); }
        s
    }).collect();
    f.render_widget(Paragraph::new(Line::from(spans)).style(Style::default().bg(SURFACE)), area);
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn render_status(f: &mut Frame, app: &App, area: Rect) {
    if let Some(s) = &app.status_message {
        let c = if s.contains("rror") { RED } else { GOLD };
        f.render_widget(Paragraph::new(format!(" {}", s)).style(Style::default().fg(c)), area);
    }
}

fn pill(text: &str, active: bool, color: Color) -> Span<'_> {
    if active {
        Span::styled(text, Style::default().fg(SURFACE).bg(color).add_modifier(Modifier::BOLD))
    } else {
        Span::styled(text, Style::default().fg(MUTED))
    }
}

fn row_bg(selected: bool, index: usize) -> Color {
    if selected { ROW_SEL } else if index.is_multiple_of(2) { SURFACE } else { ROW_ALT }
}

fn list_visible_height(area: Rect) -> usize {
    (area.height as usize).saturating_sub(3)
}

fn panel<S: Into<String>>(title: S) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(DIM))
        .style(Style::default().bg(SURFACE))
        .title(Span::styled(title.into(), Style::default().fg(MUTED)))
}

fn panel_scroll(title: String, scroll: String) -> Block<'static> {
    let mut b = panel(title);
    if !scroll.is_empty() {
        b = b.title_bottom(Span::styled(scroll, Style::default().fg(DIM)));
    }
    b
}

fn centered_rect(w: u16, h: u16, area: Rect) -> Rect {
    Rect::new(
        area.x + (area.width.saturating_sub(w)) / 2,
        area.y + (area.height.saturating_sub(h)) / 2,
        w.min(area.width), h.min(area.height),
    )
}

fn inner_rect(area: Rect, h: u16, v: u16) -> Rect {
    Rect::new(area.x + h, area.y + v, area.width.saturating_sub(h * 2), area.height.saturating_sub(v * 2))
}

fn fmt_dur(s: u64) -> String {
    format!("{}:{:02}", s / 60, s % 60)
}

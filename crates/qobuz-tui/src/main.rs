// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 r3dlight
#![deny(unsafe_code)]

mod app;
mod sandbox;
mod ui;

use anyhow::Result;
use app::{App, AppMessage};
use crossterm::event::{self, Event};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use qobuz_lib::{Config, Player};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use rodio::{OutputStream, Sink};
use std::io::stdout;
use std::time::Duration;
use tokio::sync::mpsc;

fn main() -> Result<()> {
    // Redirect stderr to /dev/null to suppress ALSA diagnostic messages
    // (e.g. snd_pcm_recover) that write directly to fd 2 and corrupt the TUI.
    // This must happen before OutputStream::try_default() which initializes ALSA.
    #[cfg(unix)]
    redirect_stderr();

    let (_stream, stream_handle) = OutputStream::try_default()
        .map_err(|e| anyhow::anyhow!("Failed to open audio output: {}", e))?;
    let sink = Sink::try_new(&stream_handle)?;
    let player = Player::new(sink);

    let config_dir = Config::path()
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("qobuz-tui");
    if let Err(e) = sandbox::apply(&config_dir, &cache_dir) {
        eprintln!("[sandbox] Warning: {}", e);
    }

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let config = Config::load();
        let (tx, rx) = mpsc::unbounded_channel();
        let app = App::new(config, player, tx);
        run_tui(app, rx).await
    })
}

async fn run_tui(
    mut app: App,
    mut rx: mpsc::UnboundedReceiver<AppMessage>,
) -> Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = stdout().execute(LeaveAlternateScreen);
        original_hook(panic_info);
    }));

    loop {
        terminal.draw(|f| ui::render(f, &mut app))?;

        while let Ok(msg) = rx.try_recv() {
            app.handle_message(msg);
        }

        app.tick();

        if event::poll(Duration::from_millis(50))?
            && let Event::Key(key) = event::read()?
            && key.kind == crossterm::event::KeyEventKind::Press
        {
            app.handle_key(key);
        }

        if app.should_quit {
            break;
        }
    }

    app.save_session();

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

/// Redirect stderr (fd 2) to /dev/null so that ALSA C library messages
/// don't corrupt the ratatui alternate screen.
/// Called once at the start of main() before any threads are spawned.
/// No unsafe code — uses nix::unistd::dup2_stderr which is a safe wrapper.
#[cfg(unix)]
fn redirect_stderr() {
    if let Ok(devnull) = std::fs::File::open("/dev/null") {
        let _ = nix::unistd::dup2_stderr(&devnull);
    }
}

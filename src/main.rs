mod api;
mod app;
mod cache;
mod config;
mod player;
mod sandbox;
mod stream;
mod ui;

use anyhow::Result;
use app::{App, AppMessage};
use config::Config;
use crossterm::event::{self, Event};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use player::Player;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use rodio::{OutputStream, Sink};
use std::io::stdout;
use std::time::Duration;
use tokio::sync::mpsc;

fn main() -> Result<()> {
    let (_stream, stream_handle) = OutputStream::try_default()
        .map_err(|e| anyhow::anyhow!("Failed to open audio output: {}", e))?;
    let sink = Sink::try_new(&stream_handle)?;
    let player = Player::new(sink);

    // Apply Landlock sandbox: restrict filesystem to config + cache dirs,
    // allow outbound TCP for Qobuz API. Must happen after audio init
    // (which needs /dev access during OutputStream::try_default).
    let config_dir = Config::path().parent().unwrap().to_path_buf();
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

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

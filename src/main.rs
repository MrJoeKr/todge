mod app;
mod config;
mod dash;
mod obstacle;
mod player;
mod ultimate;

use app::App;
use std::fs::File;
use std::io;
use tracing_subscriber::{EnvFilter, fmt};

fn main() -> io::Result<()> {
    let log_file = File::create("debug.log")?;

    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(log_file)
        .with_ansi(false)
        .init();

    ratatui::run(|terminal| App::new().run(terminal))
}

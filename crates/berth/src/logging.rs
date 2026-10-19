use anyhow::Context;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::prelude::*;

use berth_core::logging::SizeRotatingWriter;

pub const LOG_MAX_BYTES: u64 = 1024 * 1024;
pub const LOG_MAX_FILES: usize = 3;

pub struct Logging {
    _guard: WorkerGuard,
}

pub fn init(state_dir: &std::path::Path, verbose: bool) -> anyhow::Result<Logging> {
    let writer = SizeRotatingWriter::open(state_dir, "berth.log", LOG_MAX_BYTES, LOG_MAX_FILES)
        .with_context(|| format!("failed to open the log file in {}", state_dir.display()))?;
    let (file_writer, guard) = tracing_appender::non_blocking(writer);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_writer(file_writer);
    let stderr_layer =
        verbose.then(|| tracing_subscriber::fmt::layer().with_writer(std::io::stderr));

    tracing_subscriber::registry()
        .with(file_layer)
        .with(stderr_layer)
        .try_init()
        .context("failed to initialize logging")?;

    Ok(Logging { _guard: guard })
}

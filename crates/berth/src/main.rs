#![forbid(unsafe_code)]

mod logging;

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    let paths =
        berth_core::paths::Paths::discover().context("failed to determine berth directories")?;
    paths
        .ensure_dirs()
        .context("failed to create berth directories")?;

    let _logging =
        logging::init(paths.state_dir(), false).context("failed to initialize logging")?;

    let config = berth_core::config::Config::load(&paths.config_file(), paths.default_bin_dir())
        .context("failed to load the berth configuration")?;

    let _lock = berth_core::lock::ProcessLock::acquire(&paths.lock_file())
        .context("failed to acquire the berth process lock")?;

    let connection =
        berth_core::db::open(&paths.db_file()).context("failed to open the berth database")?;

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        keep_versions = config.keep_versions,
        concurrency = config.concurrency,
        bin_dir = %config.bin_dir.display(),
        "berth initialized"
    );

    drop(connection);
    Ok(())
}

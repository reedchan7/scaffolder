use std::path::Path;
use std::process::Command;

use anyhow::Context;
use indicatif::{ProgressBar, ProgressStyle};

use crate::config::PackageManager;

/// Initialize a git repository in `dir`.
pub fn git_init(dir: &Path) -> anyhow::Result<()> {
    run(dir, "git", &["init", "-q"])
}

/// Install dependencies using the chosen package manager, with a spinner.
pub fn install(dir: &Path, pm: PackageManager) -> anyhow::Result<()> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::with_template("{spinner} {msg}").expect("valid spinner template"),
    );
    spinner.set_message(format!("{} install", pm.bin()));
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    let result = run(dir, pm.bin(), &["install"]);

    spinner.finish_and_clear();
    result
}

fn run(dir: &Path, cmd: &str, args: &[&str]) -> anyhow::Result<()> {
    let status = Command::new(cmd)
        .args(args)
        .current_dir(dir)
        .status()
        .with_context(|| format!("failed to run `{cmd}` (is it installed and on PATH?)"))?;
    if !status.success() {
        anyhow::bail!("`{cmd} {}` failed with {status}", args.join(" "));
    }
    Ok(())
}

use axoupdater::AxoUpdater;

pub fn run() -> anyhow::Result<()> {
    let mut updater = AxoUpdater::new_for("scaffolder");

    // No install receipt → built/copied manually; self-update can't work.
    if updater.load_receipt().is_err() {
        eprintln!(
            "self-update unavailable: no install receipt found.\n\
             This binary was likely built from source or copied manually.\n\
             Reinstall via the install script to enable self-update."
        );
        return Ok(());
    }

    match updater.run_sync()? {
        Some(result) => println!("Updated to {}", result.new_version),
        None => println!("Already up to date."),
    }
    Ok(())
}

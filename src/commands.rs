use anyhow::{bail, Context};
use log::{debug, warn};
use std::{
    path::Path,
    process::{Command, Stdio},
};

pub fn spawn(command: &[String]) -> anyhow::Result<()> {
    let Some(program) = command.first() else {
        bail!("empty command")
    };
    Command::new(program)
        .args(&command[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("failed to start {program}"))?;
    debug!("started command {program}");
    Ok(())
}

pub fn autostart(path: &Path) {
    if !path.exists() {
        debug!("autostart not present: {}", path.display());
        return;
    }
    let command = vec![path.to_string_lossy().into_owned()];
    if let Err(error) = spawn(&command) {
        warn!("autostart failed: {error:#}");
    }
}

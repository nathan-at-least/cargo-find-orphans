use crate::find_orphans;
use anyhow_std::PathAnyhow;

pub fn run() -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let mut orphans = String::new();
    for orphan in find_orphans(&cwd)? {
        let relpath = orphan.strip_prefix_anyhow(&cwd)?;
        let rpdisp = relpath.display();
        orphans = format!("{orphans}{rpdisp}\n");
    }

    if orphans.is_empty() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "orphans found (these are .rs files which are not mods within the crate):\n{}",
            orphans
        ))
    }
}

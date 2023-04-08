use anyhow_std::PathAnyhow;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn main() -> anyhow::Result<()> {
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
        Err(anyhow::anyhow!("orphans found:\n{}", orphans))
    }
}

fn find_orphans<P>(cratedir: P) -> anyhow::Result<Vec<PathBuf>>
where
    P: AsRef<Path>,
{
    let cratedir = cratedir.as_ref().canonicalize_anyhow()?;
    let mut modpaths = BTreeSet::new();
    let mut entrypaths = vec![];
    for entrypoint in &["src/main.rs", "src/lib.rs"] {
        let entrypath = cratedir.join(entrypoint);
        if entrypath.is_file() {
            find_mod_paths(&mut modpaths, &entrypath, true)?;
            entrypaths.push(entrypath.to_path_buf());
        }
    }

    let mut fspaths = BTreeSet::new();
    find_rs_paths(&mut fspaths, Path::new("src/"))?;

    for entrypath in entrypaths {
        fspaths.remove(&entrypath);
    }

    Ok(fspaths
        .difference(&modpaths)
        .map(|pb| pb.to_path_buf())
        .collect())
}

fn find_mod_paths(
    paths: &mut BTreeSet<PathBuf>,
    path: &Path,
    toplevel: bool,
) -> anyhow::Result<()> {
    let parent = path.parent_anyhow()?;
    let contents = path.read_to_string_anyhow()?;
    let file = syn::parse_file(&contents)?;
    for item in file.items {
        if let syn::Item::Mod(m) = item {
            if m.content.is_none() {
                let filename = format!("{}.rs", m.ident);
                let subpath = if toplevel {
                    parent.join(filename)
                } else {
                    let stem = path.file_stem_anyhow()?;
                    parent.join(stem).join(filename)
                };
                paths.insert(subpath.canonicalize_anyhow()?);
                find_mod_paths(paths, &subpath, false)?;
            } else {
                // Do nothing for embedded mods?
                /*
                use quote::ToTokens;

                unimplemented!("in {:?}:\n{:#}", path.display(), m.into_token_stream());
                */
            }
        }
    }
    Ok(())
}

fn find_rs_paths(paths: &mut BTreeSet<PathBuf>, dir: &Path) -> anyhow::Result<()> {
    for entres in dir.read_dir_anyhow()? {
        let entry = entres?;
        let entry_path = entry.path();
        let entry_type = entry.file_type()?;
        if entry_type.is_dir() {
            find_rs_paths(paths, &entry_path)?;
        } else if entry_type.is_file() && entry_path.to_str_anyhow()?.ends_with(".rs") {
            paths.insert(entry_path.canonicalize_anyhow()?);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;

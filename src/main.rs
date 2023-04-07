use anyhow_std::PathAnyhow;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn main() -> anyhow::Result<()> {
    let mut orphans = String::new();
    for orphan in find_orphans(".")? {
        orphans = format!("{}{}\n", orphans, orphan.display());
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
    let mut modpaths = BTreeSet::new();
    let mainpath = cratedir.as_ref().join("src/main.rs");
    find_mod_paths(&mut modpaths, &mainpath, true)?;

    let mut fspaths = BTreeSet::new();
    find_rs_paths(&mut fspaths, Path::new("src/"))?;
    fspaths.remove(&mainpath);

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
        match item {
            syn::Item::Mod(m) => {
                if m.content.is_none() {
                    let filename = format!("{}.rs", m.ident);
                    let subpath = if toplevel {
                        parent.join(filename)
                    } else {
                        let stem = path.file_stem_anyhow()?;
                        parent.join(stem).join(filename)
                    };
                    paths.insert(subpath.to_path_buf());
                    find_mod_paths(paths, &subpath, false)?;
                } else {
                    // Do nothing for embedded mods?
                    /*
                    use quote::ToTokens;

                    unimplemented!("in {:?}:\n{:#}", path.display(), m.into_token_stream());
                    */
                }
            }
            _ => {}
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
            paths.insert(entry_path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;

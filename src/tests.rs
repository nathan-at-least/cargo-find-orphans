use anyhow_std::{OsStrAnyhow, PathAnyhow};
use include_dir::{include_dir, Dir};
use std::path::Path;
use std::process::Command;

static TEST_CASES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/tests/");

#[test]
fn all_fs_cases() -> anyhow::Result<()> {
    let tempdir = tempfile::TempDir::new()?;
    let tdpath = tempdir.path();
    unpack_dir(&tdpath, &TEST_CASES)?;

    let mut failures = vec![];

    for entres in tdpath.read_dir_anyhow()? {
        let entry = entres?;
        let path = entry.path();

        if entry.file_type()?.is_dir() {
            if let Some(err) = test_crate(&path).err() {
                failures.push(format!(
                    "test crate {:?}: {:#}\n",
                    path.file_name_anyhow()?,
                    err
                ));
            }
        } else {
            eprintln!("Unexpected internal test data path: {:?}", path.display());
        }
    }

    if failures.is_empty() {
        Ok(())
    } else {
        let mut errmsg = "Failures:\n\n".to_string();
        for f in failures {
            errmsg += &f;
        }
        errmsg += &format!(
            "\nLeaving test directory: {:?}",
            tempdir.into_path().display()
        );
        Err(anyhow::Error::msg(errmsg))
    }
}

fn test_crate(cratedir: &Path) -> anyhow::Result<()> {
    println!("Test crate {:?}", cratedir.display());

    let file_name = cratedir.file_name_anyhow()?.to_str_anyhow()?;

    std::env::set_current_dir(cratedir)?;
    cargo_check()?;
    test_find_orphans(file_name, &cratedir)?;
    test_find_orphans(file_name, Path::new("."))?;
    Ok(())
}

fn test_find_orphans(file_name: &str, cratedir: &Path) -> anyhow::Result<()> {
    println!("Finding orphans in {:?}...", cratedir.display());
    let orphans = crate::find_orphans(&cratedir)?;

    if file_name.starts_with("ok-") {
        if orphans.is_empty() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("unexpected failure:\n{orphans:#?}"))
        }
    } else if file_name.starts_with("err-") {
        if orphans.is_empty() {
            Err(anyhow::anyhow!("false negative for {:?}", file_name))
        } else if orphans
            .iter()
            .all(|p| p.to_str_anyhow().unwrap().contains("dangly"))
        {
            Ok(())
        } else {
            Err(anyhow::anyhow!("unexpected orphans: {orphans:#?}"))
        }
    } else {
        Err(anyhow::anyhow!(
            "Unexpected internal test crate path: {:?}",
            file_name,
        ))
    }
}

fn unpack_dir(dest: &Path, src: &Dir<'static>) -> anyhow::Result<()> {
    use include_dir::DirEntry::{Dir, File};

    for entry in src.entries() {
        let file_name = entry.path().file_name_anyhow()?;
        let subdest = dest.join(file_name);
        match entry {
            Dir(d) => {
                subdest.create_dir_anyhow()?;
                unpack_dir(&subdest, d)?;
            }
            File(f) => {
                subdest.write_anyhow(f.contents())?;
            }
        }
    }

    Ok(())
}

fn cargo_check() -> anyhow::Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.arg("check");
    println!("Running {:?}", &cmd);
    let status = cmd.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Command {cmd:?}\nExit status: {status:?}"))
    }
}

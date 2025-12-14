use std::{fs, io, path::Path};

use anyhow::Result;

use anyhow::{Context, bail};
use directories::ProjectDirs;

use crate::interface::BasaltInterface;

mod interface;

const DEFAULT_DATABASE_NAME: &str = "origin";
const DEFAULT_INIT_MD_CONTENTS: &str = "# init.md

A blank page... so much opportunity.
";

fn init_default_db(data_path: &Path) -> Result<()> {
    let db_pathbuf = data_path.join(DEFAULT_DATABASE_NAME);
    let db_init_md_pathbuf = db_pathbuf.join("init.md");

    fs::create_dir(db_pathbuf.as_path())?;
    fs::write(db_init_md_pathbuf.as_path(), DEFAULT_INIT_MD_CONTENTS)?;

    Ok(())
}

fn init_data_dirs() -> Result<()> {
    let d = ProjectDirs::from("com", "Basalt", "Basalt")
        .context("could not establish project directories")?;
    let data_path = d.data_dir();

    match fs::create_dir(data_path) {
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
            // directory already exists, load the dbs
        }
        Err(e) => bail!("Failed to create Basalt project dirs: {e:?}"),
        Ok(_) => {
            let _ = init_default_db(data_path);
        }
    }

    Ok(())
}

pub fn run() -> Result<()> {
    init_data_dirs()?;

    BasaltInterface::default().run()
}

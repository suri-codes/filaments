use std::{
    env::current_dir,
    fs::{File, create_dir_all},
    io::Write,
};

use color_eyre::eyre::{Context, Result};

use crate::{
    cli::Commands,
    config::{Config, get_config_dir},
};

impl Commands {
    pub fn process(self) -> Result<()> {
        match self {
            Self::Init { name } => {
                // create the directory
                let dir = current_dir()
                    .context("Failed to get current directory")?
                    .join(&name);

                // create the .filaments folder
                let filaments_dir = dir.join(".filaments");

                create_dir_all(&filaments_dir)
                    .context("Failed to create the filaments directory!")?;

                // create the database inside there
                File::create(filaments_dir.join("filaments.db"))?;

                // write config that sets the filaments directory to current dir!
                let config_kdl = dbg! {Config::generate(&dir)};

                // create the config dir
                let config_dir = get_config_dir();

                create_dir_all(config_dir).expect("creating the config dir should not error");

                let mut config_file = File::create(get_config_dir().join("config.kdl"))
                    .context("Failed to create config file")?;

                write!(config_file, "{config_kdl}")?;

                println!("wrote config to {config_file:#?}");

                // report status!
                println!("Initialized successfully!");
            }
        }

        Ok(())
    }
}

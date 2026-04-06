use std::{
    env::current_dir,
    fs::{File, create_dir_all},
    io::Write,
};

use color_eyre::eyre::{Context, Result};

use crate::{
    cli::{Commands, ZettelSubcommand},
    config::{Config, get_config_dir},
    types::{Kasten, Zettel},
};

impl Commands {
    pub async fn process(self) -> Result<()> {
        match self {
            Self::Init { name } => {
                // create the directory
                let dir = current_dir()
                    .context("Failed to get current directory")?
                    .join(&name);

                Kasten::initialize(dir.clone()).await?;

                // write config that sets the filaments directory to current dir!
                let config_str = dbg! {Config::generate(&dir)}?;

                // create the config dir
                let config_dir = get_config_dir();

                create_dir_all(config_dir).expect("creating the config dir should not error");

                let mut config_file = File::create(get_config_dir().join("config.ron"))
                    .context("Failed to create config file")?;

                write!(config_file, "{config_str}")?;

                println!("wrote config to {config_file:#?}");

                // report status!
                println!("Initialized successfully!");
            }

            Self::Zettel(zettel_sub_command) => {
                let conf = Config::parse()?;
                let mut kt = Kasten::instansiate(conf.fil_dir).await?;

                match zettel_sub_command {
                    ZettelSubcommand::New { title } => {
                        let zettel = Zettel::new(title, &mut kt).await?;
                        println!("Zettel Created! {zettel:#?}");
                    }
                    ZettelSubcommand::List { by_tag: _by_tag } => {}
                }
            }
        }

        Ok(())
    }
}

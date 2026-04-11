use std::{
    env::current_dir,
    fs::{File, create_dir_all},
    io::Write,
};

use color_eyre::eyre::{Context, Result, eyre};
use dto::{
    GroupActiveModel, GroupEntity, HasOne, IntoActiveModel, TagActiveModel, TagEntity,
    TaskActiveModel, TaskEntity, ZettelEntity,
};
use tower_lsp::{LspService, Server};

use crate::{
    cli::{Commands, ZettelSubcommand},
    config::{Config, get_config_dir},
    lsp::Backend,
    types::{Group, Kasten, Priority, Tag, Task, Zettel},
};

impl Commands {
    #[expect(clippy::too_many_lines)]
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
                        let zettel = Zettel::new(title, &mut kt, vec![]).await?;
                        println!("Zettel Created! {zettel:#?}");
                    }
                    ZettelSubcommand::List { by_tag: _by_tag } => {}
                }
            }
            Self::Lsp => {
                let conf = Config::parse().with_context(|| "Failed to parse the config!!")?;
                let kt = Kasten::instansiate(conf.fil_dir)
                    .await
                    .with_context(|| "Failed to initialize a kasten!!")?;

                let stdin = tokio::io::stdin();
                let stdout = tokio::io::stdout();

                let (service, socket) = LspService::new(|client| Backend::new(client, kt));

                Server::new(stdin, stdout, socket).serve(service).await;
            }

            Self::Todo(command) => {
                let conf = Config::parse()?;
                let mut kt = Kasten::instansiate(conf.fil_dir).await?;

                match command {
                    super::TodoSubcommand::Group { name, parent_id } => {
                        // lets create a tag for this first group first
                        let tag: Tag = TagActiveModel::builder()
                            .set_name(name.clone())
                            .insert(&kt.db)
                            .await?
                            .into();

                        let tag_id = tag.id.clone();

                        // then create the zettel for the group
                        let zettel = Zettel::new(name.clone(), &mut kt, vec![tag]).await?;

                        // then insert that shi
                        let inserted = GroupActiveModel::builder()
                            .set_name(name)
                            .set_parent_group_id(parent_id)
                            .set_tag(
                                TagEntity::load()
                                    .filter_by_nano_id(tag_id)
                                    .one(&kt.db)
                                    .await?
                                    .expect("Tag must exist since we just created it")
                                    .into_active_model(),
                            )
                            .set_zettel(
                                ZettelEntity::load()
                                    .filter_by_nano_id(zettel.id)
                                    .one(&kt.db)
                                    .await?
                                    .expect("Zettel must exist since we just created it")
                                    .into_active_model(),
                            )
                            .set_priority(Priority::default())
                            .insert(&kt.db)
                            .await?;

                        // group should also have the accompanying tag for it.
                        let group: Group = GroupEntity::load()
                            .with(TagEntity)
                            .with((ZettelEntity, TagEntity))
                            .filter_by_nano_id(inserted.nano_id)
                            .one(&kt.db)
                            .await?
                            .expect("We just inserted it")
                            .into();

                        println!("created group {group:#?}");
                    }
                    super::TodoSubcommand::Task { name, parent_id } => {
                        // need to create the task
                        let parent = GroupEntity::load()
                            .with(TagEntity)
                            .filter_by_nano_id(parent_id)
                            .one(&kt.db)
                            .await
                            .with_context(|| "failed to communicate with db")?
                            .ok_or_else(|| eyre!("could not find the group"))?;

                        let HasOne::Loaded(tag) = parent.tag else {
                            panic!("this has to be loaded since we just loaded it right above")
                        };

                        let zettel =
                            Zettel::new(name.clone(), &mut kt, vec![(*tag).into()]).await?;

                        let inserted = TaskActiveModel::builder()
                            .set_name(name)
                            .set_group_id(parent.nano_id.clone())
                            .set_priority(Priority::default())
                            .set_zettel(
                                ZettelEntity::load()
                                    .filter_by_nano_id(zettel.id)
                                    .one(&kt.db)
                                    .await?
                                    .expect("Zettel must exist since we just created it")
                                    .into_active_model(),
                            )
                            .insert(&kt.db)
                            .await?;

                        let group = GroupEntity::load()
                            .with(TagEntity)
                            .with((ZettelEntity, TagEntity))
                            .filter_by_nano_id(parent.nano_id)
                            .one(&kt.db)
                            .await?
                            .expect("We just inserted it");

                        let mut task = TaskEntity::load()
                            .with((ZettelEntity, TagEntity))
                            .filter_by_nano_id(inserted.nano_id)
                            .one(&kt.db)
                            .await?
                            .expect("We just inserted it");

                        task.group = HasOne::Loaded(Box::new(group));

                        println!("task: {task:#?}");

                        let task: Task = task.into();

                        println!("created task: {task:#?}");
                    }
                }
            }
            Self::Test => {
                let conf = Config::parse()?;
                let kt = Kasten::instansiate(conf.fil_dir).await?;
                println!("kt: {kt:#?}");
            }
        }

        Ok(())
    }
}

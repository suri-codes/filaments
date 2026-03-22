use clap::{Parser, Subcommand};

use crate::config::{get_config_dir, get_data_dir};

#[derive(Parser, Debug)]
#[command(author, version = version(), about)]
pub struct Cli {
    /// Tick rate, i.e. number of ticks per second
    #[arg(short, long, value_name = "FLOAT", default_value_t = 4.0)]
    pub tick_rate: f64,

    /// Frame rate, i.e. number of frames per second
    #[arg(short, long, value_name = "FLOAT", default_value_t = 60.0)]
    pub frame_rate: f64,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage TARS groups.
    // #[command(subcommand)]
    // Group(GroupSubcommand),

    /// Manage TARS tasks.
    // #[command(subcommand)]
    // Task(TaskSubcommand),

    /// simple testing stuff
    Test,
    // Imports bulk data into TARS
    // NOTE: By default the importer will fill in fields with
    // default values if they arent present / aren't able to be
    // parsed properly
    // Import(ImportArgs),
}

// #[derive(Subcommand, Debug)]
// /// Subcommand to manage tars groups.
// pub enum GroupSubcommand {
//     /// Add a group.
//     Add(GroupAddArgs),
//     /// List groups.
//     List(GroupListArgs),
// }

// #[derive(Debug, Args)]
// pub struct ExportArgs {
//     #[arg(short, long, default_value = "./tars.json")]
//     /// The file-path for data to pe put into.
//     pub out_file: PathBuf,
// }

const VERSION_MESSAGE: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "-",
    env!("VERGEN_GIT_DESCRIBE"),
    " (",
    env!("VERGEN_BUILD_DATE"),
    ")"
);

pub fn version() -> String {
    let author = clap::crate_authors!();

    // let current_exe_path = PathBuf::from(clap::crate_name!()).display().to_string();
    let config_dir_path = get_config_dir().display().to_string();
    let data_dir_path = get_data_dir().display().to_string();

    format!(
        "\
{VERSION_MESSAGE}

Authors: {author}

Config directory: {config_dir_path}
Data directory: {data_dir_path}"
    )
}

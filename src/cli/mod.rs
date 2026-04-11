use clap::{Parser, Subcommand};
use dto::NanoId;

use crate::config::{get_config_dir, get_data_dir};

mod process;

#[derive(Parser, Debug)]
#[command(author, version = version(), about)]
pub struct Cli {
    /// Tick rate, i.e. number of ticks per second
    #[arg(short, long, value_name = "FLOAT", default_value_t = 4.0)]
    pub tick_rate: f64,

    /// Frame rate, i.e. number of frames per second
    #[arg(short, long, value_name = "FLOAT", default_value_t = 60.0)]
    pub frame_rate: f64,

    /// Open the visualizer along with the tui
    #[arg(short, long, default_value_t = false)]
    pub visualizer: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage `Zettel`s
    #[command(subcommand)]
    Zettel(ZettelSubcommand),

    #[command(subcommand)]
    Todo(TodoSubcommand),

    /// Spawn the `LSP`
    Lsp,

    // / Manage TARS groups.
    //  #[command(subcommand)]
    //  Group(GroupSubcommand),

    // / Manage TARS tasks.
    //  #[command(subcommand)]
    //  Task(TaskSubcommand),
    //
    //
    /// Initalize Filaments.
    ///
    /// This will write a default config to ~/.config/filaments,
    /// as well as creating a new "notebook" in the current
    /// directory with the specified name. Note that we currently
    /// only support one notebook.
    Init {
        #[arg(default_value = "ZettelKasten")]
        name: String,
    },
    // NOTE: By default the importer will fill in fields with
    // default values if they arent present / aren't able to be
    // parsed properly
    // Import(ImportArgs),
}

#[derive(Subcommand, Debug)]
/// Subcommand to manage Zettels.
pub enum ZettelSubcommand {
    /// Add a group.
    New {
        #[arg(short, long)]
        /// The file-path for data to pe put into.
        title: String,
    },
    /// List groups.
    List {
        /// Filter by tag
        #[arg(short = 't', long)]
        by_tag: String,
    },
}

#[derive(Subcommand, Debug)]
/// Subcommand to manage To-Do functionality
pub enum TodoSubcommand {
    Group {
        /// The name of this group
        #[arg(short, long)]
        name: String,

        /// If this group has a parent, provide the parent id.
        #[arg(short, long)]
        parent_id: Option<NanoId>,
    },
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
//

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

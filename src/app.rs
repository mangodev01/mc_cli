use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "mc_cli", version = "0.0.1")]
pub struct App {
    #[clap(subcommand)]
    pub command: Subcommand,
}

#[derive(clap::Subcommand, Debug)]
pub enum Subcommand {
    Vanilla {
        #[clap(short, long)]
        version: Option<String>,

        #[clap(short, long, default_value = "10G")]
        mem: String,

        #[clap(short, long, default_value = "Player")]
        username: String,
    },
    Fabric {
        #[clap(short, long)]
        version: Option<String>,

        #[clap(short, long)]
        loader_version: Option<String>,

        #[clap(short, long, default_value = "10G")]
        mem: String,

        #[clap(short, long, default_value = "Player")]
        username: String,
    },
    Quilt {
        #[clap(short, long)]
        version: Option<String>,

        #[clap(short, long)]
        loader_version: Option<String>,

        #[clap(short, long, default_value = "10G")]
        mem: String,

        #[clap(short, long, default_value = "false")]
        use_release: bool,

        #[clap(short, long, default_value = "Player")]
        username: String,
    },
    Liteloader {
        #[clap(short, long)]
        version: Option<String>,

        #[clap(short, long)]
        loader_version: Option<String>,

        #[clap(short, long, default_value = "10G")]
        mem: String,

        #[clap(short, long, default_value = "Player")]
        username: String,
    },
    #[command(about = "List versions")]
    Versions,
    #[command(about = "Open directories or files with the preferred application")]
    Open {
        #[command(subcommand)]
        target: OpenTarget,
    },
}

#[derive(clap::Subcommand, Debug)]
pub enum OpenTarget {
    #[command(about = "Opens the game directory")]
    Game,
    #[command(about = "Opens the mods directory")]
    Mods,
    #[command(about = "Opens the resource packs directory")]
    ResourcePacks,
    #[command(about = "Opens the logs directory")]
    Logs,
    #[command(about = "Opens the saves directory")]
    Saves,
    #[command(about = "Opens the downloads directory")]
    Downloads,
    #[command(about = "Opens the data directory")]
    Data,
    #[command(about = "Opens the config directory")]
    Config,
    #[command(about = "Opens options.txt")]
    McOptions,
}

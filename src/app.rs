use std::path::{Path, PathBuf};

use clap::Parser;
use indicatif::MultiProgress;

use crate::util::LauncherDirs;

#[derive(Parser, Debug)]
#[clap(name = "mc_cli", version = "0.0.1")]
pub struct App {
    #[clap(subcommand)]
    pub command: Subcommand,
}

#[derive(clap::Subcommand, Debug)]
pub enum Subcommand {
    Vanilla {
        version: Option<String>,

        #[clap(short, long, default_value = "10G")]
        mem: String,

        #[clap(short, long, default_value = "Player")]
        username: String,
    },
	Javaagent {
        version: Option<String>,

        #[clap(short, long, default_value = "10G")]
        mem: String,

        #[clap(short, long, default_value = "Player")]
        username: String,
	},
    Fabric {
        version: Option<String>,

        #[clap(short, long)]
        loader_version: Option<String>,

        #[clap(short, long, default_value = "10G")]
        mem: String,

        #[clap(short, long, default_value = "Player")]
        username: String,
    },
	Labric {
		version: Option<String>,

		#[clap(short, long)]
		loader_version: Option<String>,

		#[clap(short, long, default_value = "10G")]
		mem: String,

		#[clap(short, long, default_value = "Player")]
		username: String,
	},
	Babric {
		#[clap(short, long)]
		loader_version: Option<String>,

		#[clap(short, long, default_value = "10G")]
		mem: String,

		#[clap(short, long, default_value = "Player")]
		username: String,
	},
	Ornithe {
		version: Option<String>,

		#[clap(short, long)]
		loader_version: Option<String>,

		#[clap(short, long, default_value = "10G")]
		mem: String,

		#[clap(short, long, default_value = "Player")]
		username: String,
	},
    Quilt {
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
        version: Option<String>,

        #[clap(short, long)]
        loader_version: Option<String>,

        #[clap(short, long, default_value = "10G")]
        mem: String,

        #[clap(short, long, default_value = "Player")]
        username: String,
    },
	Risugami {
		version: Option<String>,

        #[clap(short, long, default_value = "10G")]
        mem: String,

        #[clap(short, long, default_value = "Player")]
        username: String,
	},
	#[command(about = "Install a modpack from an .mrpack")]
	Install {
		mrpack: PathBuf
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

#[derive(Debug, Clone)]
pub struct McCtx<'a> {
	pub dirs: LauncherDirs,
	pub mp: &'a MultiProgress,
	pub opt_version: Option<String>,
	pub opt_loader_version: Option<String>,
	pub limit: String,
	pub version_dir: Option<&'a Path>,
	pub username: String,
	pub javaagent: bool,
}

#[derive(Debug, Clone)]
pub struct McLaunchCtx {
	pub limit: String,
	pub username: String,
	pub javaagent: bool,
	pub extra_game_args: Vec<String>,
	pub launchwrapper: bool
}

impl<'a> McCtx<'a> {
	pub fn into_launch(self, extra_game_args: Vec<String>, launchwrapper: bool) -> McLaunchCtx {
		McLaunchCtx {
			limit: self.limit,
			username: self.username,
			javaagent: self.javaagent,
			extra_game_args,
			launchwrapper,
		}
	}
}

pub trait McLoader<'a> {
	async fn install(&mut self, ctx: McCtx<'a>);
	fn launch(&self, ctx: McLaunchCtx);

	fn game_versions() -> String;

	fn loader_versions() -> String {
		"".to_string()
	}

	fn intermediary_versions() -> String {
		"".to_string()
	}

	fn maven() -> String {
		"".to_string()
	}

	fn id() -> String;
}


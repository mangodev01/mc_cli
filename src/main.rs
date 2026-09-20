#![allow(dead_code, unused_variables)]
mod version;
mod app;
mod mem;
mod util;
mod assets;

mod vanilla;
mod fabric;
mod quilt;
mod ornithe;
mod labric;
mod babric;
mod liteloader;
mod risugami;

use app::OpenTarget;
use clap::Parser;
use cli_table::{Cell as _, Table};
use directories::ProjectDirs;
use indicatif::MultiProgress;

use crate::{app::{McCtx, McLoader as _, Subcommand}, babric::schema::Babric, fabric::schema::Fabric, labric::schema::Labric, liteloader::schema::Liteloader, ornithe::schema::Ornithe, quilt::schema::Quilt, risugami::schema::Risugami, vanilla::schema::Vanilla};

#[tokio::main]
async fn main() {
    let app = app::App::parse();

    let pdirs = ProjectDirs::from("me", "illia", "mc_cli").unwrap();
	let launcher_dir = pdirs.data_dir();

    let dirs = util::LauncherDirs { 
		root_dir: launcher_dir.to_path_buf(),
		game_dir: launcher_dir.join("game"),
		assets_dir: launcher_dir.join("assets"),
		vers_dir: launcher_dir.join("vers") 
	};

    let mp = MultiProgress::new();

    match app.command {
        Subcommand::Vanilla { version, mem, username } => {
			let mut vanilla = Vanilla::default();
			let ctx = McCtx { dirs, mp: &mp, opt_version: version, limit: mem.clone(), version_dir: None, opt_loader_version: None, username: username.clone(), javaagent: false };

            vanilla.install(ctx.clone()).await;
            vanilla.launch(ctx.into_launch(vec![], false));
        },
		Subcommand::Javaagent { version, mem, username } => {
			let mut vanilla = Vanilla::default();
			let ctx = McCtx { dirs, mp: &mp, opt_version: version, limit: mem.clone(), version_dir: None, opt_loader_version: None, username: username.clone(), javaagent: true };

            vanilla.install(ctx.clone()).await;
            vanilla.launch(ctx.into_launch(vec![], false));
		},
        Subcommand::Fabric { version, loader_version, mem, username } => {
			let mut fabric = Fabric::default();

			let ctx = McCtx { dirs, mp: &mp, opt_version: version, limit: mem.clone(), version_dir: None, opt_loader_version: loader_version, username: username.clone(), javaagent: true };

            fabric.install(ctx.clone()).await;
            fabric.launch(ctx.into_launch(vec![], false));
        },
		Subcommand::Labric { version, loader_version, mem, username } => {
			let mut labric = Labric::default();

			let ctx = McCtx { dirs, mp: &mp, opt_version: version, limit: mem.clone(), version_dir: None, opt_loader_version: loader_version, username: username.clone(), javaagent: true };

			labric.install(ctx.clone()).await;
			labric.launch(ctx.into_launch(vec![], false));
		},
		Subcommand::Babric { loader_version, mem, username } => {
			let mut babric = Babric::default();

			let ctx = McCtx { dirs, mp: &mp, opt_version: None, limit: mem.clone(), version_dir: None, opt_loader_version: loader_version, username: username.clone(), javaagent: true };

			babric.install(ctx.clone()).await;
			babric.launch(ctx.into_launch(vec![], false));
		},
		Subcommand::Ornithe { version, loader_version, mem, username } => {
			let mut ornithe = Ornithe::default();

			let ctx = McCtx { dirs, mp: &mp, opt_version: None, limit: mem.clone(), version_dir: None, opt_loader_version: loader_version, username: username.clone(), javaagent: true };

			ornithe.install(ctx.clone()).await;
			ornithe.launch(ctx.into_launch(vec![], false));
		},
        Subcommand::Quilt { version, loader_version, mem, use_release, username } => {
			let mut quilt = Quilt::default();

			let ctx = McCtx { dirs, mp: &mp, opt_version: None, limit: mem.clone(), version_dir: None, opt_loader_version: loader_version, username: username.clone(), javaagent: true };

			quilt.install(ctx.clone()).await;
			quilt.launch(ctx.into_launch(vec![], false));
        },
        Subcommand::Liteloader { version, loader_version, mem, username } => {
			let mut liteloader = Liteloader::default();

			let ctx = McCtx { dirs, mp: &mp, opt_version: version, limit: mem.clone(), version_dir: None, opt_loader_version: loader_version, username: username.clone(), javaagent: false };

			liteloader.install(ctx.clone()).await;
			liteloader.launch(ctx.into_launch(vec![
				"--tweakClass".to_string(),
				"com.mumfrey.liteloader.launch.LiteLoaderTweaker".to_string(),
			], true));
		},
		Subcommand::Risugami { version, mem, username } => {
			let mut risugami = Risugami::default();

			let ctx = McCtx { dirs, mp: &mp, opt_version: version, limit: mem.clone(), version_dir: None, opt_loader_version: None, username: username.clone(), javaagent: false };

			risugami.install(ctx.clone()).await;
			risugami.launch(ctx.into_launch(vec![], true));
		},
        Subcommand::Open { target: OpenTarget::Game } => {
            open::that(dirs.game_dir).unwrap();
        },
        Subcommand::Open { target: OpenTarget::Mods } => {
            let path = dirs.game_dir.join("mods");
            open::that(path).unwrap();
        },
        Subcommand::Open { target: OpenTarget::ResourcePacks } => {
            let path = dirs.game_dir.join("resourcepacks");
            open::that(path).unwrap();
        },
        Subcommand::Open { target: OpenTarget::Saves } => {
            let path = dirs.game_dir.join("saves");
            open::that(path).unwrap();
        },
        Subcommand::Open { target: OpenTarget::Logs } => {
            let path = dirs.game_dir.join("logs");
            open::that(path).unwrap();
        },
        Subcommand::Open { target: OpenTarget::Downloads } => {
            let path = dirs.game_dir.join("downloads");
            open::that(path).unwrap();
        },
        Subcommand::Open { target: OpenTarget::Data } => {
            let path = dirs.game_dir.join("data");
            open::that(path).unwrap();
        },
        Subcommand::Open { target: OpenTarget::Config } => {
            let path = dirs.game_dir.join("config");
            open::that(path).unwrap();
        },
        Subcommand::Open { target: OpenTarget::McOptions } => {
            let path = dirs.game_dir.join("options.txt");
            open::that(path).unwrap();
        },
        Subcommand::Versions => {
            let path = dirs.vers_dir;

            println!("Installed versions:");

            let mut rows = vec![
                vec![
                    "type".cell(),
                    "name".cell()
                ]
            ];

            // Then add directories
            for dir_entry in path.read_dir().unwrap() {
                let path = dir_entry.unwrap().path();
                let fname = path.file_name().unwrap().to_str().unwrap();

                let (type_name, colored_type) = if fname.starts_with("fabric-") {
                    ("fabric", "fabric")
                } else if fname.starts_with("liteloader-") {
                    ("liteloader", "liteloader")
                } else if fname.starts_with("quilt-") {
                    ("quilt", "quilt")
				} else if fname.starts_with("risugami-") {
					("risugami", "risugami")
				} else if fname.starts_with("ornithe-") {
					("ornithe", "ornithe")
				} else if fname.starts_with("babric-") {
					("babric", "babric")
				} else if fname.starts_with("labric-") {
					("legacy_fabric", "legacy_fabric")
                } else {
                    ("vanilla", "vanilla")
                };

                let name = fname.strip_prefix(&format!("{}-", type_name)).unwrap_or(fname);

                rows.push(vec![
                    colored_type.cell(),
                    name.cell(),
                ]);
            }

            // Create table and print
            let table = rows.table();
            println!("{}", table.display().unwrap());
        },
    }
}

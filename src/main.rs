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
mod mrpack;

use std::io::{BufRead as _, Cursor, Read};

use app::OpenTarget;
use byteorder::ReadBytesExt as _;
use clap::Parser;
use cli_table::{Cell as _, Table};
use directories::ProjectDirs;
use indicatif::MultiProgress;
use std::io::BufReader;

use crate::{app::{McCtx, McLoader as _, Subcommand}, babric::schema::Babric, fabric::schema::Fabric, labric::schema::Labric, liteloader::schema::Liteloader, mrpack::{MrpackIndexJson, SideType}, ornithe::schema::Ornithe, quilt::schema::Quilt, risugami::schema::Risugami, vanilla::schema::Vanilla};

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
		Subcommand::Install { mrpack } => {
			let contents = match std::fs::read(mrpack) {
				Ok(mrpack) => mrpack,
				Err(e) => {
					eprintln!("error while trying to read mrpack: {e}");
					std::process::exit(-1);
				}
			};

			let cur = Cursor::new(contents);

			let mut archive = match zip::ZipArchive::new(cur) {
				Ok(archive) => archive,
				Err(e) => {
					eprintln!("error while trying to read mrpack as zip: {e}");
					std::process::exit(-1);
				}
			};

			let mut index = match archive.by_name("modrinth.index.json") {
				Ok(index) => index,
				Err(e) => {
					eprintln!("no modrinth.index.json found in mrpack: {e}");
					std::process::exit(-1);
				}
			};

			let mut index_json = String::new();

			match index.read_to_string(&mut index_json) {
				Err(e) => {
					eprintln!("failed to read modrinth.index.json from mrpack: {e}");
					std::process::exit(-1);
				},
				_ => {}
			};

            let parsed: MrpackIndexJson = match serde_json::from_str(&index_json) {
                Ok(x) => x,
                Err(e) => {
					eprintln!("failed to parse modrinth.index.json: {e}");
					std::process::exit(-1);
                }
            };

            if parsed.game != "minecraft" {
                eprintln!("non-minecraft modpacks unsupported");
                std::process::exit(-1);
            }

            println!("are you sure you wanna install '{}' version {}?", parsed.name, parsed.version_id);
            if let Some(summary) = parsed.summary {
                println!("[{}]", summary);
            }

			let all = parsed.files
				.iter()
				.map(|file| {
					let dependencies = file
						.dependencies
						.iter()
						.map(|(name, version)| format!("{name}@{version}"))
						.collect::<Vec<_>>()
						.join(", ");

					format!("{} [{}]", file.path, dependencies)
				})
				.collect::<Vec<_>>();

			for f in all {
				let mods = f
					.strip_prefix("mods/").unwrap_or(&f);

				let res = &mods
					.strip_prefix("resourcepacks/").unwrap_or(&mods);

				println!("| {}", res);
			}

            println!("> [Y/n] ");

            let mut buf = String::new();
            let _ = std::io::stdin().read_line(&mut buf);

            let lower_buf = buf.to_lowercase();
            let buf = lower_buf.trim();

            if buf == "yes" || buf == "y" || buf == "yea" || buf == "ye" || buf == "" {
                for file in parsed.files {

                    if file.env.client == SideType::Unsupported {
						println!("skipping {} - unsupported for client-side", file.path);
						continue;
					}

                    let mut file_downloaded = false;

                    for download in file.downloads {
						let res = match util::download_async(&mp, &download, &dirs.game_dir.join(&file.path), format!("downloaded {}", file.path)).await {
                            Ok(res) => {
                                file_downloaded = true;
                                res
                            },
                            Err(_) => continue
						};
                    }

                    if !file_downloaded {
                        eprintln!("failed to download file {}, terminating...", file.path);
                        std::process::exit(-1);
                    }
                }
            } else {
                eprintln!("cancelled modpack installation, terminating");
                std::process::exit(-1);
            }
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

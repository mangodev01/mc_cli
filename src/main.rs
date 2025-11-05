#![allow(dead_code, unused_variables)]
mod version;
mod app;
mod vanilla;
mod mem;
mod fabric;
mod util;
mod rules;
mod assets;
mod liteloader;

use app::OpenTarget;
use clap::Parser;
use cli_table::{Cell as _, Table};
use directories::ProjectDirs;
use version::UseQuilt;

use crate::app::Subcommand;

#[tokio::main]
async fn main() {
    let app = app::App::parse();
    let dirs = ProjectDirs::from("me", "illia", "mc_cli").unwrap();
    let game_dir = dirs.data_dir().join("game");

    match app.command {
        Subcommand::Vanilla { version, mem, username } => {
            vanilla::handle(version, mem, true, None, username).await;
        },
        Subcommand::Fabric { version, loader_version, mem, username } => {
            fabric::handle(version, loader_version, mem, UseQuilt::No, username).await;
        },
        Subcommand::Quilt { version, loader_version, mem, use_release, username } => {
            fabric::handle(version, loader_version, mem, UseQuilt::Yes(use_release), username).await;
        },
        Subcommand::Liteloader { version, loader_version, mem, username } => {
            eprintln!("Liteloader isn't implemented yet. Please consider using fabric,quilt,or just running vanilla");
            liteloader::handle(version, loader_version, mem, username).await;
        },
        Subcommand::Open { target: OpenTarget::Game } => {
            open::that(game_dir).unwrap();
        },
        Subcommand::Open { target: OpenTarget::Mods } => {
            let path = game_dir.join("mods");
            open::that(path).unwrap();
        },
        Subcommand::Open { target: OpenTarget::ResourcePacks } => {
            let path = game_dir.join("resourcepacks");
            open::that(path).unwrap();
        },
        Subcommand::Open { target: OpenTarget::Saves } => {
            let path = game_dir.join("saves");
            open::that(path).unwrap();
        },
        Subcommand::Open { target: OpenTarget::Logs } => {
            let path = game_dir.join("logs");
            open::that(path).unwrap();
        },
        Subcommand::Open { target: OpenTarget::Downloads } => {
            let path = game_dir.join("downloads");
            open::that(path).unwrap();
        },
        Subcommand::Open { target: OpenTarget::Data } => {
            let path = game_dir.join("data");
            open::that(path).unwrap();
        },
        Subcommand::Open { target: OpenTarget::Config } => {
            let path = game_dir.join("config");
            open::that(path).unwrap();
        },
        Subcommand::Open { target: OpenTarget::McOptions } => {
            let path = game_dir.join("options.txt");
            open::that(path).unwrap();
        },
        Subcommand::Versions => {
            let path = dirs.data_dir().join("vers");

            println!("Installed versions:");

            let mut rows = vec![
                vec![
                    "TYPE".cell(),
                    "NAME".cell()
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

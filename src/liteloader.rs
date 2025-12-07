use std::{fs, path::PathBuf};
use directories::ProjectDirs;
use indicatif::MultiProgress;

use crate::{util, vanilla, version::{LiteLoaderVersions, MavenMetadataRoot}};

const LITELOADER_VERSIONS_JSON: &'static str = "https://dl.liteloader.com/versions/versions.json";

pub async fn handle(mp: &MultiProgress, opt_version: Option<String>, opt_loader_version: Option<String>, limit: String, username: String) {
    let versions_json_text = util::download_text_no_save_async(mp, LITELOADER_VERSIONS_JSON, "Downloaded liteloader versions json".to_owned()).await.expect("Failed to download liteloader versions json");
    tokio::fs::write("ver.json", versions_json_text.to_string()).await.unwrap();
    let versions_json: LiteLoaderVersions = serde_json::from_str(&versions_json_text).expect("Failed to parse liteloader versions");
    let meta = versions_json.meta;
    let versions = versions_json.versions;
    let version = if opt_version.is_some() {
        if !versions.iter().any(|i| *i.0 == opt_version.clone().unwrap()) {
            eprintln!("The version you provided doesn't exist");
            std::process::exit(-1);
        }
        opt_version.clone().unwrap()
    } else {
        let mut max = 0;
        for version in &versions {
            if version.0.split('.').collect::<Vec<&str>>()[1].parse::<i32>().unwrap() > max {
                max = version.0.split('.').collect::<Vec<&str>>()[1].parse::<i32>().unwrap();
            }
        }
        format!("1.{}", max)
    };

    println!("Launching {}...", version);

    let proj_dirs = ProjectDirs::from("me", "illia", "mc_cli").unwrap();
    let data_dir = proj_dirs.data_dir();
    let vers = data_dir.join("vers");
    let ver_path = vers.join(format!("{}-{}", "liteloader", version));
    create_dirs(vers, ver_path.clone());

    if let Some(lite_loader_artifact) = &versions[&version].artefacts {
        println!("{:?}", versions[&version].artefacts);
    } else if let Some(snap) = &versions[&version].snapshots {
        let repo = &versions[&version].repo;
        let liteloader_path = &format!("{}com/mumfrey/liteloader/{}", &repo.url, snap.liteloader["latest"].version);
        let metadata = util::download_text_no_save_async(mp, &format!("{}/maven-metadata.xml", liteloader_path), "Downloaded metadata".to_owned()).await.expect("Failed to download metadata");
        let xmled_meta: MavenMetadataRoot = serde_xml_rs::from_str(&metadata).unwrap();

        let ll_jar_path = ver_path.join("ll.jar");
        let ll_path = format!("{}/liteloader-{}-{}-{}.jar", &liteloader_path, &version, xmled_meta.versioning.snapshot.timestamp, xmled_meta.versioning.snapshot.buildNumber);

        println!("{}", ll_path);

        let ll = util::download_async(mp, ll_path.as_str(), &ll_jar_path.as_path(), "Downloaded ll jar".to_string()).await.expect("Failed to download ll jar");
    }

    vanilla::handle(mp, Some(version), limit.clone(), false, Some(ver_path.as_path()), username).await;
}

pub fn create_dirs(vers: PathBuf, ver: PathBuf) {
    let _ = fs::create_dir_all(vers.clone());
    let _ = fs::create_dir(ver.clone());
    let _ = fs::create_dir(vers.parent().unwrap().join("game"));
    let _ = fs::create_dir(ver.join("libs"));
    let _ = fs::create_dir(vers.parent().unwrap().join("assets"));
}

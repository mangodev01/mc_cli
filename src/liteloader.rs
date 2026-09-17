use std::{fs, path::{Path, PathBuf}};

use directories::ProjectDirs;
use indicatif::MultiProgress;

use crate::version::{LiteLoaderLibrary, LiteLoaderVersions, MavenMetadataRoot};
use crate::{util, vanilla};

const LITELOADER_VERSIONS_JSON: &str = "https://dl.liteloader.com/versions/versions.json";
const MOJANG_LIBS: &str = "https://libraries.minecraft.net";
const REPO_LITELOADER: &str = "http://repo.liteloader.com";
const MAVEN_CENTRAL: &str = "https://repo1.maven.org/maven2";

async fn url_ok(url: &str) -> bool {
    match reqwest::get(url).await {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}

async fn resolve_snapshot_url(mp: &MultiProgress, base: &str, group_path: &str, artifact: &str, version: &str, classifier: Option<&str>) -> Option<String> {
    let base = base.trim_end_matches('/');
    let metadata_url = format!("{}/{}/{}/{}/maven-metadata.xml", base, group_path, artifact, version);

    if !url_ok(&metadata_url).await {
        return None;
    }

    let text = util::download_text_no_save_async(mp, &metadata_url, format!("Snapshot metadata {}", artifact)).await.ok()?;
    let meta: MavenMetadataRoot = serde_xml_rs::from_str(&text).ok()?;
    let snapshot = &meta.versioning.snapshot;
    let base_ver = version.trim_end_matches("-SNAPSHOT");

    let classifier = classifier.map(|c| format!("-{}", c)).unwrap_or_default();

    Some(format!("{}/{}/{}/{}/{}-{}-{}-{}{}.jar", base, group_path, artifact, version, artifact, base_ver, snapshot.timestamp, snapshot.buildNumber, classifier))
}

async fn download_library(mp: &MultiProgress, lib: &LiteLoaderLibrary, repo_url: &str, libs_dir: &Path) {
    let parts: Vec<&str> = lib.name.split(':').collect();
    if parts.len() != 3 {
        eprintln!("Skipping unrecognized liteloader library: {}", lib.name);
        return;
    }

    let (group, artifact, version) = (parts[0], parts[1], parts[2]);
    let group_path = group.replace('.', "/");
    let dest = libs_dir
        .join(&group_path)
        .join(artifact)
        .join(version)
        .join(format!("{}-{}.jar", artifact, version));

    if dest.exists() {
        return;
    }
    let _ = fs::create_dir_all(dest.parent().unwrap());

    let mut bases: Vec<String> = Vec::new();
    if let Some(url) = &lib.url {
        bases.push(url.trim_end_matches('/').to_string());
    }
    if group == "net.minecraft" && artifact == "launchwrapper" {
        bases.push(MOJANG_LIBS.to_string());
    }
    bases.push(REPO_LITELOADER.to_string());
    bases.push(MAVEN_CENTRAL.to_string());
    bases.push(repo_url.trim_end_matches('/').to_string());

    let mut seen: Vec<String> = Vec::new();
    bases.retain(|b| {
        if seen.contains(b) {
            false
        } else {
            seen.push(b.clone());
            true
        }
    });

    for base in bases {
        let url = if version.ends_with("-SNAPSHOT") {
            match resolve_snapshot_url(mp, &base, &group_path, artifact, version, None).await {
                Some(url) => url,
                None => continue,
            }
        } else {
            format!("{}/{}/{}/{}/{}-{}.jar", base, group_path, artifact, version, artifact, version)
        };

        if !url_ok(&url).await {
            continue;
        }

        match util::download_async(mp, &url, &dest, format!("Downloaded {}", lib.name)).await {
            Ok(bytes) if !bytes.is_empty() => {
                println!("Downloaded liteloader library {}", lib.name);
                return;
            }
            _ => {
                let _ = fs::remove_file(&dest);
            }
        }
    }

    eprintln!("Failed to download liteloader library {}", lib.name);
}

pub async fn handle(mp: &MultiProgress, opt_version: Option<String>, _opt_loader_version: Option<String>, limit: String, username: String, javaagent: bool) {
    let versions_json_text = util::download_text_no_save_async(mp, LITELOADER_VERSIONS_JSON, "Downloaded liteloader versions json".to_owned()).await.expect("Failed to download liteloader versions json");
    let versions_json: LiteLoaderVersions = serde_json::from_str(&versions_json_text).expect("Failed to parse liteloader versions");
    let versions = versions_json.versions;

    let version = match opt_version {
        Some(v) => {
            if !versions.contains_key(&v) {
                eprintln!("The version you provided doesn't exist");
                std::process::exit(-1);
            }
            v
        }
        None => {
            let mut best: Option<String> = None;
            let mut best_parts: Vec<u64> = Vec::new();
            for key in versions.keys() {
                let parts: Vec<u64> = key.split('.').filter_map(|p| p.parse().ok()).collect();
                if parts > best_parts {
                    best_parts = parts;
                    best = Some(key.clone());
                }
            }
            best.unwrap()
        }
    };

    println!("Launching {}...", version);

    let proj_dirs = ProjectDirs::from("me", "illia", "mc_cli").unwrap();
    let data_dir = proj_dirs.data_dir();
    let vers = data_dir.join("vers");
    let ver_path = vers.join(format!("liteloader-{}", version));
    create_dirs(vers, ver_path.clone());

    let entry = versions.get(&version).unwrap();
    let repo_url = entry.repo.url.clone();
    let libs_dir = ver_path.join("libs");

    let mut required_libs: Vec<LiteLoaderLibrary> = Vec::new();
    {
        let mut push = |lib: &LiteLoaderLibrary| {
            if !required_libs.iter().any(|l| l.name == lib.name) {
                required_libs.push(lib.clone());
            }
        };

        if let Some(art) = &entry.artefacts {
            if let Some(tweaks) = art.liteloader.get("latest").or_else(|| art.liteloader.values().next()) {
                for lib in &tweaks.libraries {
                    push(lib);
                }
            }
        }

        if let Some(snap) = &entry.snapshots {
            for lib in &snap.libraries {
                push(lib);
            }
            if let Some(tweaks) = snap.liteloader.get("latest").or_else(|| snap.liteloader.values().next()) {
                for lib in &tweaks.libraries {
                    push(lib);
                }
            }
        }
    }

    for lib in &required_libs {
        download_library(mp, lib, &repo_url, &libs_dir).await;
    }

    let ll_dir = libs_dir.join("com").join("mumfrey").join("liteloader");
    let _ = fs::create_dir_all(&ll_dir);

    if let Some(art) = &entry.artefacts {
        if let Some(tweaks) = art.liteloader.get("latest").or_else(|| art.liteloader.values().next()) {
            let url = format!("{}com/mumfrey/liteloader/{}/{}", repo_url, version, tweaks.file);
            let dest = ll_dir.join(&tweaks.file);
            let _ = util::download_async(mp, &url, &dest, "Downloaded liteloader jar".to_string()).await.expect("Failed to download liteloader jar");
        }
    } else if let Some(snap) = &entry.snapshots {
        if let Some(tweaks) = snap.liteloader.get("latest").or_else(|| snap.liteloader.values().next()) {
			println!("{:#?}", tweaks.version);
            let url = resolve_snapshot_url(mp, &repo_url, "com/mumfrey", "liteloader", &tweaks.version, Some("release")).await.expect("Failed to resolve liteloader snapshot jar");
            let filename = url.rsplit('/').next().unwrap();
            let dest = ll_dir.join(filename);
            let _ = util::download_async(mp, &url, &dest, "Downloaded liteloader jar".to_string()).await.expect("Failed to download liteloader jar");
        }
    }

    vanilla::handle(mp, Some(version), limit.clone(), true, Some(ver_path.as_path()), username, javaagent, true).await;
}

pub fn create_dirs(vers: PathBuf, ver: PathBuf) {
    let _ = fs::create_dir_all(vers.clone());
    let _ = fs::create_dir(ver.clone());
    let _ = fs::create_dir(vers.parent().unwrap().join("game"));
    let _ = fs::create_dir(ver.join("libs"));
    let _ = fs::create_dir(vers.parent().unwrap().join("assets"));
}

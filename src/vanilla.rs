use std::{collections::HashMap, fs, io::{BufRead, BufReader}, path::{Path, PathBuf}, process::{Command, Stdio}, sync::Arc};
use directories::ProjectDirs;
use indicatif::MultiProgress;
use jars::JarOptionBuilder;
use serde::Deserialize;
use tokio::sync::Semaphore;
use uuid::Uuid;

use crate::{assets::AssetIndexJson, mem, rules, util, version::{self, VersionJson}};

const VANILLA_MANIFEST: &'static str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
const MAX_CONCURRENT_DOWNLOADS: i32 = 8;

#[derive(Deserialize, Debug)]
pub struct VanillaVersion {
    pub id: String,
    pub url: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct VanillaLatest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Deserialize, Debug)]
pub struct VanillaManifest {
    pub latest: VanillaLatest,
    pub versions: Vec<VanillaVersion>,
}

pub fn get_ver_json_url(manifest: VanillaManifest, version: String) -> String {
    let mut found = false;
    let mut url = "".to_owned();
    manifest.versions.iter().any(|item| {
        found = item.id.trim() == version.trim();
        if found {
            url = item.url.clone();
        }
        found
    });
    if !found {
        eprintln!("FATAL: The specified version does not exist.");
        std::process::exit(-1);
    };
    url
}

pub async fn get_manifest(mp: &MultiProgress) -> VanillaManifest {
    let manifest_txt = util::download_text_no_save_async(mp, VANILLA_MANIFEST, "Downloaded vanilla manifest".to_owned()).await.expect("Failed to download vanilla manifest to RAM");
    serde_json::from_str(&manifest_txt).unwrap()
}

pub fn launch(json: VersionJson, version_dir: PathBuf, limit: String, username: String, javaagent: bool, liteloader: bool) {
    let game_dir = version_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("game");
    let assets_dir = version_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("assets");
    let libs = version_dir.join("libs");

    let mut classpath_paths = util::list_files_recursively(&libs);
    classpath_paths.retain(|p| {
        matches!(p.extension().and_then(|e| e.to_str()), Some("jar" | "zip"))
    });
    classpath_paths.push(version_dir.join("client.jar"));
    let classpath = if std::env::consts::OS == "windows" {
        classpath_paths
            .iter()
            .map(|e| e.to_string_lossy())
            .collect::<Vec<_>>()
            .join(";")
    } else {
        classpath_paths
            .iter()
            .map(|e| e.to_string_lossy())
            .collect::<Vec<_>>()
            .join(":")
    };

    let mut jvm_args: Vec<String> = vec![format!("-Xmx{}", limit)];

	if javaagent {
		let dir = std::fs::read_dir(game_dir.join("mods"));

		if let Ok(dir) = dir {
			for f in dir {
				if let Ok(f) = f {
					if f.path().extension().is_some_and(|ext| ext == "jar") {
						let agent = format!("-javaagent:{}", f.path().display());

						jvm_args.push(agent);
					}
				}
			}
		} else if let Err(e) = dir {
			eprintln!("there was an error while reading mods from mods dir: {e}");
		}
	}

    if let Some(arguments) = json.arguments.clone() {
        for arg in arguments.jvm {
            match arg {
                version::JvmArgument::String(arg) => jvm_args.push(arg),
                version::JvmArgument::ArgWithRule { rules, value } => {
                    for rule in &rules {
                        if rules::matches_os_rule(rule) {
                            match value {
                                version::JvmArgumentValue::String(ref val) => {
                                    jvm_args.push(val.clone());
                                }
                                version::JvmArgumentValue::Strings(ref vals) => {
                                    jvm_args.extend(vals.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        jvm_args.extend(vec![
            "-Djava.library.path=${natives_directory}".to_owned(),
            "-Djna.tmpdir=${natives_directory}".to_owned(),
            "-Dorg.lwjgl.system.SharedLibraryExtractPath=${natives_directory}".to_owned(),
            "-Dio.netty.native.workdir=${natives_directory}".to_owned(),
            "-Dminecraft.launcher.brand=${launcher_name}".to_owned(),
            "-Dminecraft.launcher.version=${launcher_version}".to_owned(),
            "-cp".to_owned(),
            "${classpath}".to_owned(),
        ]);
    }

    let jvm_args_resolved: Vec<String> = jvm_args
        .into_iter()
        .map(|arg| {
            arg.replace("${natives_directory}", &libs.to_string_lossy())
                .replace("${classpath}", &classpath)
                .replace("${launcher_name}", "mc_cli")
                .replace("${launcher_version}", env!("CARGO_PKG_VERSION"))
        })
        .collect::<Vec<_>>();

    let mut game_args: Vec<String> = vec![];
    let mut features: HashMap<String, bool> = HashMap::new();

    features.insert("is_demo_user".to_owned(), false);
    features.insert("has_custom_resolution".to_owned(), false);
    features.insert("has_quick_plays_support".to_owned(), false);
    features.insert("is_quick_play_singleplayer".to_owned(), false);
    features.insert("is_quick_play_multiplayer".to_owned(), false);
    features.insert("is_quick_play_realms".to_owned(), false);

    if let Some(arguments) = json.arguments.clone() {
        for arg in arguments.game {
            match arg {
                version::GameArgument::String(arg) => game_args.push(arg),
                version::GameArgument::ArgWithRule { rules, value } => {
                    for rule in &rules {
                        if rules::matches_arg_rule(features.clone(), rule) {
                            match value {
                                version::GameArgumentValue::String(ref val) => {
                                    game_args.push(val.clone());
                                }
                                version::GameArgumentValue::Strings(ref vals) => {
                                    game_args.extend(vals.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    } else if let Some(minecraft_arguments) = json.minecraftArguments.clone() {
        let args = minecraft_arguments.split(' ').map(|s| s.to_owned()).collect::<Vec<_>>();
        game_args.extend(args);
    }

	if liteloader {
		game_args.push("--tweakClass".to_string());
		game_args.push("com.mumfrey.liteloader.launch.LiteLoaderTweaker".to_string());
	}

    let game_args_resolved: Vec<String> = game_args
        .into_iter()
        .map(|arg| {
            arg.replace("${auth_player_name}", &username)
                .replace("${player_name}", &username)
                .replace(
                    "${version_name}",
                    version_dir.file_name().unwrap().to_str().unwrap(),
                )
                .replace("${game_directory}", game_dir.to_str().unwrap())
                .replace("${auth_uuid}", &Uuid::new_v4().to_string())
                .replace("${auth_access_token}", "")
                .replace("${clientid}", &Uuid::new_v4().to_string())
                .replace("${auth_xuid}", "0")
                .replace("${user_type}", "legacy")
                .replace("${version_type}", &json.r#type)
                .replace("${user_properties}", "{}")
                .replace(
                    "${assets_index_name}",
                    &version_dir.file_name().unwrap().to_string_lossy(),
                )
                .replace("${assets_root}", assets_dir.to_str().unwrap())
                .replace("${game_assets}", assets_dir.to_str().unwrap())
        })
        .collect::<Vec<_>>();

    let mut cmd: Vec<String> = vec![];
    cmd.extend(jvm_args_resolved);
    if liteloader {
        cmd.push("net.minecraft.launchwrapper.Launch".to_owned());
    } else {
        cmd.push(json.mainClass);
    }
    cmd.extend(game_args_resolved);

    println!("cmd: {:?}", cmd);


    let mut process = Command::new("java")
        .current_dir(game_dir)
        .args(&cmd)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to run Minecraft");

    let stdout = process.stdout.take().expect("Failed to take stdout");
    let reader = BufReader::new(stdout);
    for line in reader.lines() {
        let line = line.expect("Failed to read stdout line");
        println!("{}", line);
    }

    let status = process.wait().expect("Failed to wait for child");
    println!("Exited with {}", status);
}

pub fn create_dirs(vers: PathBuf, ver: PathBuf) {
    let _ = fs::create_dir_all(vers.clone());
    let _ = fs::create_dir(ver.clone());
    let _ = fs::create_dir(vers.parent().unwrap().join("game"));
    let _ = fs::create_dir(vers.parent().unwrap().join("game").join("mods"));
    let _ = fs::create_dir(ver.join("libs"));
    let _ = fs::create_dir(vers.parent().unwrap().join("assets"));
}

pub async fn handle(mp: &MultiProgress, opt_version: Option<String>, limit: String, b_launch: bool, version_dir: Option<&Path>, username: String, javaagent: bool, liteloader: bool) {
    mem::check_if_valid(limit.clone());

    let manifest = get_manifest(mp).await;
    let version = opt_version.unwrap_or(manifest.latest.snapshot.clone());

    let proj_dirs = ProjectDirs::from("me", "illia", "mc_cli").unwrap();
    let data_dir = proj_dirs.data_dir();
    let vers = data_dir.join("vers");
    let binding = vers.join(version.as_str());
    let ver = version_dir.unwrap_or(&binding);
    let libs = ver.join("libs");

    'launch_logic: {
        if ver.is_dir() {
            println!("Launching vanilla {} with memory limit {}", version, limit);

            let text = fs::read_to_string(ver.join("version.json"));
            if !text.is_ok() {
                break 'launch_logic;
            }
            let text = text.unwrap();
            let mut version_json_err = serde_json::Deserializer::from_str(&text);
            let version_json_res = serde_path_to_error::deserialize::<_, VersionJson>(&mut version_json_err);
            let version_json = match version_json_res {
                Ok(val) => val,
                Err(err) => panic!("err: {:#?}", err),
            };

            if b_launch {
                launch(version_json, ver.to_path_buf(), limit.clone(), username, javaagent, liteloader);
            }

            return;
        }
    }

    create_dirs(vers, ver.to_path_buf());

    let ver_url = get_ver_json_url(manifest, version.clone());

    let text = util::download_text_async(mp, ver_url.as_str(), ver.join("version.json").as_path(), "Downloaded version.json".to_owned()).await.expect("Failed to download version json");

    let mut version_json_err = serde_json::Deserializer::from_str(text.as_str());
    let version_json_res = serde_path_to_error::deserialize::<_, VersionJson>(&mut version_json_err);
    let version_json = match version_json_res {
        Ok(val) => val,
        Err(err) => panic!("err: {:#?}", err),
    };

    // download minecraft jar
    let client_url = version_json.downloads.client.url.clone();
    let _ = util::download_async(mp, client_url.as_str(), ver.join("client.jar").as_path(), "Downloaded client jar".to_owned()).await.expect("Failed to download client jar");
    let lib_semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_DOWNLOADS as usize));

    let mut download_tasks = Vec::new();

    for lib in &version_json.libraries {
        let should_download = if let Some(rules) = &lib.rules {
            rules.iter().any(rules::matches_os_rule)
        } else {
            true
        };

        if should_download && lib.downloads.artifact.is_some() {
            let artifact = lib.downloads.artifact.as_ref().unwrap();
            let path = Path::new(&artifact.path);
            let dir_path = libs.join(path.parent().unwrap());
            let download_path = libs.join(path);
            let url = artifact.url.clone();
            let sem = Arc::clone(&lib_semaphore);
            let mp_clone = mp.clone();

            tokio::fs::create_dir_all(&dir_path).await.unwrap_or_default();

            download_tasks.push(tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();

                util::download_async(
                    &mp_clone,
                    &url, 
                    &download_path, 
                    "Downloaded lib".to_owned()
                ).await.expect("Failed to download library")
            }));
        }

        if let Some(classifiers) = &lib.downloads.classifiers {
            let needed = rules::classifiers_needed(classifiers);

            for needed_classifier in needed {
                let path = Path::new(&needed_classifier.path);
                let dir_path = libs.join(path.parent().unwrap());
                let download_path = libs.join(path);
                let url = needed_classifier.url.clone();
                let libs_clone = libs.to_path_buf();
                let extract = lib.extract.clone();
                let mp_clone = mp.clone();

                tokio::fs::create_dir_all(&dir_path).await.unwrap_or_default();

                download_tasks.push(tokio::spawn(async move {
                    let classifier_lib = util::download_async(
                        &mp_clone,
                        &url,
                        &download_path,
                        "Downloaded classifier lib".to_owned()
                    ).await.expect("Failed to download classifier lib");

                    if let Some(extract_info) = extract {
                        let option = JarOptionBuilder::builder().target(libs_clone.to_str().unwrap()).build();
                        println!("lib: {:#?}", download_path);
                        let jar = jars::jar(download_path, option).expect("Failed to extract library jar file");

                        for (file_path, content) in jar.files {
                            let dir = if Path::new(&file_path).is_dir() {
                                Path::new(&file_path)
                            } else {
                                Path::new(&file_path).parent().unwrap()
                            };

                            tokio::fs::create_dir_all(libs_clone.join(dir)).await
                                .expect("Failed to create directory for extracted file");

                            tokio::fs::write(libs_clone.join(&file_path), content).await
                                .expect(&format!("Failed to write extracted file {}", file_path));
                            }

                        for excluded in extract_info.exclude {
                            let _ = tokio::fs::remove_file(excluded).await;
                        }
                    }
                    classifier_lib
                }));
            }
        }
    }

    // Wait for all downloads to complete
    futures_util::future::join_all(download_tasks).await;

    let assets_dir = data_dir.join("assets");
    let _ = fs::create_dir(assets_dir.join("indexes"));

    let asset_index_url = version_json.assetIndex.url.clone();
    let asset_index = util::download_text_async(mp, &asset_index_url, &assets_dir.join("indexes").join(format!("{}.json", version)), "Downloaded asset index".to_owned()).await.expect("Failed to download asset index json");

    let asset_index_json: AssetIndexJson = serde_json::from_str(&asset_index).expect("Failed to parse asset index json");
    let assets = asset_index_json.objects;

    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_DOWNLOADS as usize));

    let download_futures = assets.iter().map(|asset| {
        let hash = asset.1.hash.clone();
        let assets_dir = assets_dir.clone();
        let sem = Arc::clone(&semaphore);
        let mp_clone = mp.clone();

        async move {
            let _permit = sem.acquire().await.unwrap();

            let dir = &hash[..2];
            let dir_full = assets_dir.join("objects").join(dir);
            let _ = fs::create_dir_all(&dir_full);

            let url = format!("https://resources.download.minecraft.net/{}/{}", dir, hash);
            let destination = dir_full.join(&hash);

            match tokio::time::timeout(
                tokio::time::Duration::from_secs(60),
                util::download_async(&mp_clone, &url, &destination, "Done".to_owned())
            ).await {
                Ok(Ok(_)) => {
                    Ok(hash)
                },
                Ok(Err(e)) => {
                    eprintln!("Failed to download resource {}: {}", hash, e);
                    Err(format!("Download error: {}", e))
                },
                Err(_) => {
                    eprintln!("Download timed out for resource {}", hash);
                    Err(format!("Download timeout: {}", hash))
                }
            }
        }
    }).collect::<Vec<_>>();

    futures_util::future::join_all(download_futures).await;

    if b_launch {
        launch(version_json, ver.to_path_buf(), limit.clone(), username, javaagent, liteloader);
    }
}

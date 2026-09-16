use std::fs;
use std::io::{BufRead as _, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use indicatif::MultiProgress;
use uuid::Uuid;
use crate::loaders::{BABRIC, FABRIC, LABRIC, ORNITHE, QUILT};

use crate::util::LauncherDirs;
use crate::{mem, util, vanilla, version};
use crate::version::{FabricIntermediaryVersion, FabricLoaderJSON, FabricLoaderVersion, FabricVersion, FabricBase};

pub fn get_ver(versions: Vec<FabricVersion>, version: String) -> FabricVersion {
    let mut found = false;
    let mut ver = &FabricVersion { version: "".to_string() };

    versions.iter().any(|_ver| {
        found = _ver.version == version.clone();

        ver = _ver;

        found
    });
    ver.clone()
}

pub async fn down_intermediary(mp: &MultiProgress, loader: &FabricLoaderVersion, version: &FabricVersion, ver: PathBuf, use_quilt: FabricBase) {
    let is_quilt = matches!(use_quilt, FabricBase::Quilt(_));
    let use_release = match use_quilt {
        FabricBase::Quilt(value) => if value { "repository/release/" } else { "repository/snapshot/" },
        FabricBase::Fabric | FabricBase::Labric => "",
		FabricBase::Babric => "releases/",
		FabricBase::Ornithe => "releases/"
    };

	let loader = match use_quilt {
		FabricBase::Quilt(_) => QUILT,
		FabricBase::Fabric => FABRIC,
		FabricBase::Labric => LABRIC,
		FabricBase::Babric => BABRIC,
		FabricBase::Ornithe => ORNITHE,
	};

    let intermediary_versions_text = util::download_text_no_save_async(mp, loader.intermediary_versions, "Downloaded intermediary version JSON".to_string()).await.expect("Failed to download intermediary version JSON");
    let intermediaries: Vec<FabricIntermediaryVersion> = serde_json::from_str(intermediary_versions_text.as_str()).expect("Failed to deserialize intermediary version JSON");
    let mut intermediary: &FabricIntermediaryVersion = &FabricIntermediaryVersion {
        maven: "".to_string(),
        version: "".to_string(),
    };

    intermediaries.iter().any(|i| {
        if i.version == version.version {
            intermediary = i;
            return true;
        }
        return false;
    });

    let maven_path = version::maven_to_path(intermediary.maven.clone());
    let maven_path_with_domain = format!("{}{}{}", loader.maven.to_string(), use_release, maven_path);
    dbg!(&maven_path_with_domain);
    let _ = util::download_async(mp, maven_path_with_domain.as_str(), ver.join("inter.jar").as_ref(), "Downloaded intermediary...".to_string()).await.expect("Failed to download intermediary");
}

/// returns the full fabric loader JSON
/// {loader} the loader version
/// {version} the minecarft version
/// {ver} the version dir
pub async fn down(mp: &MultiProgress, loader: &FabricLoaderVersion, version: &FabricVersion, ver: PathBuf, use_quilt: FabricBase) -> FabricLoaderJSON {
    let is_quilt = matches!(use_quilt, FabricBase::Quilt(_));
    let use_release = match use_quilt {
        FabricBase::Quilt(value) => if value { "repository/release/" } else { "repository/snapshot/" },
        FabricBase::Fabric | FabricBase::Labric | FabricBase::Babric | FabricBase::Ornithe => "",
    };

	let loader_consts = match use_quilt {
		FabricBase::Quilt(_) => QUILT,
		FabricBase::Fabric => FABRIC,
		FabricBase::Labric => LABRIC,
		FabricBase::Babric => BABRIC,
		FabricBase::Ornithe => ORNITHE,
	};

    down_intermediary(mp, loader, version, ver.clone(), use_quilt).await;
    let loader_jar_url = format!("{}{}{}", if loader_consts.id == LABRIC.id || loader_consts.id == BABRIC.id || loader_consts.id == ORNITHE.id { FABRIC.maven } else { loader_consts.maven }, use_release, loader.jar_path(loader_consts));

    let jar_path = ver.join("fabric.jar");

    if !jar_path.exists() {
		println!("{:#?}", loader_jar_url);
        let _ = util::download_async(mp, &loader_jar_url, jar_path.as_path(), "Downloaded fabric loader jar".to_string()).await.expect("Failed to download fabric JAR");
    }

    let loader_json_url = format!("{}{}{}", 
		if loader_consts.id == LABRIC.id ||
		   loader_consts.id == BABRIC.id ||
		   loader_consts.id == ORNITHE.id {
			FABRIC.maven 
		} else { 
			loader_consts.maven 
		}, 
		use_release,
		loader.json_path(loader_consts)
	);
	dbg!(&loader_json_url);


    let json_path = ver.join("fabric.json");

    let loader_json = if !json_path.exists() {
        util::download_text_async(mp, &loader_json_url, json_path.as_path(), "Downloaded fabric loader JSON".to_string()).await.expect("Failed to download fabric loader JSON")
    } else {
        fs::read_to_string(json_path.as_path()).unwrap()
    };

    println!("{:#?}", loader_json);

    let parsed_json: FabricLoaderJSON = serde_json::from_str(&loader_json).expect("Failed to parse loader JSON");

    for lib in &parsed_json.libraries.common {
        let path_from_maven = version::maven_to_path(lib.name.clone());
        let path = format!("{}{}", lib.url, path_from_maven);
        let lib_path = ver.join("libs").join(path_from_maven.clone());
        let _ = fs::create_dir_all(lib_path.clone().parent().unwrap());
        let _ = util::download_async(mp, path.as_str(), &lib_path, "Downloaded common lib jar".to_owned()).await.expect("Failed to download server lib jar");
    }

    println!("Downloaded common libs...");

    for lib in &parsed_json.libraries.server {
        let path_from_maven = version::maven_to_path(lib.name.clone());
        let path = format!("{}{}", lib.url, path_from_maven);
        let lib_path = ver.join("libs").join(path_from_maven.clone());
        let _ = fs::create_dir_all(lib_path.clone().parent().unwrap());
        let _ = util::download_async(mp, path.as_str(), &lib_path, "Downloaded server lib jar".to_owned()).await.expect("Failed ot download server lib jar");
    }

    println!("Downloaded server libs...");

    for lib in &parsed_json.libraries.client {
        let path_from_maven = version::maven_to_path(lib.name.clone());
        let path = format!("{}{}", lib.url, path_from_maven);
        let lib_path = ver.join("libs").join(path_from_maven.clone());
        let _ = fs::create_dir_all(lib_path.clone().parent().unwrap());
        let _ = util::download_async(mp, path.as_str(), &lib_path, "Downloaded client lib jar".to_owned()).await.expect("Failed to download client lib jar");
    }
    println!("Downloaded client libs...");

    parsed_json
}

pub fn create_dirs(vers: PathBuf, ver: PathBuf) {
    let _ = fs::create_dir_all(vers.clone());
    let _ = fs::create_dir(ver.clone());
    let _ = fs::create_dir(vers.parent().unwrap().join("game"));
    let _ = fs::create_dir(ver.join("libs"));
    let _ = fs::create_dir(vers.parent().unwrap().join("assets"));
}

pub fn launch(ver_dir: PathBuf, main_class: String, username: String) {
    println!("Launching minecraft client...");
    let main_class = dbg!(main_class);
    let game_dir = ver_dir.join("game");
    let _ = fs::create_dir_all(&game_dir);

    let mut classpath: String = "".to_owned();
    let libs = util::list_files_recursively(&ver_dir.join("libs"));
    let libs = libs.iter().filter(|p| {
        matches!(p.extension().and_then(|e| e.to_str()), Some("jar" | "zip"))
    });
    let sep = if cfg!(target_os = "windows") { ';' } else { ':' };

    for lib in libs {
        classpath.push_str(lib.to_str().unwrap());
        classpath.push(sep);
    }

    classpath.push_str(ver_dir.join("inter.jar").to_str().unwrap());
    classpath.push(sep);
    classpath.push_str(ver_dir.join("client.jar").to_str().unwrap());
    classpath.push(sep);
    classpath.push_str(ver_dir.join("fabric.jar").to_str().unwrap());

    let mut cmd: Vec<String> = vec![];
    if cfg!(target_os = "macos") {
        cmd.push("-XstartOnFirstThread".to_owned());
    }
    cmd.push("-cp".to_owned());
    cmd.push(classpath);
    cmd.push(main_class);
    cmd.push("--gameDir".to_string());
    let game_dir = dbg!(ver_dir.parent().unwrap().parent().unwrap().join("game").to_str().unwrap().to_owned());
    cmd.push(game_dir.clone());

    cmd.push("--assetsDir".to_string());
    let assets_dir = dbg!(ver_dir.parent().unwrap().parent().unwrap().join("assets").to_str().unwrap().to_string());
    cmd.push(assets_dir);
    cmd.push("--assetIndex".to_string());
    let ver = ver_dir.file_name().unwrap().to_str().unwrap().replace("fabric-", "").replace("quilt-", "");
    cmd.push(ver);
    cmd.push("--uuid".to_string());
    cmd.push(Uuid::new_v4().to_string());

    cmd.push("--username".to_string());
    cmd.push(username);

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

pub async fn handle(dirs: LauncherDirs, mp: &MultiProgress, opt_version: Option<String>, opt_loader_version: Option<String>, limit: String, use_quilt: FabricBase, username: String, javaagent: bool) {
    let is_quilt = matches!(use_quilt, FabricBase::Quilt(_));
    let use_release = match use_quilt {
        FabricBase::Quilt(value) => if value { "repository/release/" } else { "repository/snapshot/" },
        FabricBase::Fabric | FabricBase::Labric => "",
		FabricBase::Babric | FabricBase::Ornithe => "releases/"
    };

	let loader_consts = match use_quilt {
		FabricBase::Quilt(_) => QUILT,
		FabricBase::Fabric => FABRIC,
		FabricBase::Labric => LABRIC,
		FabricBase::Babric => BABRIC,
		FabricBase::Ornithe => ORNITHE,
	};


    let game_versions = util::download_text_no_save_async(mp, loader_consts.game_versions, "Downloaded fabric game versions json".to_string()).await.expect("Failed to download fabric game versions json");
    let versions: Vec<FabricVersion> = serde_json::from_str(game_versions.as_str()).expect("Failed to parse fabric game versions JSON");
    let ver = if opt_version.is_some() {
        get_ver(versions, opt_version.unwrap())
    } else {
        versions.first().unwrap().clone()
    };

    let loader_versions_json = util::download_text_no_save_async(mp, loader_consts.loader_versions, "".to_string()).await.expect("Failed to download loader versions JSON");
    let loader_versions: Vec<FabricLoaderVersion> = serde_json::from_str(&loader_versions_json).expect("Failed to parse fabric loader versions JSON");
    let loader = if let Some(ver_str) = opt_loader_version {
        &loader_versions
            .into_iter()
            .find(|v| v.version == ver_str)
            .expect("Loader version not found")
    } else {
        loader_versions.first().expect("No loader versions found")
    };

    let (loader_version, loader_build) = (loader.version.as_str(), loader.build);

    if !mem::is_valid(limit.clone()) {
        eprintln!("Invalid memory limit");
        std::process::exit(-1);
    }

    if !mem::can_use(limit.clone()) {
        eprintln!("Your memory limit is too big");
        std::process::exit(-1);
    }

    println!("Launching {} {}-{} build {} with memory limit {} and username {}", loader_consts.id, ver.version, loader_version, loader_build, limit, username);

	let vers = dirs.vers_dir;
    let ver_path = vers.join(format!("{}-{}", loader_consts.id, ver.version.clone()));

    create_dirs(vers, ver_path.clone());

    vanilla::handle(mp, Some(ver.version.clone()), limit.clone(), false, Some(ver_path.as_path()), username.clone(), javaagent).await;
    let parsed_json = down(mp, loader, &ver, ver_path.clone(), use_quilt.clone()).await;

    let _ = fs::remove_dir_all(ver_path.join("libs").join("META-INF"));
    let _ = fs::remove_dir_all(ver_path.join("libs").join("org").join("ow2").join("asm").join("asm-all"));

    let asm = ver_path.join("libs").join("org").join("ow2").join("asm").join("asm");
    let entries = fs::read_dir(&asm).expect("Failed to read lib dir").filter_map(|e| e.ok()).filter(|e| e.file_type().map(|ft| ft.is_dir()).unwrap_or(false));


	if !matches!(use_quilt, FabricBase::Babric) {
		let mut min_dir: Option<(f64, String)> = None;
		let mut len = 0;
		for entry in entries {
			len += 1;
			let name = entry.file_name().into_string().unwrap_or_default();
			if let Ok(num) = name.parse::<f64>() {
				match min_dir {
					Some((min_val, _)) if num < min_val => {
						min_dir = Some((num, name));
					}
					None => {
						min_dir = Some((num, name));
					}
					_ => {}
				}
			}
		}

		if let Some((_, dir_name)) = min_dir {
			if len > 1 {
				let _ = fs::remove_dir_all(asm.join(Path::new(&dir_name)));
			}
		} else {
			println!("No numeric-named directories found.");
		}
	}

	launch(ver_path, parsed_json.mainClass.client, username);
}

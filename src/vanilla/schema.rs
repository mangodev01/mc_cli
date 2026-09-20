use std::{collections::HashMap, path::PathBuf};

use indicatif::MultiProgress;
use serde::Deserialize;

use crate::{app::McLoader, util};

#[derive(Deserialize, Debug, Clone)]
pub struct VanillaVersion {
    pub id: String,
    pub url: String,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct VanillaLatest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct VanillaManifest {
    pub latest: VanillaLatest,
    pub versions: Vec<VanillaVersion>,
}

#[derive(Deserialize, Debug, Clone)]
// because serde doesn't wanna rename it for me!
#[allow(non_snake_case)]
pub struct VanillaAssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: i32,
    pub totalSize: i32,
    pub url: String,
}


#[derive(Deserialize, Debug, Clone)]
// because serde doesn't wanna rename it for me!
#[allow(non_snake_case)]
pub struct VanillaVersionJson {
    pub arguments: Option<VanillaArguments>,
    pub minecraftArguments: Option<String>,
    pub downloads: VanillaDownloads,
    pub libraries: Vec<VanillaLibrary>,
    pub mainClass: String,
    pub r#type: String,
    pub assetIndex: VanillaAssetIndex,
}

#[derive(Deserialize, Debug, Clone)]
pub struct VanillaArguments {
    pub game: Vec<VanillaGameArgument>,
    pub jvm: Vec<VanillaJvmArgument>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct VanillaLibraryClassifiers {
    pub natives_windows_64: Option<VanillaLibraryDownload>,
    pub natives_windows_32: Option<VanillaLibraryDownload>,
    pub natives_windows: Option<VanillaLibraryDownload>,
    pub natives_osx: Option<VanillaLibraryDownload>,
    pub natives_osx_64: Option<VanillaLibraryDownload>,
    pub natives_osx_32: Option<VanillaLibraryDownload>,
    pub natives_linux: Option<VanillaLibraryDownload>,
    pub natives_linux_32: Option<VanillaLibraryDownload>,
    pub natives_linux_64: Option<VanillaLibraryDownload>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct VanillaLibraryDownloads {
    pub artifact: Option<VanillaLibraryDownload>,
    pub classifiers: Option<VanillaLibraryClassifiers>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct VanillaExtract {
    pub exclude: Vec<String>
}


#[derive(Deserialize, Debug, Clone)]
pub struct VanillaLibrary {
    pub downloads: VanillaLibraryDownloads,
    pub name: String,
    pub rules: Option<Vec<VanillaRule>>,
    pub extract: Option<VanillaExtract>,
}



#[derive(Deserialize, Debug, Clone)]
pub struct VanillaDownload {
    pub sha1: String,
    pub size: i32,
    pub url: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct VanillaLibraryDownload {
    pub path: String,
    pub sha1: String,
    pub size: i32,
    pub url: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct VanillaDownloads {
    pub client: VanillaDownload,
    pub client_mappings: Option<VanillaDownload>,
    pub server: Option<VanillaDownload>,
    pub server_mappings: Option<VanillaDownload>,
}


#[derive(Deserialize, Debug, Clone)]
pub struct VanillaRuleFeatures(pub HashMap<String, bool>);

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Os {
    Name { name: String },
    Arch { arch: String},
    Both { name: String, arch: String },
    // pub name: String,
}


#[derive(Deserialize, Debug, Clone)]
pub struct VanillaRule {
    pub action: String,
    pub features: Option<VanillaRuleFeatures>,
    pub os: Option<Os>
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum VanillaGameArgumentValue {
    String(String),
    Strings(Vec<String>),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum VanillaJvmArgumentValue {
    String(String),
    Strings(Vec<String>),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum VanillaJvmArgument {
    String(String),
    ArgWithRule { rules: Vec<VanillaRule>, value: VanillaJvmArgumentValue },
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum VanillaGameArgument {
    String(String),
    ArgWithRule { rules: Vec<VanillaRule>, value: VanillaGameArgumentValue },
}


#[derive(Debug, Clone)]
pub struct Vanilla {
    pub json: Option<VanillaVersionJson>,
    pub ver_dir: Option<PathBuf>
}

impl Default for Vanilla {
    fn default() -> Self {
        Self { json: None, ver_dir: None }
    }
}

pub fn matches_os_rule(rule: &VanillaRule) -> bool {
    let mut matches_rule: bool = false;
    let os = &rule.os;
    if os.is_none() {
        return true;
    }
    match os.clone().unwrap().clone() {
        Os::Name { name } => {
            let os = rust_os_to_minecraft_os();
            if name == os {
                matches_rule = true;
            }
        },
        Os::Arch { arch } => {
            let cur_arch = rust_arch_to_minecraft_arch();
            if arch == cur_arch {
                matches_rule = true;
            }
        },
        Os::Both { name, arch } => {
            let cur_arch = rust_arch_to_minecraft_arch();
            if arch == cur_arch {
                matches_rule = true;
            }
            let os = rust_os_to_minecraft_os();
            if name == os && matches_rule {
                matches_rule = true;
            }
        },
    }
    return matches_rule && rule.action == "allow";
}

pub fn matches_arg_rule(features: HashMap<String, bool>, rule: &VanillaRule) -> bool {
    if rule.features.is_none() {
        return true;
    }
    let matches_rule: bool = hashmap_contains::<String, bool>(&features, &rule.features.clone().unwrap().0);
    return matches_rule && rule.action == "allow";
}

pub fn classifiers_needed(classifiers: &VanillaLibraryClassifiers) -> Vec<&VanillaLibraryDownload> {
    let mut downloads = vec![];

    let arch = std::env::consts::ARCH;

    match std::env::consts::OS {
        "macos" => {
            if let Some(download) = &classifiers.natives_osx {
                downloads.push(download);
            }
            if let Some(download) = &classifiers.natives_osx_64 {
                downloads.push(download);
            }
            if let Some(download) = &classifiers.natives_osx_32 {
                downloads.push(download);
            }
        }
        "windows" => {
            if arch == "x86_64" {
                if let Some(download) = &classifiers.natives_windows_64 {
                    downloads.push(download);
                }
            } else if arch == "x86" {
                if let Some(download) = &classifiers.natives_windows_32 {
                    downloads.push(download);
                }
            }

            if let Some(download) = &classifiers.natives_windows {
                downloads.push(download);
            }
        }
        "linux" => {
            if arch == "x86_64" {
                if let Some(download) = &classifiers.natives_linux_64 {
                    downloads.push(download);
                }
            } else if arch == "x86" {
                if let Some(download) = &classifiers.natives_linux_32 {
                    downloads.push(download);
                }
            }

            if let Some(download) = &classifiers.natives_linux {
                downloads.push(download);
            }
        }
        _ => {}
    }

    downloads
}

pub fn hashmap_contains<K: Eq + std::hash::Hash, V: PartialEq>(
    big: &std::collections::HashMap<K, V>,
    small: &std::collections::HashMap<K, V>,
) -> bool {
    small.iter().all(|(k, v)| big.get(k) == Some(v))
}

pub fn rust_os_to_minecraft_os() -> &'static str {
    match std::env::consts::OS {
        "windows" => "windows",
        "macos" => "osx",
        "linux" => "linux",
        _ => "unknown", // fallback for weird platforms
    }
}
pub fn rust_arch_to_minecraft_arch() -> &'static str {
    match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "x86" => "x86",
        "aarch64" => "aarch64",
        "arm" => "arm32",
        _ => "unknown",
    }
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
    let manifest_txt = util::download_text_no_save_async(
		mp,
		&Vanilla::game_versions(),
		"Downloaded vanilla manifest".to_owned()
	).await.expect("Failed to download vanilla manifest to RAM");

    serde_json::from_str(&manifest_txt).unwrap()
}


use std::path::PathBuf;

use serde::Deserialize;

pub struct Fabric {
	pub ver_dir: Option<PathBuf>,
	pub main_class: String
}

impl Default for Fabric {
    fn default() -> Self {
        Self { ver_dir: None, main_class: "".to_string() }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct FabricVersion {
    pub version: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FabricLoaderVersion {
    pub separator: String,
    pub build: i32,
    pub maven: String,
    pub version: String,
}

impl FabricLoaderVersion {
    pub fn split(&self) -> (&str, &str) {
        let arr = self.version.split(&self.separator).collect::<Vec<&str>>();
        if arr.len() == 2 {
            (arr[0], arr[1])
        } else {
            (self.version.as_str(), "")
        }
    }
    pub fn replace(&self) -> String {
        format!("{}{}", self.maven.replace(&self.version, "").replace(self.separator.as_str(), "/").replace(".", "/").replace(":", "/"), self.version)
    }
    pub fn jar_path(&self, loader: String) -> String {
        if &loader == "labric" || &loader == "babric" || &loader == "ornithe" {
            // use same loader json for legacy fabric
            format!("{}/fabric-loader-{}.jar", self.replace(), self.version)
        } else {
            format!("{}/{}-loader-{}.jar", self.replace(), loader, self.version)
        }
    }
    pub fn json_path(&self, loader: String) -> String {
        if &loader == "labric" || &loader == "babric" || &loader == "ornithe" {
            // use same loader json for legacy fabric
            format!("{}/fabric-loader-{}.json", self.replace(), self.version)
		} else {
            format!("{}/{}-loader-{}.json", self.replace(), loader, self.version)
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct FabricLib {
    pub name: String,
    pub url: String,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct FabricLoaderJSON {
    pub version: i32,
    pub min_java_version: i32,
    pub libraries: FabricLibraries,
    pub mainClass: FabricMainClass,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FabricLibraries {
    pub client: Vec<FabricLib>,
    pub common: Vec<FabricLib>,
    pub server: Vec<FabricLib>,
    pub development: Vec<FabricLib>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FabricMainClass {
    pub client: String,
    pub server: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FabricIntermediaryVersion {
    pub maven: String,
    pub version: String,
}



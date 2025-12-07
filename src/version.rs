use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Arguments {
    pub game: Vec<GameArgument>,
    pub jvm: Vec<JvmArgument>,
}

#[derive(Deserialize, Debug)]
pub struct Download {
    pub sha1: String,
    pub size: i32,
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct LibraryDownload {
    pub path: String,
    pub sha1: String,
    pub size: i32,
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct Downloads {
    pub client: Download,
    pub client_mappings: Option<Download>,
    pub server: Option<Download>,
    pub server_mappings: Option<Download>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct LibraryClassifiers {
    pub natives_windows_64: Option<LibraryDownload>,
    pub natives_windows_32: Option<LibraryDownload>,
    pub natives_windows: Option<LibraryDownload>,
    pub natives_osx: Option<LibraryDownload>,
    pub natives_osx_64: Option<LibraryDownload>,
    pub natives_osx_32: Option<LibraryDownload>,
    pub natives_linux: Option<LibraryDownload>,
    pub natives_linux_32: Option<LibraryDownload>,
    pub natives_linux_64: Option<LibraryDownload>,
}

#[derive(Deserialize, Debug)]
pub struct LibraryDownloads {
    pub artifact: Option<LibraryDownload>,
    pub classifiers: Option<LibraryClassifiers>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Os {
    Name { name: String },
    Arch { arch: String},
    Both { name: String, arch: String },
    // pub name: String,
}

#[derive(Deserialize, Debug)]
// because serde doesn't wanna rename it for me!
#[allow(non_snake_case)]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: i32,
    pub totalSize: i32,
    pub url: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Extract {
    pub exclude: Vec<String>
}

#[derive(Deserialize, Debug)]
pub struct Library {
    pub downloads: LibraryDownloads,
    pub name: String,
    pub rules: Option<Vec<Rule>>,
    pub extract: Option<Extract>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FabricLib {
    pub name: String,
    pub url: String,
}

#[derive(Deserialize, Debug)]
// because serde doesn't wanna rename it for me!
#[allow(non_snake_case)]
pub struct VersionJson {
    pub arguments: Option<Arguments>,
    pub minecraftArguments: Option<String>,
    pub downloads: Downloads,
    pub libraries: Vec<Library>,
    pub mainClass: String,
    pub r#type: String,
    pub assetIndex: AssetIndex,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RuleFeatures(pub HashMap<String, bool>);

#[derive(Deserialize, Debug, Clone)]
pub struct Rule {
    pub action: String,
    pub features: Option<RuleFeatures>,
    pub os: Option<Os>
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum GameArgumentValue {
    String(String),
    Strings(Vec<String>),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum JvmArgumentValue {
    String(String),
    Strings(Vec<String>),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum JvmArgument {
    String(String),
    ArgWithRule { rules: Vec<Rule>, value: JvmArgumentValue },
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum GameArgument {
    String(String),
    ArgWithRule { rules: Vec<Rule>, value: GameArgumentValue },
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

pub fn maven_to_path(coords: String) -> String {
    let parts: Vec<&str> = coords.split(':').collect();

    if parts.len() != 3 {
        panic!("Invalid Maven coordinates, expected format 'groupId:artifactId:version'");
    }

    let group_id = parts[0];
    let artifact_id = parts[1];
    let version = parts[2];

    let group_path = group_id.replace('.', "/");

    format!("{}/{}/{}/{}-{}.jar", group_path, artifact_id, version, artifact_id, version)
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
    pub fn jar_path(&self, is_quilt: bool) -> String {
        format!("{}/{}-loader-{}.jar", self.replace(), if is_quilt { "quilt" } else { "fabric" }, self.version)
    }
    pub fn json_path(&self, is_quilt: bool) -> String {
        format!("{}/{}-loader-{}.json", self.replace(), if is_quilt { "quilt" } else { "fabric" }, self.version)
    }
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

pub enum UseQuilt {
    Yes(bool),
    No
}

#[derive(Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct LiteLoaderMeta {
    pub description: String,
    pub authors: String,
    pub url: String,
    pub updated: String,
    pub updatedTime: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LiteLoaderRepo {
    pub stream: String,
    pub r#type: String,
    pub url: String,
    pub classifier: String,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct LiteLoaderTweaks {
    pub tweakClass: String,
    pub libraries: Vec<LiteLoaderLibrary>,
    pub stream: String,
    pub file: String,
    pub version: String,
    pub build: String,
    pub md5: String,
    pub timestamp: String,
    pub lastSuccessfulBuild: i32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LiteLoaderLibrary {
    pub name: String,
    pub url: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LiteLoaderSnapshots {
    pub libraries: Vec<LiteLoaderLibrary>,
    #[serde(rename = "com.mumfrey:liteloader")]
    pub liteloader: HashMap<String, LiteLoaderTweaks>, 
}

#[derive(Deserialize, Debug, Clone)]
pub struct LiteLoaderVersion {
    pub repo: LiteLoaderRepo,
    pub snapshots: Option<LiteLoaderSnapshots>,
    pub artefacts: Option<LiteLoaderArtifact>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct LiteLoaderArtifacts {
    pub tweakClass: String,
    pub libraries: Vec<LiteLoaderLibrary>,
    pub stream: String,
    pub file: String,
    pub version: String,
    pub md5: String,
    pub timestamp: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LiteLoaderArtifact {
    #[serde(rename = "com.mumfrey:liteloader")]
    pub liteloader: HashMap<String, LiteLoaderArtifacts>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LiteLoaderVersions {
    pub meta: LiteLoaderMeta,
    pub versions: HashMap<String, LiteLoaderVersion>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct MavenMetadataVersioning {
    pub snapshot: MavenMetadataSnapshot,
    pub lastUpdated: String,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct MavenMetadataSnapshot {
    pub timestamp: String,
    pub buildNumber: i32,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct MavenMetadataRoot {
    pub groupId: String,
    pub artifactId: String,
    pub version: String,
    pub versioning: MavenMetadataVersioning,
}

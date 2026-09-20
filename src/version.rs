use std::collections::HashMap;

use serde::Deserialize;
use uuid::Uuid;

#[derive(Clone, Copy)]
pub struct FabricLike<'a> {
	pub game_versions: &'a str,
	pub loader_versions: &'a str,
	pub intermediary_versions: &'a str,
	pub maven: &'a str,
	pub id: &'a str
}


#[derive(Deserialize, Debug)]
pub struct McArchiveModVersion {
	pub files: Vec<McArchiveModFile>,
	pub uuid: Uuid,
	pub name: String,
	pub page_url: String,
	pub description: String,
}

#[derive(Deserialize, Debug)]
pub struct McArchiveModFile {
	pub uuid: Uuid,
	pub sha256: String,
	pub name: String,
	pub description: String,
	pub page_url: String,
	pub redirect_url: String,
	pub archive_url: Option<String>
}

#[derive(Deserialize, Debug)]
pub struct McArchiveMod {
	pub mod_versions: Vec<McArchiveModVersion>,
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

#[derive(Clone)]
pub enum FabricBase {
    Quilt(bool),
    Fabric,
	Labric,
	Babric,
	Ornithe
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

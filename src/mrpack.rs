use std::collections::HashMap;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MrpackEnv {
	pub client: SideType,
	pub server: SideType
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum SideType {
	#[serde(rename = "required")]
	Required,

	#[serde(rename = "optional")]
	Optional,

	#[serde(rename = "unsupported")]
	Unsupported
}

fn empty_hashmap() -> HashMap<String, String> {
	HashMap::new()
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MrpackFile {
	pub path: String,
	pub env: MrpackEnv,
	pub downloads: Vec<String>,
	pub file_size: u32,
	/// dependency 2 version
	#[serde(default = "empty_hashmap")]
	pub dependencies: HashMap<String, String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MrpackIndexJson {
	pub format_version: u8,
	pub game: String,
	pub version_id: String,
	pub name: String,
	pub summary: Option<String>,
	pub files: Vec<MrpackFile>,
}

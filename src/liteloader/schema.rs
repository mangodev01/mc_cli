use std::path::PathBuf;

pub struct Liteloader {
	pub ver_dir: Option<PathBuf>,
	pub main_class: String,
}

impl Default for Liteloader {
    fn default() -> Self {
        Self { main_class: "".to_string(), ver_dir: None }
    }
}


use std::path::PathBuf;

pub struct Quilt {
	pub snapshot: bool,
	pub ver_dir: Option<PathBuf>,
	pub main_class: String,
}

impl Default for Quilt {
    fn default() -> Self {
        Self { main_class: "".to_string(), snapshot: false, ver_dir: None }
    }
}


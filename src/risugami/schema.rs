use std::path::PathBuf;

pub struct Risugami {
	pub ver_dir: Option<PathBuf>,
	pub main_class: String,
}

impl Default for Risugami {
    fn default() -> Self {
        Self { main_class: "".to_string(), ver_dir: None }
    }
}
use std::{fs::File, io::{Cursor, Read, Write}, path::Path};

use compress_tools::Ownership;
use zip::write::SimpleFileOptions;

use crate::risugami::schema::Risugami;
use crate::version::McArchiveMod;
use crate::{app::{McCtx, McLaunchCtx, McLoader}, util, vanilla::schema::{Vanilla, VanillaVersionJson}};

pub mod schema;

const RISUGAMI_VERSIONS: &str = "https://mcarchive.net/api/v1/mods/by_slug/modloader";

fn zip_dir(src_dir: &Path, dst_file: &Path) -> std::io::Result<()> {
	let file = File::create(dst_file)?;
	let mut zip = zip::ZipWriter::new(file);
	let options = SimpleFileOptions::default()
		.compression_method(zip::CompressionMethod::Deflated)
		.unix_permissions(0o755);

	let mut buffer = Vec::new();
	let mut stack = vec![src_dir.to_path_buf()];

	while let Some(dir) = stack.pop() {
		for entry in std::fs::read_dir(&dir)? {
			let entry = entry?;
			let path = entry.path();
			let name = path.strip_prefix(src_dir).unwrap().to_str().unwrap().replace('\\', "/");

			if path.is_file() {
				zip.start_file(name, options)?;
				let mut f = File::open(&path)?;
				buffer.clear();
				f.read_to_end(&mut buffer)?;
				zip.write_all(&buffer)?;
			} else if path.is_dir() {
				zip.add_directory(format!("{}/", name), options)?;
				stack.push(path);
			}
		}
	}

	zip.finish()?;
	Ok(())
}

impl<'a> McLoader<'a> for Risugami {
	async fn install(&mut self, ctx: McCtx<'a>) {
		let versions_json_text = util::download_text_no_save_async(ctx.mp, RISUGAMI_VERSIONS, "Downloaded risugami versions json".to_owned()).await.expect("Failed to download risugami versions json");
		let versions_json: McArchiveMod = serde_json::from_str(&versions_json_text).expect("Failed to parse risugami versions");
		let versions = versions_json.mod_versions;

		let version = match ctx.opt_version {
			Some(ref v) => versions.iter().find(|x| x.name == *v),
			None => Some(&versions[0]),
		};

		let version = match version {
			Some(v) => v,
			None => {
				eprintln!("unable to find risugami version {}", ctx.opt_version.unwrap());
				std::process::exit(-1);
			}
		};

		let files = &version.files;
		let modloader_archive_url = files[0].archive_url.clone().unwrap_or_else(|| {
			format!("http://b2.mcarchive.net/file/mcarchive/{}/{}", files[0].sha256, urlencoding::encode(&files[0].name))
		});
		let modloader_archive = util::download_no_save_async(ctx.mp, &modloader_archive_url, "Downloaded risugami.zip".to_owned()).await.expect("Failed to download risugami.zip");

		let temp = std::env::temp_dir();
		let modloader_archive_extract_path = temp.join("modloader");

		let _ = std::fs::remove_dir_all(&modloader_archive_extract_path);
		let _ = std::fs::create_dir_all(&modloader_archive_extract_path);

		let mut cursor = Cursor::new(modloader_archive);
		if let Err(e) = compress_tools::uncompress_archive(&mut cursor, &modloader_archive_extract_path, Ownership::Ignore) {
			eprintln!("error while uncompressing risugami's modloader archive: {}", e);
			return;
		}

		let risugami_dir = ctx.dirs.vers_dir.join(format!("risugami-{}", version.name));

		let mut vanilla = Vanilla::default();
		let ctx = McCtx { mp: ctx.mp, opt_version: Some(version.name.clone()), limit: ctx.limit.clone(), version_dir: Some(&risugami_dir), username: ctx.username.clone(), javaagent: ctx.javaagent, dirs: ctx.dirs, opt_loader_version: None };

		vanilla.install(ctx.clone()).await;

		let jar_path = risugami_dir.join("client.jar");

		if jar_path.is_file() {
			let jar = match std::fs::read(&jar_path) {
				Ok(jar_bytes) => jar_bytes,
				Err(_) => {
					eprintln!("unable to read vanilla client.jar");
					return;
				}
			};

			let unjar_path = temp.join("cli_jar");

			let _ = std::fs::remove_dir_all(&unjar_path);
			let _ = std::fs::create_dir_all(&unjar_path);

			match compress_tools::uncompress_archive(Cursor::new(jar), &unjar_path, Ownership::Ignore) {
				Ok(_) => {},
				Err(e) => {
					eprintln!("unable to uncompress vanilla client.jar: {e}");
					return;
				},
			};

			let read_dir_res = std::fs::read_dir(&modloader_archive_extract_path);

			if let Err(e) = read_dir_res {
				eprintln!("failed to read extracted risugami's modloader files: {e}");
				return;
			}

			for entry in read_dir_res.unwrap() {
				let entry = entry.unwrap();
				let src_path = entry.path();
				let dst_path = unjar_path.join(entry.file_name());

				if src_path.is_file() {
					std::fs::copy(&src_path, &dst_path).unwrap();
				}
			}

			if let Err(e) = zip_dir(&unjar_path, &jar_path) {
				eprintln!("failed to repackage client.jar: {e}");
				return;
			}

			self.ver_dir = Some(risugami_dir);
			self.main_class = "org.mcphackers.launchwrapper.Launch".to_string();
		}
	}

	fn launch(&self, ctx: McLaunchCtx) {
		let Some(ver_dir) = &self.ver_dir else {
			eprintln!("launch called before install, version directory is not present; terminating..");
			std::process::exit(-1);
		};

		let text = std::fs::read_to_string(ver_dir.join("version.json")).expect("Failed to read version.json");
		let json: VanillaVersionJson = serde_json::from_str(&text).expect("Failed to parse version json");

		Vanilla { json: Some(json), ver_dir: Some(ver_dir.clone()) }.launch(ctx);
	}

	fn game_versions() -> String {
		RISUGAMI_VERSIONS.to_string()
	}

	fn id() -> String {
		"risugami".to_string()
	}
}
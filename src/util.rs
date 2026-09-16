use std::{error::Error, fs::{self, File}, io::{Read, Write}, path::{Path, PathBuf}};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use futures_util::StreamExt as _;

pub struct LauncherDirs {
	pub root_dir: PathBuf,
	pub game_dir: PathBuf,
	pub assets_dir: PathBuf,
	pub vers_dir: PathBuf
}

pub fn download_text(mp: &MultiProgress, url: &str, out: &Path, msg: String) -> Result<String, Box<dyn Error>> {
    let client = Client::new();
    let mut resp = client.get(url).send()?;

    let total = resp.content_length().unwrap_or(0);
    let pb = mp.add(ProgressBar::new(total));
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}")
        .unwrap()
        .progress_chars("#>-"));

    let mut f = File::create(out)?;
    let mut buf = [0u8; 8192]; // use a stack array buffer
    let mut downloaded = Vec::new(); // store downloaded data here

    loop {
        let bytes = resp.read(&mut buf)?;
        if bytes == 0 { break; }

        f.write_all(&buf[..bytes])?;
        downloaded.extend_from_slice(&buf[..bytes]);

        pb.inc(bytes as u64);
    }
    pb.finish_with_message(msg);

    Ok(String::from_utf8(downloaded)?) // now return the full text
}

pub async fn download_text_async(mp: &MultiProgress, url: &str, out: &Path, msg: String) -> Result<String, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let resp = client.get(url).send().await?;

    let total = resp.content_length().unwrap_or(0);
    let pb = mp.add(ProgressBar::new(total));
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}")
        .unwrap()
        .progress_chars("#>-"));

    let mut f = File::create(out)?;
    let mut stream = resp.bytes_stream();
    let mut data: Vec<u8> = vec![];

    while let Some(Ok(item)) = stream.next().await {
        f.write_all(&item)?;
        data.extend(&item);

        pb.inc(item.len() as u64);
    }
    pb.finish_with_message(msg);
    
    Ok(String::from_utf8(data)?) // now return the full text
}

pub fn download_text_no_save(mp: &MultiProgress, url: &str, msg: String) -> Result<String, Box<dyn Error>> {
    let client = Client::new();
    let mut resp = client.get(url).send()?;

    let total = resp.content_length().unwrap_or(0);
    let pb = mp.add(ProgressBar::new(total));
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}")
        .unwrap()
        .progress_chars("#>-"));

    let mut buf = [0u8; 8192]; // use a stack array buffer
    let mut downloaded = Vec::new(); // store downloaded data here

    loop {
        let bytes = resp.read(&mut buf)?;
        if bytes == 0 { break; }

        downloaded.extend_from_slice(&buf[..bytes]);

        pb.inc(bytes as u64);
    }
    pb.finish_with_message(msg);

    Ok(String::from_utf8(downloaded)?) // now return the full text
}

pub async fn download_text_no_save_async(mp: &MultiProgress, url: &str, msg: String) -> Result<String, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let resp = client.get(url).send().await?;

    let total = resp.content_length().unwrap_or(0);
    let pb = mp.add(ProgressBar::new(total));
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}")
        .unwrap()
        .progress_chars("#>-"));

    let mut stream = resp.bytes_stream();
    let mut data: Vec<u8> = vec![];

    while let Some(Ok(item)) = stream.next().await {
        data.extend(&item);

        pb.inc(item.len() as u64);
    }
    pb.finish_with_message(msg);

    Ok(String::from_utf8(data)?) // now return the full text
}

pub fn download(mp: &MultiProgress, url: &str, out: &Path, msg: String) -> Result<Vec<u8>, Box<dyn Error>> {
    let client = Client::new();
    let mut resp = client.get(url).send()?;

    let total = resp.content_length().unwrap_or(0);
    let pb = mp.add(ProgressBar::new(total));
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}")
        .unwrap()
        .progress_chars("#>-"));

    let mut f = File::create(out)?;
    let mut buf = [0u8; 8192];
    let mut downloaded = Vec::new();

    loop {
        let bytes = resp.read(&mut buf)?;
        if bytes == 0 { break; }

        f.write_all(&buf[..bytes])?;
        downloaded.extend_from_slice(&buf[..bytes]);

        pb.inc(bytes as u64);
    }
    pb.finish_with_message(msg);

    Ok(downloaded)
}

pub async fn download_async(mp: &MultiProgress, url: &str, out: &Path, msg: String) -> Result<Vec<u8>, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let resp = client.get(url).send().await?;

    let total = resp.content_length().unwrap_or(0);
    let pb = mp.add(ProgressBar::new(total));
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}")
        .unwrap()
        .progress_chars("#>-"));

    let mut f = File::create(out)?;
    let mut data: Vec<u8> = vec![];
    let mut stream = resp.bytes_stream();

    while let Some(Ok(item)) = stream.next().await {
        data.extend(&item);
        f.write_all(&item)?;

        pb.inc(item.len() as u64);
    }
    pb.finish_with_message(msg);

    Ok(data)
}


pub fn download_no_save(mp: &MultiProgress, url: &str, msg: String) -> Result<Vec<u8>, Box<dyn Error>> {
    let client = Client::new();
    let mut resp = client.get(url).send()?;

    let total = resp.content_length().unwrap_or(0);
    let pb = mp.add(ProgressBar::new(total));
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}")
        .unwrap()
        .progress_chars("#>-"));

    let mut buf = [0u8; 8192];
    let mut downloaded = Vec::new();

    loop {
        let bytes = resp.read(&mut buf)?;
        if bytes == 0 { break; }

        downloaded.extend_from_slice(&buf[..bytes]);

        pb.inc(bytes as u64);
    }
    pb.finish_with_message(msg);

    Ok(downloaded)
}

pub async fn download_no_save_async(mp: &MultiProgress, url: &str, msg: String) -> Result<Vec<u8>, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let resp = client.get(url).send().await?;

    let total = resp.content_length().unwrap_or(0);
    let pb = mp.add(ProgressBar::new(total));
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}")
        .unwrap()
        .progress_chars("#>-"));

    let mut stream = resp.bytes_stream();
    let mut data: Vec<u8> = vec![];

    while let Some(Ok(item)) = stream.next().await {
        data.extend(&item);

        pb.inc(item.len() as u64);
    }
    pb.finish_with_message(msg);

    Ok(data)
}

pub fn list_files_recursively(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if dir.is_dir() {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                files.extend(list_files_recursively(&path));
            } else {
                files.push(path);
            }
        }
    }
    files
}


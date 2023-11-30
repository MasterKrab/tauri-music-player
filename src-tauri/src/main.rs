#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[derive(Clone, serde::Serialize)]
struct Comment {
    lang: String,
    description: String,
    text: String,
}

#[derive(Clone, serde::Serialize)]
struct Picture {
    mime_type: String,
    picture_type: String,
    description: String,
    url: String,
}

#[derive(Clone, serde::Serialize)]
struct Song {
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    year: Option<u32>,
    comment: Option<String>,
    track: Option<u32>,
    genre: Option<String>,
    duration: Option<u32>,
}

use id3::Tag;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::PathBuf;
use taglib::File as FileTag;

#[tauri::command]
async fn read_song_metadata(path: &str) -> Result<Song, String> {
    let file = match FileTag::new(&path) {
        Ok(file) => file,
        Err(_e) => return Err("Failed to read file".to_string()),
    };

    let tag = match file.tag() {
        Ok(tag) => tag,
        Err(_e) => return Err("Failed to read metadata".to_string()),
    };

    let duration = match file.audioproperties() {
        Ok(properties) => Some(properties.length()),
        Err(_e) => return Err("Failed to read audio properties".to_string()),
    };

    let title = if let Some(title) = tag.title() {
        Some(title)
    } else {
        None
    };

    let artist = if let Some(artist) = tag.artist() {
        Some(artist)
    } else {
        None
    };

    let album = if let Some(album) = tag.album() {
        Some(album)
    } else {
        None
    };

    let year = if let Some(year) = tag.year() {
        Some(year)
    } else {
        None
    };

    let comment = if let Some(comment) = tag.comment() {
        Some(comment)
    } else {
        None
    };

    let track = if let Some(track) = tag.track() {
        Some(track)
    } else {
        None
    };

    let genre = if let Some(genre) = tag.genre() {
        Some(genre)
    } else {
        None
    };

    Ok(Song {
        title,
        artist,
        album,
        year,
        comment,
        track,
        genre,
        duration,
    })
}

use base64;

fn to_image_url(picture: &id3::frame::Picture) -> String {
    let base64 = base64::encode(&picture.data);
    format!("data:{};base64,{}", &picture.mime_type, base64)
}

fn image_url_to_buffer(url: &str) -> Result<Vec<u8>, std::io::Error> {
    let base64 = url.split(',').last().unwrap();
    Ok(base64::decode(&base64).unwrap())
}

#[tauri::command]
fn write_image_url(path: String, data: &str) -> Result<(), String> {
    let mut folders_path = PathBuf::from(&path);
    folders_path.pop();

    match create_dir_all(folders_path) {
        Ok(_) => (),
        Err(e) => return Err(e.to_string()),
    }

    let mut file = match File::create(path) {
        Ok(file) => file,
        Err(e) => return Err(e.to_string()),
    };

    match file.write_all(&image_url_to_buffer(data).unwrap()) {
        Ok(_) => Ok(()),
        Err(e) => return Err(e.to_string()),
    }
}

#[tauri::command]
fn get_song_picture(path: &str) -> Result<Option<Picture>, String> {
    let tag = match Tag::read_from_path(path) {
        Ok(tag) => tag,
        Err(e) => return Err(e.to_string()),
    };

    let picture = tag.pictures().next();

    if let Some(picture) = picture {
        Ok(Some(Picture {
            mime_type: picture.mime_type.to_string(),
            picture_type: picture.picture_type.to_string(),
            description: picture.description.to_string(),
            url: to_image_url(&picture),
        }))
    } else {
        Ok(None)
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            read_song_metadata,
            get_song_picture,
            write_image_url
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

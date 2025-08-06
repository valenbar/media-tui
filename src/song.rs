use color_eyre::Result;
use image::{DynamicImage, load_from_memory};
use lofty::{file::TaggedFileExt, probe::Probe, tag::Accessor};
use mpd::Client;
use std::fmt::Display;

use crate::ascii::AsciiEngine;

#[derive(Default)]
pub struct Song {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub cover: DynamicImage,
}

impl Song {
    pub fn from_mpd(mpd_connection: &mut Client, music_library: &str) -> Result<Self> {
        let current_song = mpd_connection
            .currentsong()
            .expect("Failed to get current song")
            .unwrap();

        let file = format!("{}/{}", music_library, &current_song.file);

        let tagged_file = Probe::open(file)
            .expect("ERROR: Bad path provided!")
            .read()
            .expect("ERROR: Failed to read file!");

        let tag = match tagged_file.primary_tag() {
            Some(primary_tag) => primary_tag,
            None => tagged_file.first_tag().expect("ERROR: No tags found!"),
        };

        let album = tag.album().as_deref().unwrap_or("None").to_owned();

        let cover = tag.pictures()[0].clone();
        let cover = load_from_memory(cover.data()).expect("ERROR: Failed to load cover");

        Ok(Song {
            title: current_song.title.unwrap(),
            artist: current_song.artist.unwrap(),
            album,
            cover,
        })
    }
}

impl Display for Song {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Title: {}", self.title)?;
        writeln!(f, "Artist: {}", self.artist)?;
        writeln!(f, "Album: {}", self.album)?;
        writeln!(
            f,
            "{}",
            AsciiEngine::Chafa.render_image_ansi(&self.cover).unwrap()
        )
    }
}

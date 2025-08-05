use color_eyre::{
    Result,
    eyre::{WrapErr, bail},
};
use image::{DynamicImage, ImageOutputFormat, load_from_memory};
use lofty::{file::TaggedFileExt, probe::Probe, tag::Accessor};
use mpd::Client;
use std::{
    fmt::Display,
    io::{Cursor, Write},
    process::{Command, Stdio},
};

pub struct Song {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub cover: DynamicImage,
}

impl Song {
    pub fn from_mpd(mpd_conn: &mut Client, music_library: String) -> Self {
        let current_song = mpd_conn
            .currentsong()
            .expect("Failed to get current song")
            .unwrap();

        let file = music_library + "/" + &current_song.file;

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

        Song {
            title: current_song.title.unwrap(),
            artist: current_song.artist.unwrap(),
            album,
            cover,
        }
    }

    #[allow(dead_code)]
    pub fn render_cover_using_rascii(&self) -> Result<String> {
        let mut cover_ascii = String::new();
        let charset = self
            .album
            .as_str()
            .chars()
            .map(|c| c.to_string())
            .collect::<Vec<String>>();
        let charset = charset.iter().map(|c| c.as_str()).collect::<Vec<&str>>();
        rascii_art::render_image_to(
            &self.cover,
            &mut cover_ascii,
            &rascii_art::RenderOptions {
                width: Some(25),
                colored: true,
                // charset: &["▁", "▂", "▃", "▄", "▅", "▆", "▇"],
                // charset: &["󰝤"],
                // charset: &["#"],
                // charset: &[" ", "░", "▒", "▓", "█"],
                charset: &charset,
                ..Default::default()
            },
        )
        .expect("ERROR: Failed to render ascii image");
        Ok(cover_ascii)
    }

    pub fn render_cover_using_chafa(&self) -> Result<String> {
        let mut image_buffer = Cursor::new(Vec::new());
        self.cover
            .write_to(&mut image_buffer, ImageOutputFormat::Png)?;
        let image_bytes = image_buffer.into_inner();

        let mut child = Command::new("chafa")
            .arg("--size=25x25")
            .arg("--format=symbols")
            .arg("--colors=full")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .wrap_err("failed to execute chafa command")?;

        {
            let mut stdin = child.stdin.take().unwrap();
            if let Err(err) = stdin.write_all(&image_bytes) {
                let _ = child.wait();
                bail!("failed to write image bytes: {err}");
            }
        }

        let output = child.wait_with_output()?;
        let ascii = String::from_utf8(output.stdout)?;
        Ok(ascii)
    }
}

impl Display for Song {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Title: {}", self.title)?;
        writeln!(f, "Artist: {}", self.artist)?;
        writeln!(f, "Album: {}", self.album)?;
        writeln!(f, "{}", &self.render_cover_using_chafa().unwrap())
    }
}

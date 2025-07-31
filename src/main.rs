use std::io::{Cursor, Write};
use std::process::{Command, Stdio};
use std::{error::Error, fmt::Display};

use crossterm::event::{self, Event};
use image::{DynamicImage, ImageOutputFormat, load_from_memory};
use lofty::prelude::*;
use lofty::probe::Probe;
use mpd::Client;
use rascii_art::{self};
use ratatui::{Frame, text::Text};

const MUSIC_LIBRARY: &str = "/mnt/Volume/music-library/Music";

struct Song {
    title: String,
    artist: String,
    album: String,
    cover: DynamicImage,
}

impl Song {
    fn from_mpd(mpd_conn: &mut Client, music_library: String) -> Self {
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

    fn get_cover_ascii(&self) -> Result<String, Box<dyn Error>> {
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
}

impl Display for Song {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Title: {}", self.title)?;
        writeln!(f, "Artist: {}", self.artist)?;
        writeln!(f, "Album: {}", self.album)?;
        writeln!(f, "{}", &self.get_cover_ascii().expect("fail"))
    }
}

fn draw(frame: &mut Frame) {
    let mut conn = Client::connect("localhost:6600").expect("Failed to connect to MPD server");
    let song = Song::from_mpd(&mut conn, MUSIC_LIBRARY.to_string());

    let text = Text::raw(song.get_cover_ascii().unwrap());
    frame.render_widget(text, frame.area());
}

fn run(terminal: &mut ratatui::DefaultTerminal) -> std::io::Result<()> {
    loop {
        terminal.draw(draw).expect("failed to draw frame");
        if matches!(event::read().expect("failed to read event"), Event::Key(_)) {
            break Ok(());
        }
    }
}

fn print_using_chafa(image: &DynamicImage) -> Result<String, Box<dyn Error>> {
    let mut child = Command::new("chafa")
        .arg("--size=50x25")
        .arg("--format=symbols")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    {
        let mut buffer = Cursor::new(Vec::new());
        image.write_to(&mut buffer, ImageOutputFormat::Png)?;
        let image_bytes = buffer.into_inner();

        let mut stdin = child.stdin.take().unwrap();
        stdin.write_all(&image_bytes);
    }

    let output = child.wait_with_output()?;
    let ascii = String::from_utf8(output.stdout)?;
    Ok(ascii)
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut conn = Client::connect("localhost:6600").expect("Failed to connect to MPD server");

    let song = Song::from_mpd(&mut conn, MUSIC_LIBRARY.to_string());

    // println!("{}", song.get_cover_ascii().unwrap());

    println!("{}", print_using_chafa(&song.cover).unwrap());

    // let mut terminal = ratatui::init();
    // run(&mut terminal)?;
    // ratatui::restore();

    Ok(())
}

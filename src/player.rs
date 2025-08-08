use color_eyre::{Result, eyre::Context};
use lofty::{file::TaggedFileExt, probe::Probe, tag::Accessor};

use crate::song;

const DEFAULT_HOST: &str = "localhost";
const DEFAULT_PORT: u32 = 6600;

pub trait Player {
    fn next_song(&mut self) -> Result<()>;
    fn prev_song(&mut self) -> Result<()>;
    fn toggle_play_pause(&mut self) -> Result<()>;
    fn get_song_info(&mut self) -> Result<song::Song>;
    // fn get_song_cover(&mut self) -> Result<image::DynamicImage>;
}

pub struct MPDPlayer {
    host: String,
    port: u32,
    music_library_dir: String,
    mpd_connection: mpd::Client,
}

impl Default for MPDPlayer {
    fn default() -> Self {
        Self {
            host: DEFAULT_HOST.to_string(),
            port: DEFAULT_PORT,
            music_library_dir: Default::default(),
            mpd_connection: Default::default(),
        }
    }
}

impl MPDPlayer {
    pub fn new(host: String, port: u32, music_library_dir: String) -> Result<Self> {
        let mpd_connection = connect_to_mpd(&host, &port)?;
        Ok(Self {
            host,
            port,
            music_library_dir,
            mpd_connection,
        })
    }
}

fn connect_to_mpd(host: &String, port: &u32) -> Result<mpd::Client> {
    let address = format!("{host}:{port}");
    let mpd_connection = mpd::Client::connect(address).wrap_err("failed to connect to mpd")?;
    Ok(mpd_connection)
}

impl Player for MPDPlayer {
    fn next_song(&mut self) -> Result<()> {
        self.mpd_connection.next()?;
        Ok(())
    }

    fn prev_song(&mut self) -> Result<()> {
        self.mpd_connection.prev()?;
        Ok(())
    }

    fn toggle_play_pause(&mut self) -> Result<()> {
        self.mpd_connection.toggle_pause()?;
        Ok(())
    }

    fn get_song_info(&mut self) -> Result<song::Song> {
        let current_song = self
            .mpd_connection
            .currentsong()
            .expect("Failed to get current song")
            .unwrap();

        let file = format!("{}/{}", self.music_library_dir, &current_song.file);

        let tagged_file = Probe::open(file)
            .expect("ERROR: Bad path provided!")
            .read()
            .expect("ERROR: Failed to read file!");

        let tag = match TaggedFileExt::primary_tag(&tagged_file) {
            Some(primary_tag) => primary_tag,
            None => TaggedFileExt::first_tag(&tagged_file).expect("ERROR: No tags found!"),
        };

        let album = tag.album().as_deref().unwrap_or("None").to_owned();

        let cover = tag.pictures()[0].clone();
        let cover = image::load_from_memory(cover.data()).expect("ERROR: Failed to load cover");

        Ok(song::Song {
            title: current_song.title.unwrap(),
            artist: current_song.artist.unwrap(),
            album,
            cover,
        })
    }
}

pub struct MPRISPlayer;

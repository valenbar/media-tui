use std::{thread::sleep, time::Duration};

use color_eyre::{
    Result,
    eyre::{Context, bail},
};
use lofty::{file::TaggedFileExt, probe::Probe, tag::Accessor};

use crate::song;

const DEFAULT_HOST: &str = "localhost";
const DEFAULT_PORT: u32 = 6600;

pub trait Player {
    fn next_song(&mut self) -> Result<()>;
    fn previous_song(&mut self) -> Result<()>;
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

    fn previous_song(&mut self) -> Result<()> {
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
            cover: Some(cover),
        })
    }
}

pub struct MPRISPlayer {
    mpris_player: mpris::Player,
}

impl MPRISPlayer {
    pub fn new() -> Result<Self> {
        let mpris_player = mpris::PlayerFinder::new()?.find_active()?;
        Ok(Self { mpris_player })
    }
}

impl Player for MPRISPlayer {
    fn next_song(&mut self) -> Result<()> {
        self.mpris_player.next()?;
        sleep(Duration::from_millis(300));
        Ok(())
    }

    fn previous_song(&mut self) -> Result<()> {
        self.mpris_player.previous()?;
        sleep(Duration::from_millis(300));
        Ok(())
    }

    fn toggle_play_pause(&mut self) -> Result<()> {
        self.mpris_player.play_pause()?;
        Ok(())
    }

    fn get_song_info(&mut self) -> Result<song::Song> {
        let metadata = self.mpris_player.get_metadata()?;
        let title = metadata.title().unwrap_or("<missing title>").to_string();
        let artist = metadata
            .artists()
            .unwrap_or(vec!["<missing artist>"])
            .join("& ");
        let album = metadata
            .album_name()
            .unwrap_or("<missing album>")
            .to_string();
        let cover = match metadata.art_url() {
            Some(url) => get_cover_image_from_url(url)?,
            None => None,
        };

        let song = song::Song {
            title,
            artist,
            album,
            cover,
        };
        Ok(song)
    }
}

fn get_cover_image_from_url(image_url: &str) -> Result<Option<image::DynamicImage>> {
    let url = url::Url::parse(image_url)
        .wrap_err_with(|| format!("URL parsing failed\n{image_url:#?}"))?;
    match url.scheme() {
        "file" => {
            let path = url
                .to_file_path()
                .expect("Failed to convert file URL to path");
            let img = image::io::Reader::open(path)?.decode()?;
            Ok(Some(img))
        }
        scheme => {
            bail!("Unable to load image from URL, scheme: \"{scheme}\" not implemented yet");
        }
    }
}

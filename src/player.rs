use std::{fs::File, io::BufReader, thread::sleep, time::Duration};

use color_eyre::{
    Result,
    eyre::{self, Context, bail},
};
use lofty::{file::TaggedFileExt, probe::Probe, tag::Accessor};

use crate::song;

const MPRIS_METADATA_QUERY_DELAY: Duration = Duration::from_millis(100);

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
            .wrap_err_with(|| {
                format!(
                    "Failed to open audio file at:\n{}/{}\nunable to get cover image",
                    self.music_library_dir, &current_song.file
                )
            })?
            .read()
            .expect("ERROR: Failed to read file!");

        let tag = match TaggedFileExt::primary_tag(&tagged_file) {
            Some(primary_tag) => primary_tag,
            None => TaggedFileExt::first_tag(&tagged_file).expect("ERROR: No tags found!"),
        };

        // TODO handle rating tags
        // let rating = tag.get_string(&lofty::tag::ItemKey::Popularimeter);

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
        let events = self.mpris_player.events()?;
        self.mpris_player.next()?;
        for event in events {
            match event? {
                mpris::Event::TrackChanged(metadata) => break,
                _ => continue,
            }
        }
        Ok(())
    }

    fn previous_song(&mut self) -> Result<()> {
        let events = self.mpris_player.events()?;
        self.mpris_player.previous()?;
        for event in events {
            match event? {
                mpris::Event::TrackChanged(metadata) => break,
                _ => continue,
            }
        }
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
                .map_err(|_| eyre::eyre!("invalid file URL"))
                .wrap_err("Failed to convert file URL to path")?;

            let img = match path.extension() {
                Some(_) => {
                    // infers image format from extension
                    image::io::Reader::open(&path)?
                        .decode()
                        .wrap_err("Failed to open image from path")?
                }
                None => {
                    // Assume file without extension is a jpeg
                    let image_format = image::ImageFormat::Jpeg;
                    image::load(BufReader::new(File::open(&path)?), image_format)
                        .wrap_err("Failed to load image without extension as a jpeg")?
                }
            };
            Ok(Some(img))
        }
        "http" | "https" => {
            let response = reqwest::blocking::get(url.as_str()).wrap_err("HTTP request failed")?;
            if !response.status().is_success() {
                bail!("HTTP request returned error: {}", response.status());
            }
            let bytes = response.bytes().wrap_err("Failed to read response body")?;
            let img = image::load_from_memory(&bytes).wrap_err("Failed to decode image")?;
            Ok(Some(img))
        }
        scheme => {
            bail!(
                "Unable to load image from URL, scheme: \"{scheme}\" not implemented yet. unable to get thumbnail from: {url}"
            );
        }
    }
}

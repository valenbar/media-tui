use std::{fs::File, io::BufReader, thread::sleep, time::Duration};

use color_eyre::{
    Result,
    eyre::{self, Context, bail},
};

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
        song::Song::from_mpd(current_song, &self.music_library_dir)
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
        song::Song::from_mpris(metadata)
    }
}

use color_eyre::{Result, eyre::WrapErr};

use crate::ascii;
use crate::song::Song;

pub struct App {
    host: String,
    port: u32,
    music_library: String,
    mpd_connection: mpd::Client,
    pub current_song: Song,
    ascii_engine: ascii::AsciiEngine,
    pub exit: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            host: String::from("localhost"),
            port: 6600,
            music_library: Default::default(),
            mpd_connection: Default::default(),
            current_song: Default::default(),
            ascii_engine: ascii::AsciiEngine::Chafa,
            exit: false,
        }
    }
}

impl App {
    pub fn new(
        music_library: String,
        host: Option<String>,
        port: Option<u32>,
        ascii_engine: Option<ascii::AsciiEngine>,
    ) -> Result<Self> {
        let mut app = App {
            host: host.unwrap_or_default(),
            port: port.unwrap_or_default(),
            music_library,
            ascii_engine: ascii_engine.unwrap_or_default(),
            ..Default::default()
        };
        let address = format!("{}:{}", app.host, app.port);
        app.mpd_connection = connect_to_mpd(address)?;
        app.current_song = get_current_song(&app.music_library, &mut app.mpd_connection)?;
        Ok(app)
    }

    pub fn get_cover(&self) -> Result<String> {
        self.current_song.generate_cover_ascii(&self.ascii_engine)
    }
}

fn get_current_song(music_library: &str, mpd_connection: &mut mpd::Client) -> Result<Song> {
    let current_song =
        Song::from_mpd(mpd_connection, music_library).wrap_err("failed to get current song")?;
    Ok(current_song)
}

fn connect_to_mpd(address: String) -> Result<mpd::Client> {
    let mpd_connection = mpd::Client::connect(address).wrap_err("failed to connect to mpd")?;
    Ok(mpd_connection)
}

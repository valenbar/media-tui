use color_eyre::{Result, eyre::WrapErr};
use ratatui::style::Stylize;
use ratatui::widgets::Widget;
use ratatui::{DefaultTerminal, text};

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

    pub fn get_cover_ascii(&self, size: ascii::Size) -> Result<String> {
        self.ascii_engine
            .render_image_ansi(&self.current_song.cover, size)
    }

    pub fn get_cover_ascii_tui(&self, size: ascii::Size) -> Result<text::Text> {
        self.ascii_engine
            .render_image_tui(&self.current_song.cover, size)
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events().wrap_err("handle events failed")?;
        }
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn draw(&self, frame: &mut ratatui::Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> Result<()> {
        // TODO check if mpd song changed
        match crossterm::event::read()? {
            crossterm::event::Event::Key(key_event)
                if key_event.kind == crossterm::event::KeyEventKind::Press =>
            {
                self.handle_key_event(key_event)
                    .wrap_err_with(|| format!("handling key event failed:\n{key_event:#?}"))
            }
            _ => Ok(()),
        }
    }

    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> Result<()> {
        use crossterm::event::KeyCode;
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Esc => self.exit(),
            KeyCode::Enter => self.mpd_connection.toggle_pause()?,
            KeyCode::Left => {
                self.mpd_connection.prev()?;
                self.current_song = Song::from_mpd(&mut self.mpd_connection, &self.music_library)?
            }
            KeyCode::Right => {
                self.mpd_connection.next()?;
                self.current_song = Song::from_mpd(&mut self.mpd_connection, &self.music_library)?
            }
            KeyCode::Char('c') => {
                use ascii::AsciiEngine::{Chafa, Rascii};
                self.ascii_engine = match self.ascii_engine {
                    Chafa => Rascii,
                    Rascii => Chafa,
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let song_title = &self.current_song.title;
        let song_artist = &self.current_song.artist;
        let song_album = &self.current_song.album;
        let header = ratatui::text::Line::from(format!(" {song_title} - {song_artist} ").bold());
        let footer = ratatui::text::Line::from(format!(" {song_album} ").bold());
        let block = ratatui::widgets::Block::bordered()
            .title_top(header.centered())
            .title_bottom(footer.centered());
        let ascii_size = ascii::Size {
            width: area.width,
            height: area.height,
        };
        let ascii = match self
            .ascii_engine
            .render_image_tui(&self.current_song.cover, ascii_size)
        {
            Ok(ascii) => ascii,
            Err(e) => ratatui::text::Text::from(e.to_string()),
        };
        ratatui::widgets::Paragraph::new(ascii)
            .centered()
            .block(block)
            .render(area, buf);
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

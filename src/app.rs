use color_eyre::{Result, eyre::WrapErr};
use ratatui::style::Stylize;
use ratatui::widgets::Widget;
use ratatui::{DefaultTerminal, text};

use crate::ascii;
use crate::player;
use crate::song::Song;

pub struct App {
    pub player: Box<dyn player::Player>,
    pub current_song: Song,
    ascii_engine: ascii::AsciiEngine,
    pub exit: bool,
}

impl App {
    pub fn new(
        player: Box<dyn player::Player>,
        ascii_engine: Option<ascii::AsciiEngine>,
    ) -> Result<Self> {
        let mut app = App {
            player,
            ascii_engine: ascii_engine.unwrap_or_default(),
            current_song: Default::default(),
            exit: false,
        };
        app.current_song = app.player.get_song_info()?;
        Ok(app)
    }

    pub fn get_cover_ascii(&self, size: ascii::Size) -> Result<String> {
        match &self.current_song.cover {
            Some(cover_image) => self.ascii_engine.render_image_ansi(cover_image, size),
            None => Ok("<missing cover>".to_string()),
        }
    }

    pub fn get_cover_ascii_tui(&self, size: ascii::Size) -> Result<text::Text> {
        match &self.current_song.cover {
            Some(cover_image) => self.ascii_engine.render_image_tui(cover_image, size),
            None => Ok(text::Text::from("<missing cover>")),
        }
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
            KeyCode::Enter => self.player.toggle_play_pause()?,
            KeyCode::Left => {
                self.player.previous_song()?;
                self.current_song = self.player.get_song_info()?
            }
            KeyCode::Right => {
                self.player.next_song()?;
                self.current_song = self.player.get_song_info()?
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
        let ascii = self
            .get_cover_ascii_tui(ascii_size)
            .expect("Failed to get Cover in tui format");
        // let ascii = match self
        //     .ascii_engine
        //     .render_image_tui(&self.current_song.cover, ascii_size)
        // {
        //     Ok(ascii) => ascii,
        //     Err(e) => ratatui::text::Text::from(e.to_string()),
        // };
        ratatui::widgets::Paragraph::new(ascii)
            .centered()
            .block(block)
            .render(area, buf);
    }
}

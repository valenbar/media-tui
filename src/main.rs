use color_eyre::Result;

mod app;
mod ascii;
mod player;
mod song;

use crate::app::App;

#[cfg(test)]
mod tests;

const MUSIC_LIBRARY: &str = "/mnt/Volume/music-library/Music";

fn main() -> Result<()> {
    color_eyre::install()?;

    let player: Box<dyn player::Player> = Box::new(player::MPDPlayer::new(
        "localhost".to_string(),
        6600,
        MUSIC_LIBRARY.to_owned(),
    )?);

    let mut app = App::new(player, Some(ascii::AsciiEngine::Chafa))?;

    let mut terminal = ratatui::init();
    let app_result = app.run(&mut terminal);
    ratatui::restore();
    app_result

    // println!("{}", app.get_cover_ascii()?);
    // println!("{} - {}", app.current_song.artist, app.current_song.title);
}

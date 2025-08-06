use color_eyre::Result;

mod app;
mod ascii;
mod song;

use crate::app::App;

#[cfg(test)]
mod tests;

const MUSIC_LIBRARY: &str = "/mnt/Volume/music-library/Music";

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut app = App::new(
        MUSIC_LIBRARY.to_string(),
        Some("localhost".to_string()),
        Some(6600),
        Some(ascii::AsciiEngine::Chafa),
    )?;

    let mut terminal = ratatui::init();
    let app_result = app.run(&mut terminal);
    ratatui::restore();
    app_result

    // println!("{}", app.get_cover_ascii()?);
    // println!("{} - {}", app.current_song.artist, app.current_song.title);
}

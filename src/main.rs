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

    let app = App::new(
        MUSIC_LIBRARY.to_string(),
        Some("localhost".to_string()),
        Some(6600),
        Some(ascii::AsciiEngine::Chafa),
    )?;

    println!("{}", app.get_cover()?);
    println!("{} - {}", app.current_song.artist, app.current_song.title);

    Ok(())
}

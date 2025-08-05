use color_eyre::Result;

use mpd::Client;

mod song;

#[cfg(test)]
mod tests;

const MUSIC_LIBRARY: &str = "/mnt/Volume/music-library/Music";

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut conn = Client::connect("localhost:6600").expect("Failed to connect to MPD server");

    let song = song::Song::from_mpd(&mut conn, MUSIC_LIBRARY.to_string());

    // println!("{}", song.get_cover_ascii().unwrap());

    println!("{}", song.render_cover_using_chafa()?);
    println!("{} - {}", song.artist, song.title);

    Ok(())
}

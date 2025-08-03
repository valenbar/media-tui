use color_eyre::{
    Result,
    eyre::{WrapErr, bail},
};

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

    print!(
        "{}",
        song::print_using_chafa(&song.cover).wrap_err("failed to generate ascii")?
    );
    println!("{} - {}", song.artist, song.title);

    Ok(())
}

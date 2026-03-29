use clap::{Parser, ValueEnum};
use color_eyre::{Result, eyre::bail};

mod app;
mod ascii;
mod player;
mod song;

use crate::app::App;

#[cfg(test)]
mod tests;

const DEFAULT_ADDRESS: &str = "localhost:6600";

#[derive(Debug, Clone, ValueEnum)]
enum PlayerSelection {
    Mpd,
    Mpris,
}

/// Small terminal music player to control MPD or MPRIS with ascii cover images
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// player to use
    #[arg(short, long, value_enum, default_value_t = PlayerSelection::Mpris, requires_if("mpd", "dir"))]
    player: PlayerSelection,

    /// MPD server address in format 'hostname:port'
    #[arg(short, long, value_parser = parse_address, default_value = DEFAULT_ADDRESS)]
    address: (String, u32),

    /// MPD music directory
    #[arg(short, long, value_parser = parse_dir)]
    dir: Option<String>,
}

/// Custom parser for host:port format
fn parse_address(address: &str) -> Result<(String, u32)> {
    let parts: Vec<&str> = address.split(':').collect();
    if parts.len() != 2 {
        bail!("Address must be in format 'hostname:port");
    }

    let host = parts[0].to_string();
    let port = parts[1].parse::<u32>()?;
    Ok((host, port))
}

fn parse_dir(path: &str) -> Result<String> {
    match std::fs::exists(path).unwrap() {
        true => Ok(path.to_string()),
        false => bail!("Invalid directory: \"{path}\""),
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    let player: Box<dyn player::Player> = match args.player {
        PlayerSelection::Mpd => {
            let host = args.address.0;
            let port = args.address.1;
            let music_library_dir = args.dir.unwrap();
            Box::new(player::MPDPlayer::new(host, port, music_library_dir)?)
        }

        PlayerSelection::Mpris => Box::new(player::MPRISPlayer::new()?),
    };

    // let player: Box<dyn player::Player> = Box::new(player::MPDPlayer::new(
    //     host,
    //     port,
    //     MUSIC_LIBRARY.to_owned(),
    // )?);

    // let player: Box<dyn player::Player> = Box::new(player::MPRISPlayer::new()?);

    let mut app = App::new(player, Some(ascii::AsciiEngine::Chafa))?;

    let mut terminal = ratatui::init();
    let app_result = app.run(&mut terminal);
    ratatui::restore();
    app_result

    // println!("{}", app.get_cover_ascii()?);
    // println!("{} - {}", app.current_song.artist, app.current_song.title);
}

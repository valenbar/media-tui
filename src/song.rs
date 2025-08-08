use std::fmt::Display;

use crate::ascii::{self, AsciiEngine};

#[derive(Default)]
pub struct Song {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub cover: Option<image::DynamicImage>,
}

impl Display for Song {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Title: {}", self.title)?;
        writeln!(f, "Artist: {}", self.artist)?;
        writeln!(f, "Album: {}", self.album)?;
        if let Some(cover_image) = &self.cover {
            writeln!(
                f,
                "{}",
                AsciiEngine::Chafa
                    .render_image_ansi(cover_image, ascii::Size::default())
                    .unwrap()
            )?;
        }
        Ok(())
    }
}

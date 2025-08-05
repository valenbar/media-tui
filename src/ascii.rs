use std::{
    io::{Cursor, Write},
    process::{Command, Stdio},
};

use color_eyre::{
    Result,
    eyre::{Context, bail},
};
use image::{DynamicImage, ImageOutputFormat};

pub enum AsciiEngine {
    Chafa,
    Rascii,
}

impl Default for AsciiEngine {
    fn default() -> Self {
        Self::Chafa
    }
}

impl AsciiEngine {
    pub fn render_image(&self, image: &DynamicImage) -> Result<String> {
        match self {
            AsciiEngine::Chafa => Self::render_image_with_chafa(image),
            AsciiEngine::Rascii => Self::render_image_with_rascii(image),
        }
    }

    fn render_image_with_chafa(image: &DynamicImage) -> Result<String> {
        let mut image_buffer = Cursor::new(Vec::new());
        image.write_to(&mut image_buffer, ImageOutputFormat::Png)?;
        let image_bytes = image_buffer.into_inner();

        let mut child = Command::new("chafa")
            .arg("--size=25x25")
            .arg("--format=symbols")
            .arg("--colors=full")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .wrap_err("failed to execute chafa command")?;

        {
            let mut stdin = child.stdin.take().unwrap();
            if let Err(err) = stdin.write_all(&image_bytes) {
                let _ = child.wait();
                bail!("failed to write image bytes: {err}");
            }
        }

        let output = child.wait_with_output()?;
        let ascii = String::from_utf8(output.stdout)?;
        Ok(ascii)
    }

    fn render_image_with_rascii(image: &DynamicImage) -> Result<String> {
        let mut cover_ascii = String::new();
        rascii_art::render_image_to(
            image,
            &mut cover_ascii,
            &rascii_art::RenderOptions {
                width: Some(25),
                colored: true,
                // charset: &["▁", "▂", "▃", "▄", "▅", "▆", "▇"],
                // charset: &["󰝤"],
                // charset: &["#"],
                // charset: &[" ", "░", "▒", "▓", "█"],
                // charset: &charset,
                ..Default::default()
            },
        )
        .expect("ERROR: Failed to render ascii image");
        Ok(cover_ascii)
    }
}

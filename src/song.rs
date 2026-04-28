use color_eyre::{
    Result,
    eyre::{self, Context, bail},
};
use lofty::{file::TaggedFileExt, probe::Probe, tag::Accessor};
use std::{fmt::Display, fs::File, io::BufReader};

use crate::ascii::{self, AsciiEngine};

#[derive(Default, PartialEq)]
pub struct Song {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub cover: Option<image::DynamicImage>,
}

impl Song {
    pub fn from_mpd(mpd_song: mpd::Song, music_library_dir: &str) -> Result<Self> {
        let file = format!("{}/{}", music_library_dir, &mpd_song.file);

        let tagged_file = Probe::open(file)
            .wrap_err_with(|| {
                format!(
                    "Failed to open audio file at:\n{}/{}\nunable to get cover image",
                    music_library_dir, &mpd_song.file
                )
            })?
            .read()
            .expect("ERROR: Failed to read file!");

        let tag = match TaggedFileExt::primary_tag(&tagged_file) {
            Some(primary_tag) => primary_tag,
            None => TaggedFileExt::first_tag(&tagged_file).expect("ERROR: No tags found!"),
        };

        // TODO handle rating tags
        // let rating = tag.get_string(&lofty::tag::ItemKey::Popularimeter);

        let album = tag.album().as_deref().unwrap_or("None").to_owned();

        let cover = tag.pictures()[0].clone();
        let cover = image::load_from_memory(cover.data()).expect("ERROR: Failed to load cover");

        Ok(Song {
            title: mpd_song.title.unwrap(),
            artist: mpd_song.artist.unwrap(),
            album,
            cover: Some(cover),
        })
    }

    pub fn from_mpris(metadata: mpris::Metadata) -> Result<Self> {
        let title = metadata.title().unwrap_or("<missing title>").to_string();
        let artist = metadata
            .artists()
            .unwrap_or(vec!["<missing artist>"])
            .join("& ");
        let album = metadata
            .album_name()
            .unwrap_or("<missing album>")
            .to_string();
        let cover = match metadata.art_url() {
            Some(url) => get_cover_image_from_url(url)?,
            None => None,
        };

        Ok(Song {
            title,
            artist,
            album,
            cover,
        })
    }
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

fn get_cover_image_from_url(image_url: &str) -> Result<Option<image::DynamicImage>> {
    let url = url::Url::parse(image_url)
        .wrap_err_with(|| format!("URL parsing failed\n{image_url:#?}"))?;
    match url.scheme() {
        "file" => {
            let path = url
                .to_file_path()
                .map_err(|_| eyre::eyre!("invalid file URL"))
                .wrap_err("Failed to convert file URL to path")?;

            let img = match path.extension() {
                Some(_) => {
                    // infers image format from extension
                    image::io::Reader::open(&path)?
                        .decode()
                        .wrap_err("Failed to open image from path")?
                }
                None => {
                    // Assume file without extension is a jpeg
                    let image_format = image::ImageFormat::Jpeg;
                    image::load(BufReader::new(File::open(&path)?), image_format)
                        .wrap_err("Failed to load image without extension as a jpeg")?
                }
            };
            Ok(Some(img))
        }
        "http" | "https" => {
            let response = reqwest::blocking::get(url.as_str()).wrap_err("HTTP request failed")?;
            if !response.status().is_success() {
                bail!("HTTP request returned error: {}", response.status());
            }
            let bytes = response.bytes().wrap_err("Failed to read response body")?;
            let img = image::load_from_memory(&bytes).wrap_err("Failed to decode image")?;
            Ok(Some(img))
        }
        scheme => {
            bail!(
                "Unable to load image from URL, scheme: \"{scheme}\" not implemented yet. unable to get thumbnail from: {url}"
            );
        }
    }
}

// fn get_rating_from_file(file: &str) -> Rating {}

use std::path::{Path, PathBuf};
use crate::constants;

#[derive(Debug)]
pub enum ReadImageError {
    FileNotFound(PathBuf),
    ImageError(image::ImageError),
    NotRgba8
}

impl std::fmt::Display for ReadImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadImageError::FileNotFound(path) => write!(f, "File not found: {:?}", path),
            ReadImageError::ImageError(err) => write!(f, "Image errror: {}", err),
            ReadImageError::NotRgba8 => write!(f, "The image doesn't support format RGBA")
        }
    }
}

impl std::error::Error for ReadImageError { }

pub struct ImageData {
    image: image::DynamicImage
}

impl ImageData {
    pub fn data(&self) -> Result<&[u8], ReadImageError> {
        self.image.as_rgba8()
            .map(|image| image.as_raw().as_slice())
            .ok_or(ReadImageError::NotRgba8)
    }

    pub fn width(&self) -> u32 {
        self.image.width()
    }

    pub fn height(&self) -> u32 {
        self.image.height()
    }
}

pub fn read_image(path: &Path) -> Result<ImageData, ReadImageError> {
    let file = constants::RESOURCE_DIR.get_file(path)
        .ok_or(ReadImageError::FileNotFound(path.into()))?;

    let image_bytes: &[u8] = file.contents();
    let image = image::load_from_memory(image_bytes)
        .map_err(|err| ReadImageError::ImageError(err))?
        .flipv();

    Ok(ImageData { image })
}
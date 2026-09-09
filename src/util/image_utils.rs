use std::path::{Path, PathBuf};
use image::GenericImageView;
use crate::constants;

#[derive(Debug)]
pub enum ReadImageError {
    FileNotFound(PathBuf),
    ImageError(image::ImageError),
}

impl std::fmt::Display for ReadImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadImageError::FileNotFound(path) => write!(f, "File not found: {:?}", path),
            ReadImageError::ImageError(err) => write!(f, "Image error: {}", err),
        }
    }
}

impl std::error::Error for ReadImageError { }

#[derive(Debug)]
pub struct PixelOutOfBoundError {
    row: u32,
    col: u32,
    width: u32,
    height: u32
}

impl PixelOutOfBoundError {
    fn new(row: u32, col: u32, width: u32, height: u32) -> Self {
        Self { row, col, width, height }
    }
}

impl std::fmt::Display for PixelOutOfBoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Index (row: {}, col: {}) is out of bound for image with size ({}, {})", self.row, self.col, self.width, self.height)
    }
}

impl std::error::Error for PixelOutOfBoundError { }

pub struct ImageData {
    image: image::DynamicImage
}

impl ImageData {
       pub fn width(&self) -> u32 {
        self.image.width()
    }

    pub fn height(&self) -> u32 {
        self.image.height()
    }

    pub fn get_pixel(&self, row: u32, col: u32) -> Result<image::Rgba<u8>, PixelOutOfBoundError> {
        if row >= self.height() || col >= self.width() { Err(PixelOutOfBoundError::new(row, col, self.width(), self.height())) }
        else { Ok(self.image.get_pixel(col, row)) }
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
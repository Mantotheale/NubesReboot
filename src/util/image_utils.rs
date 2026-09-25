use std::path::{Path, PathBuf};
use image::{EncodableLayout, GenericImageView};
use crate::constants;

#[derive(Debug)]
pub struct NotRGBAImageError;

impl std::fmt::Display for NotRGBAImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "The provided data doesn't support RBGA channels")
    }
}

impl std::error::Error for NotRGBAImageError { }

#[derive(Debug)]
pub enum ReadImageError {
    FileNotFound(PathBuf),
    ImageError(image::ImageError),
    NotRGBAImageError
}

impl std::fmt::Display for ReadImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadImageError::FileNotFound(path) => write!(f, "File not found: {:?}", path),
            ReadImageError::ImageError(err) => write!(f, "Image error: {}", err),
            ReadImageError::NotRGBAImageError => write!(f, "The loaded image doesn't support RBGA channels"),
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
    image: image::RgbaImage
}

impl ImageData {
    pub fn from_data(data: &[u8], width: u32, height: u32) -> Result<Self, NotRGBAImageError> {
        let image = image::RgbaImage::from_raw(width, height, data.into())
            .ok_or(NotRGBAImageError)?;
        Ok(Self { image })
    }

    pub fn width(&self) -> u32 {
        self.image.width()
    }

    pub fn height(&self) -> u32 {
        self.image.height()
    }

    pub fn get_pixel(&self, row: u32, col: u32) -> Result<image::Rgba<u8>, PixelOutOfBoundError> {
        if row >= self.height() || col >= self.width() { Err(PixelOutOfBoundError::new(row, col, self.width(), self.height())) }
        else { Ok(*self.image.get_pixel(col, row)) }
    }

    pub fn set_pixel(&mut self, row: u32, col: u32, pixel: image::Rgba<u8>) -> Result<(), PixelOutOfBoundError> {
        if row >= self.height() || col >= self.width() { Err(PixelOutOfBoundError::new(row, col, self.width(), self.height())) }
        else { 
            self.image.put_pixel(col, row, pixel);
            Ok(()) 
        }
    }

    pub fn data(&self) -> &[u8] {
        self.image.as_bytes()
    }
    
    pub fn copy_with_offset(&mut self, row_offset: u32, col_offset: u32, source: &Self) -> Result<(), PixelOutOfBoundError> {
        Ok(for row in 0..source.height() {
            for col in 0..source.width() {
                self.set_pixel(
                    row_offset + row,
                    col_offset + col,
                    source.get_pixel(row, col)?
                )?;
            }
        })
    }
    
    pub fn sub_image(&self, row_offset: u32, col_offset: u32, width: u32, height: u32) -> Result<Self, PixelOutOfBoundError> {
        let mut sub_image = Self::from_data(
            &vec![0; (width * height * 4) as usize],
            width,
            height
        ).expect("Supported format");

        for row in 0..height {
            for col in 0..width {
                sub_image.set_pixel(
                    row,
                    col,
                    self.get_pixel(row_offset + row, col_offset + col)?
                )?;
            }
        }
        
        Ok(sub_image)
    }
}

pub fn read_image(path: &Path) -> Result<ImageData, ReadImageError> {
    let file = constants::RESOURCE_DIR.get_file(path)
        .ok_or(ReadImageError::FileNotFound(path.into()))?;

    let image_bytes: &[u8] = file.contents();
    let image = image::load_from_memory(image_bytes)
        .map_err(|err| ReadImageError::ImageError(err))?
        .flipv()
        .into_rgba8();

    Ok(ImageData { image })
}
use std::fmt;

#[allow(dead_code)]
#[derive(Debug)]
pub enum CardError {
    FontLoadError(String),
    RenderError(String),
    IoError(std::io::Error),
    ImageError(image::ImageError),
}

impl fmt::Display for CardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CardError::FontLoadError(msg) => write!(f, "Font load error: {}", msg),
            CardError::RenderError(msg) => write!(f, "Render error: {}", msg),
            CardError::IoError(err) => write!(f, "IO error: {}", err),
            CardError::ImageError(err) => write!(f, "Image error: {}", err),
        }
    }
}

impl std::error::Error for CardError {}

impl From<std::io::Error> for CardError {
    fn from(err: std::io::Error) -> Self {
        CardError::IoError(err)
    }
}

impl From<image::ImageError> for CardError {
    fn from(err: image::ImageError) -> Self {
        CardError::ImageError(err)
    }
}

pub type CardResult<T> = std::result::Result<T, CardError>;

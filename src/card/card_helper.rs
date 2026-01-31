use std::env;
use std::fs;
use crate::card::card_error::{CardError, CardResult};

pub fn load_font() -> CardResult<Vec<u8>> {
    // 1. Check environment variable
    if let Ok(path) = env::var("PLAYER_CARD_FONT") {
        return fs::read(&path)
            .map_err(|e| CardError::FontLoadError(format!("Failed to read font from PLAYER_CARD_FONT {}: {}", path, e)));
    }

    // 2. Common system paths
    let common_paths = vec![
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/liberation/LiberationSans-Regular.ttf",
        "/usr/share/fonts/google-noto/NotoSans-Regular.ttf",
    ];

    for path in common_paths {
        if let Ok(data) = fs::read(path) {
            return Ok(data);
        }
    }

    Err(CardError::FontLoadError("No suitable system font found. Please set PLAYER_CARD_FONT environment variable.".to_string()))
}

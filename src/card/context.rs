use crate::card::card_error::{CardError, CardResult};
use crate::card::card_helper::load_font;
use crate::card::image_data::CardData;
use crate::card::render::RenderCard;
use ab_glyph::FontRef;

pub struct RenderContext {
    #[allow(dead_code)]
    pub width: u32,
    #[allow(dead_code)]
    pub height: u32,
    pub font_data: Vec<u8>,
}

impl RenderContext {
    pub fn new(width: u32, height: u32) -> CardResult<Self> {
        let font_data = load_font()?;
        Ok(Self {
            width,
            height,
            font_data,
        })
    }

    pub fn render(&self, data: &CardData, output_path: &str) -> CardResult<()> {
        let font = FontRef::try_from_slice(&self.font_data)
            .map_err(|e| CardError::FontLoadError(format!("Failed to parse font data: {}", e)))?;

        data.render(self, &font, output_path)
    }
}

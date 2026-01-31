use super::context::RenderContext;
use super::image_data::{
    CardData, ClanData, PersonalProfileData, RankedLeaderboardData, RankedProfileData,
};
use crate::card::card_error::{CardError, CardResult};
use ab_glyph::{FontRef, PxScale};
use image::{ImageBuffer, Rgba, RgbaImage};
use imageproc::drawing::{draw_filled_rect_mut, draw_text_mut};
use imageproc::rect::Rect;

pub trait RenderCard {
    fn render(&self, ctx: &RenderContext, font: &FontRef, output_path: &str) -> CardResult<()>;
}

impl RenderCard for CardData {
    fn render(&self, ctx: &RenderContext, font: &FontRef, output_path: &str) -> CardResult<()> {
        match self {
            CardData::PersonalProfileCard(card) => {
                render_personal_profile(ctx, font, card, output_path)
            }
            CardData::RankedProfileCard(card) => {
                render_ranked_profile(ctx, font, card, output_path)
            }
            CardData::ClanCard(card) => render_clan(ctx, font, card, output_path),
            CardData::RankedLeaderboardCard(card) => {
                render_ranked_leaderboard(ctx, font, card, output_path)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::image_data::{PersonalProfileData, RankedProfileData};
    use crate::card::context::RenderContext;

    #[test]
    fn test_render_personal_profile() {
        let stats = PersonalProfileData {
            player_name: "ceaff".to_string(),
            clan: Some("Fame".to_string()),
            level: 78,
            level_xp: (122090, 174444),
            verified: true,
            followers: 213,
            following: 1,
            created_date: "April 27, 2025".to_string(),
            kills: 10998,
            deaths: 1271,
            kdr: 8.65,
            accuracy: 52.08,
            kr: 9056,
            time_played: "17d 16h".to_string(),
            nukes: 73,
            ranked: Some(RankedProfileData {
                rank_name: "DIAMOND".to_string(),
                mmr: 3074,
                next_rank_mmr: 3300,
                kdr: 1.99,
                win_rate: 70.2,
            }),
        };

        let ctx = RenderContext::new(1100, 700).expect("Failed to create RenderContext");
        let data = CardData::PersonalProfileCard(stats);
        
        // This will save the image to test_card.png in the project root
        let result = ctx.render(&data, "test_card.png");
        assert!(result.is_ok(), "Render failed: {:?}", result.err());
    }
}

// Colors
const BG_DARK: Rgba<u8> = Rgba([25, 25, 35, 255]);
const BG_SIDEBAR: Rgba<u8> = Rgba([18, 18, 22, 255]);
const BG_CARD: Rgba<u8> = Rgba([35, 35, 50, 255]);
const TEXT_WHITE: Rgba<u8> = Rgba([255, 255, 255, 255]);
const TEXT_SECONDARY: Rgba<u8> = Rgba([160, 160, 180, 255]);
const ACCENT_BLUE: Rgba<u8> = Rgba([100, 158, 255, 255]);
const ACCENT_YELLOW: Rgba<u8> = Rgba([255, 193, 7, 255]);
const ACCENT_GREEN: Rgba<u8> = Rgba([76, 175, 80, 255]);
const ACCENT_RED: Rgba<u8> = Rgba([244, 67, 54, 255]);

pub fn render_personal_profile(
    _ctx: &RenderContext,
    font: &FontRef,
    card: &PersonalProfileData,
    output_path: &str,
) -> CardResult<()> {
    // Standard dimensions for this layout
    let width = 1100;
    let height = 700;
    let sidebar_width = 300;

    let mut img: RgbaImage = ImageBuffer::from_pixel(width, height, BG_DARK);

    // 1. Draw Sidebar Background
    draw_filled_rect_mut(&mut img, Rect::at(0, 0).of_size(sidebar_width, height), BG_SIDEBAR);

    // 2. Draw Avatar Placeholder
    draw_avatar(&mut img, sidebar_width);

    // 3. Draw Sidebar Info (Name, Level, Created Date)
    draw_sidebar_info(&mut img, font, card, sidebar_width)?;

    // 4. Draw Main Content Area
    let main_x = sidebar_width + 40;
    let main_width = width - main_x - 40;

    // 4a. Ranked Header
    let ranked_y = 40;
    draw_ranked_header(&mut img, font, &card.ranked, main_x as i32, ranked_y, main_width)?;

    // 4b. Performance Grid
    let grid_y = 200;
    draw_performance_grid(&mut img, font, card, main_x as i32, grid_y, main_width)?;

    // Save the image
    img.save(output_path)
        .map_err(|e| CardError::ImageError(e))?;

    Ok(())
}

fn draw_avatar(img: &mut RgbaImage, sidebar_width: u32) {
    let center_x = sidebar_width / 2;
    let center_y = 120;
    let radius = 80;

    // Draw outer glow/border with Anti-Aliasing
    draw_antialiased_filled_circle(img, (center_x as i32, center_y as i32), radius + 5, ACCENT_YELLOW);
    draw_antialiased_filled_circle(img, (center_x as i32, center_y as i32), radius, BG_DARK);

    // Placeholder avatar circle
    draw_antialiased_filled_circle(img, (center_x as i32, center_y as i32), radius - 10, TEXT_SECONDARY);
}

fn draw_sidebar_info(
    img: &mut RgbaImage,
    font: &FontRef,
    card: &PersonalProfileData,
    sidebar_width: u32,
) -> CardResult<()> {
    let center_x = sidebar_width as i32 / 2;
    let y_start = 250;

    // Name
    let name_scale = PxScale::from(42.0);
    let name_width = get_text_width(font, name_scale, &card.player_name);
    draw_text_mut(img, TEXT_WHITE, center_x - (name_width as i32 / 2), y_start, name_scale, font, &card.player_name);

    // Clan if any
    if let Some(clan) = &card.clan {
        let clan_text = format!("[{}]", clan);
        let clan_scale = PxScale::from(28.0);
        let clan_width = get_text_width(font, clan_scale, &clan_text);
        draw_text_mut(img, ACCENT_YELLOW, center_x - (clan_width as i32 / 2), y_start + 50, clan_scale, font, &clan_text);
    }

    // LVL
    let lvl_text = format!("LVL {}", card.level);
    let lvl_scale = PxScale::from(24.0);
    let lvl_width = get_text_width(font, lvl_scale, &lvl_text);
    draw_text_mut(img, TEXT_SECONDARY, center_x - (lvl_width as i32 / 2), y_start + 100, lvl_scale, font, &lvl_text);

    // XP Bar
    let bar_width = 200;
    let bar_x = center_x - (bar_width / 2);
    let (cur_xp, max_xp) = card.level_xp;
    draw_progress_bar(img, bar_x, y_start + 130, bar_width as u32, 25, cur_xp as f64 / max_xp as f64, ACCENT_YELLOW);
    draw_text_mut(img, TEXT_WHITE, bar_x + 20, y_start + 132, PxScale::from(18.0), font, &format!("{},{} / {},{}", cur_xp/1000, cur_xp%1000, max_xp/1000, max_xp%1000));

    // Followers/Following
    draw_text_mut(img, TEXT_SECONDARY, 40, y_start + 200, PxScale::from(18.0), font, "FOLLOWERS");
    draw_text_mut(img, TEXT_WHITE, 70, y_start + 225, PxScale::from(24.0), font, &card.followers.to_string());

    draw_text_mut(img, TEXT_SECONDARY, 160, y_start + 200, PxScale::from(18.0), font, "FOLLOWING");
    draw_text_mut(img, TEXT_WHITE, 190, y_start + 225, PxScale::from(24.0), font, &card.following.to_string());

    // Created Date
    draw_text_mut(img, TEXT_SECONDARY, center_x - 80, img.height() as i32 - 40, PxScale::from(16.0), font, &format!("Created: {}", card.created_date));

    Ok(())
}

fn draw_ranked_header(
    img: &mut RgbaImage,
    font: &FontRef,
    ranked: &Option<RankedProfileData>,
    x: i32,
    y: i32,
    width: u32,
) -> CardResult<()> {
    let header_height = 140;
    draw_rounded_rect(img, x, y, width, header_height, 10, BG_CARD);

    if let Some(r) = ranked {
        // Rank Icon Placeholder (Square)
        draw_rounded_rect(img, x + 20, y + 20, 100, 100, 5, BG_DARK);
        
        // Rank Name
        draw_text_mut(img, TEXT_SECONDARY, x + 140, y + 25, PxScale::from(20.0), font, "CURRENT RANK");
        draw_text_mut(img, ACCENT_BLUE, x + 140, y + 50, PxScale::from(48.0), font, &r.rank_name);
        draw_text_mut(img, TEXT_WHITE, x + 140, y + 105, PxScale::from(20.0), font, &format!("{} MMR", r.mmr));

        // Stats Highlights
        let stats_x = x + 380;
        draw_text_mut(img, TEXT_SECONDARY, stats_x, y + 25, PxScale::from(18.0), font, "RANKED KDR");
        draw_text_mut(img, TEXT_WHITE, stats_x + 20, y + 50, PxScale::from(32.0), font, &format!("{:.2}", r.kdr));

        draw_text_mut(img, TEXT_SECONDARY, stats_x + 150, y + 25, PxScale::from(18.0), font, "RANKED W/L");
        draw_text_mut(img, TEXT_WHITE, stats_x + 165, y + 50, PxScale::from(32.0), font, &format!("{:.1}%", r.win_rate));
    }

    Ok(())
}

fn draw_performance_grid(
    img: &mut RgbaImage,
    font: &FontRef,
    card: &PersonalProfileData,
    x: i32,
    y: i32,
    width: u32,
) -> CardResult<()> {
    draw_text_mut(img, TEXT_WHITE, x, y, PxScale::from(32.0), font, "PERFORMANCE DETAILS");

    let grid_start_y = y + 50;
    let col_count = 4;
    let item_width = (width - 60) / col_count;
    let item_height = 100;
    let spacing = 20;

    let stats = vec![
        ("KILLS", card.kills.to_string(), ACCENT_YELLOW),
        ("DEATHS", card.deaths.to_string(), ACCENT_YELLOW),
        ("KDR", format!("{:.2}", card.kdr), ACCENT_GREEN),
        ("ACCURACY", format!("{:.1}%", card.accuracy), ACCENT_BLUE),
        ("KR", card.kr.to_string(), ACCENT_YELLOW),
        ("TIME PLAYED", card.time_played.clone(), ACCENT_BLUE),
        ("NUKES", card.nukes.to_string(), ACCENT_RED),
    ];

    for (i, (label, value, accent)) in stats.iter().enumerate() {
        let row = i as i32 / 4;
        let col = i as i32 % 4;
        let item_x = x + col * (item_width as i32 + spacing);
        let item_y = grid_start_y + row * (item_height + spacing);

        draw_accent_box(img, font, item_x, item_y, item_width, item_height as u32, label, value, *accent);
    }

    Ok(())
}

// UI Helpers

fn get_text_width(font: &FontRef, scale: PxScale, text: &str) -> u32 {
    use ab_glyph::{Font, ScaleFont};
    let scaled_font = font.as_scaled(scale);
    let mut width: f32 = 0.0;
    for c in text.chars() {
        let glyph_id = font.glyph_id(c);
        width += scaled_font.h_advance(glyph_id);
    }
    width.ceil() as u32
}

fn draw_rounded_rect(img: &mut RgbaImage, x: i32, y: i32, width: u32, height: u32, _radius: u32, color: Rgba<u8>) {
    // Basic rounded rect implementation using filled rectangles
    // For simplicity, we just draw one main rect.
    draw_filled_rect_mut(img, Rect::at(x, y).of_size(width, height), color);
    
    // In a more complex implementation, we'd add circles at corners.
}

fn draw_antialiased_filled_circle(img: &mut RgbaImage, center: (i32, i32), radius: i32, color: Rgba<u8>) {
    let (cx, cy) = center;
    let r = radius as f32;
    let _r_sq = r * r;
    
    // Determine bounds to avoid checking every pixel
    let min_x = (cx - radius - 1).max(0) as u32;
    let max_x = (cx + radius + 1).min(img.width() as i32 - 1) as u32;
    let min_y = (cy - radius - 1).max(0) as u32;
    let max_y = (cy + radius + 1).min(img.height() as i32 - 1) as u32;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as f32 - cx as f32;
            let dy = y as f32 - cy as f32;
            let dist_sq = dx * dx + dy * dy;
            
            if dist_sq <= (r + 1.0) * (r + 1.0) {
                let dist = dist_sq.sqrt();
                let alpha = if dist <= r - 0.5 {
                    1.0
                } else if dist >= r + 0.5 {
                    0.0
                } else {
                    1.0 - (dist - (r - 0.5))
                };

                if alpha > 0.0 {
                    let pixel = img.get_pixel_mut(x, y);
                    if alpha >= 1.0 {
                        *pixel = color;
                    } else {
                        // Simple alpha blending
                        let bg = *pixel;
                        pixel[0] = (color[0] as f32 * alpha + bg[0] as f32 * (1.0 - alpha)) as u8;
                        pixel[1] = (color[1] as f32 * alpha + bg[1] as f32 * (1.0 - alpha)) as u8;
                        pixel[2] = (color[2] as f32 * alpha + bg[2] as f32 * (1.0 - alpha)) as u8;
                    }
                }
            }
        }
    }
}

fn draw_accent_box(
    img: &mut RgbaImage,
    font: &FontRef,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    label: &str,
    value: &str,
    accent: Rgba<u8>,
) {
    draw_rounded_rect(img, x, y, width, height, 5, BG_CARD);
    // Left accent bar
    draw_filled_rect_mut(img, Rect::at(x, y).of_size(4, height), accent);

    draw_text_mut(img, TEXT_SECONDARY, x + 15, y + 15, PxScale::from(18.0), font, label);
    draw_text_mut(img, TEXT_WHITE, x + 15, y + 45, PxScale::from(32.0), font, value);
}

fn draw_progress_bar(img: &mut RgbaImage, x: i32, y: i32, width: u32, height: u32, progress: f64, color: Rgba<u8>) {
    draw_rounded_rect(img, x, y, width, height, height / 2, BG_DARK);
    let fill_width = (width as f64 * progress.clamp(0.0, 1.0)) as u32;
    if fill_width > 0 {
        draw_filled_rect_mut(img, Rect::at(x, y).of_size(fill_width, height), color);
    }
}

pub fn render_ranked_profile(
    _ctx: &RenderContext,
    _font: &FontRef,
    _card: &RankedProfileData,
    _output_path: &str,
) -> CardResult<()> {
    Ok(())
}

pub fn render_clan(
    _ctx: &RenderContext,
    _font: &FontRef,
    _card: &ClanData,
    _output_path: &str,
) -> CardResult<()> {
    Ok(())
}

pub fn render_ranked_leaderboard(
    _ctx: &RenderContext,
    _font: &FontRef,
    _card: &RankedLeaderboardData,
    _output_path: &str,
) -> CardResult<()> {
    Ok(())
}

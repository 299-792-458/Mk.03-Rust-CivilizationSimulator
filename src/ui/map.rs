use ratatui::{
    prelude::*,
    style::Stylize,
    widgets::Widget,
};

use crate::simulation::{AxialCoord, Nation, ObserverSnapshot};
use crate::ui::{MapOverlay, MODERN_THEME};

const WORLD_ATLAS: &str = r#"
............................................................................................................................
.............%%%%%%%%%%......................%%%%%%%%%%%%%%....................................................%%%%%%%%%%%%..
...........%%%%%%%%%%%%%%...................%%%%%%%%%%%%%%%%%.................................................%%%%%%%%%%%%%..
..........%%%%%%%%%%%%%%%...................%%%%%%%%%%%%%%%%%%................................................%%%%%%%%%%%%%..
..........%%%%%%%%%%%%%%%...................%%%%%%%%%%%%%%%%%%................................................%%%%%%%%%%%%%..
..........%%%%%%%%%%%%%%%...................%%%%%%%%%%%%%%%%%%................................................%%%%%%%%%%%%%..
..........%%%%%%%%%%%%%%%...................%%%%%%%%%%%%%%%%%%................................................%%%%%%%%%%%%%..
..........%%%%%%%%%%%%%%%...................%%%%%%%%%%%%%%%%%%................................................%%%%%%%%%%%%%..
..........%%%%%%%%%%%%%%%...................%%%%%%%%%%%%%%%%%%................................................%%%%%%%%%%%%%..
..........%%%%%%%%%%%%%%%...................%%%%%%%%%%%%%%%%%%................................................%%%%%%%%%%%%%..
..........%%%%%%%%%%%%%%%...................%%%%%%%%%%%%%%%%%%................................................%%%%%%%%%%%%%..
..........%%%%%%%%%%%%%%%...................%%%%%%%%%%%%%%%%%%................................................%%%%%%%%%%%%%..
..........%%%%%%%%%%%%%%%.........%%%%%.....%%%%%%%%%%%%%%%%%%................................................%%%%%%%%%%%%%..
..........%%%%%%%%%%%%%%%........%%%%%%%....%%%%%%%%%%%%%%%%%%..............%%%%%%............................%%%%%%%%%%%%%..
..........%%%%%%%%%%%%%%%.......%%%%%%%%...%%%%%%%%%%%%%%%%%%%............%%%%%%%%...........................%%%%%%%%%%%%%..
..........%%%%%%%%%%%%%%%......%%%%%%%%%...%%%%%%%%%%%%%%%%%%%...........%%%%%%%%%...........................%%%%%%%%%%%%%..
..........%%%%%%%%%%%%%%%.....%%%%%%%%%%...%%%%%%%%%%%%%%%%%%%..........%%%%%%%%%%...........................%%%%%%%%%%%%%..
..........%%%%%%%%%%%%%%%....%%%%%%%%%%%...%%%%%%%%%%%%%%%%%%%.........%%%%%%%%%%%...........................%%%%%%%%%%%%%..
..........%%%%%%%%%%%%%%%...%%%%%%%%%%%%...%%%%%%%%%%%%%%%%%%%........%%%%%%%%%%%%...........................%%%%%%%%%%%%%..
..........%%%%%%%%%%%%%%%...%%%%%%%%%%%%...%%%%%%%%%%%%%%%%%%%........%%%%%%%%%%%%...........................%%%%%%%%%%%%%..
............................................................................................................................
....%%%%%....................................................................................................................
...%%%%%%%%...........................................................................................%%%%%%%%%%%%............
..%%%%%%%%%%%........................................................................................%%%%%%%%%%%%%%%..........
..%%%%%%%%%%%%%.....................................................................................%%%%%%%%%%%%%%%%..........
..%%%%%%%%%%%%%%%...................................................................................%%%%%%%%%%%%%%%%..........
...%%%%%%%%%%%%%%%.....................................................%%%%%%.......................%%%%%%%%%%%%%%%%..........
....%%%%%%%%%%%%%%....................................................%%%%%%%%......................%%%%%%%%%%%%%%%%..........
.....%%%%%%%%%%%%%...................................................%%%%%%%%%......................%%%%%%%%%%%%%%%..........
......%%%%%%%%%%%....................................................%%%%%%%%%.......................%%%%%%%%%%%%%...........
.......%%%%%%%%%.....................................................%%%%%%%%........................%%%%%%%%%%%%............
........%%%%%%.......................................................%%%%%%%.........................%%%%%%%%%%..............
............................................................................................................................
"#;

pub struct MapWidget<'a> {
    pub snapshot: &'a ObserverSnapshot,
    pub overlay: MapOverlay,
    pub selected_hex: Option<AxialCoord>,
    pub focus: Option<Nation>,
}

impl<'a> Widget for MapWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let atlas: Vec<&str> = WORLD_ATLAS.trim_matches('\n').lines().collect();
        if atlas.is_empty() {
            return;
        }
        let atlas_height = atlas.len() as f32;
        let atlas_width = atlas[0].len() as f32;
        let tick = self.snapshot.tick;
        let season = self.snapshot.season.as_str();
        let leader = self.snapshot.science_victory.leader;

        let season_tint = match season {
            "Peak Flame" => Color::Rgb(255, 120, 80),
            "Ember Fall" => Color::DarkGray,
            _ => Color::LightGreen,
        };

        for y in 0..area.height {
            for x in 0..area.width {
                let atlas_x = ((x as f32 / area.width as f32) * atlas_width)
                    .floor()
                    .min(atlas_width - 1.0) as usize;
                let atlas_y = ((y as f32 / area.height as f32) * atlas_height)
                    .floor()
                    .min(atlas_height - 1.0) as usize;
                let ch = atlas[atlas_y].as_bytes()[atlas_x] as char;
                let is_land = ch == '%' || ch == '#' || ch == '█';

                let mut base_char = if is_land { "▓" } else { "·" };
                let mut color = if is_land {
                    season_tint
                } else {
                    Color::Rgb(70, 110, 160)
                };

                let norm_y = y as f32 / area.height as f32;
                let sea_level = self.snapshot.overlay.sea_level;
                let ice_line = self.snapshot.overlay.ice_line;

                match self.overlay {
                    MapOverlay::Ownership => {
                        if !is_land && norm_y > sea_level {
                            base_char = "≈";
                            color = Color::Rgb(50, 90, 140);
                        }
                        if norm_y < ice_line {
                            base_char = "░";
                            color = Color::White;
                        }
                    }
                    MapOverlay::Climate => {
                        let risk =
                            (self.snapshot.science_victory.climate_risk / 140.0).clamp(0.0, 1.0);
                        let heat = (risk * 255.0).round().clamp(0.0, 255.0) as u8;
                        let green = (180.0 - risk * 120.0).max(20.0).min(255.0) as u8;
                        let cool = (150.0 - risk * 120.0).max(30.0).min(255.0) as u8;
                        color = if is_land {
                            Color::Rgb(heat, green, cool)
                        } else {
                            Color::Rgb(40, 100, 180)
                        };
                        if norm_y > sea_level {
                            base_char = "≈";
                            color = Color::Rgb(30, 80, 140);
                        }
                        if norm_y < ice_line {
                            base_char = "░";
                            color = Color::White;
                        }
                    }
                    MapOverlay::Conflict => {
                        let fatigue_norm =
                            (self.snapshot.overlay.war_fatigue / 100.0).clamp(0.0, 1.2);
                        let red = (120.0 + fatigue_norm * 100.0).min(255.0) as u8;
                        let green = (120.0 - fatigue_norm * 60.0).max(20.0) as u8;
                        color = Color::Rgb(red, green, 60);
                        if !is_land {
                            base_char = "·";
                            color = Color::Rgb(60, 90, 120);
                        }
                    }
                }

                if is_land && (tick + x as u64 + y as u64) % 13 == 0 {
                    color = Color::LightYellow;
                }
                if !is_land && tick % 5 == 0 {
                    color = Color::Rgb(90, 140, 200);
                }

                buf.set_string(
                    area.x + x,
                    area.y + y,
                    base_char,
                    Style::default().fg(color),
                );
            }
        }

        let center_x = area.x + area.width / 2;
        let center_y = area.y + area.height / 2;
        let grid = &self.snapshot.grid;
        for (&coord, hex) in &grid.hexes {
            let screen_x = center_x as i32 + coord.q * 2 + coord.r;
            let screen_y = center_y as i32 + coord.r;
            if screen_x < area.x as i32
                || screen_x >= (area.x + area.width) as i32
                || screen_y < area.y as i32
                || screen_y >= (area.y + area.height) as i32
            {
                continue;
            }
            let mut style = Style::default().fg(hex.owner.color());
            if Some(hex.owner) == leader {
                style = style.bold();
            }
            if Some(hex.owner) == self.focus {
                style = style.fg(Color::White).bold();
            }
            let glyph = if self.selected_hex == Some(coord) {
                "◎"
            } else if Some(hex.owner) == leader {
                "◆"
            } else {
                "█"
            };
            buf.set_string(screen_x as u16, screen_y as u16, glyph, style);
        }

        for (&coord, _) in &grid.hexes {
            let screen_x = center_x as i32 + coord.q * 2 + coord.r;
            let screen_y = center_y as i32 + coord.r;
            if screen_x < area.x as i32
                || screen_x >= (area.x + area.width) as i32
                || screen_y < area.y as i32
                || screen_y >= (area.y + area.height) as i32
            {
                continue;
            }
            if self.snapshot.nuclear_hexes.contains(&coord) {
                let glyph = if self.selected_hex == Some(coord) { "◎" } else { "◎" };
                buf.set_string(screen_x as u16, screen_y as u16, glyph, Style::default().fg(Color::Yellow));
            } else if self.snapshot.combat_hexes.contains(&coord) {
                let style = if tick % 2 == 0 {
                    Style::default().fg(Color::White)
                } else {
                    Style::default().fg(Color::Red)
                };
                let glyph = if self.selected_hex == Some(coord) { "◎" } else { "✸" };
                buf.set_string(screen_x as u16, screen_y as u16, glyph, style);
            }
        }
    }
}

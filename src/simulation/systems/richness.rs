use bevy_ecs::prelude::*;

use crate::simulation::{grid::HexGrid, Hex, WorldRichness};

/// Aggregates a simple "resource richness" overlay based on hex ownership diversity.
pub fn richness_overlay_system(
    grid: Res<HexGrid>,
    hexes: Query<&Hex>,
    mut overlay: ResMut<WorldRichness>,
) {
    if grid.hexes.is_empty() {
        overlay.richness = 0.0;
        return;
    }
    let mut counts = std::collections::HashMap::new();
    for entity in grid.hexes.values() {
        if let Ok(hex) = hexes.get(*entity) {
            *counts.entry(hex.owner).or_insert(0usize) += 1;
        }
    }
    let total = grid.hexes.len() as f32;
    let diversity = counts.len() as f32 / 5.0; // 5 nations baseline
    let balance = counts
        .values()
        .map(|c| (*c as f32 / total - 0.2).abs())
        .sum::<f32>();
    overlay.richness = (diversity - balance).clamp(0.0, 1.0);
}

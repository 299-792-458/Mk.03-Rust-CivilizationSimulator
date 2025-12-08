use bevy_ecs::prelude::*;

use crate::simulation::{AxialCoord, ClimateState, HexGrid, grid::Hex};

/// Applies sea level/ice line impact on hex ownership (soft flood tagging) and returns submerged coords.
pub fn flood_system(
    climate: Res<ClimateState>,
    grid: Res<HexGrid>,
    mut query: Query<(&mut Hex, &AxialCoord)>,
) {
    let sea = climate.sea_level;
    let ice = climate.ice_line;
    for (mut hex, coord) in query.iter_mut() {
        // Map axial r to normalized vertical position.
        let norm_y = 0.5 + coord.r as f32 / (grid.radius as f32 * 2.0 + 1.0);
        // Soft flood: elevation under sea raises biome degradation (handled by consumers).
        if hex.elevation < sea + 0.1 && norm_y > sea {
            hex.biome = crate::simulation::Biome::Market; // treated as lowland/coastal
        }
        // Ice creep near poles.
        if norm_y < ice {
            hex.biome = crate::simulation::Biome::Desert; // frozen wasteland stand-in
        }
    }
}

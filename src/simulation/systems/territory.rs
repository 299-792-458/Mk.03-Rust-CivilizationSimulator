use crate::simulation::{AllNationMetrics, AxialCoord, Hex, HexGrid, pentagon_centers};
use bevy_ecs::prelude::*;

fn distance(q1: i32, r1: i32, q2: i32, r2: i32) -> i32 {
    ((q1 - q2).abs() + (q1 + r1 - q2 - r2).abs() + (r1 - r2).abs()) / 2
}

pub fn territory_system(
    metrics: Res<AllNationMetrics>,
    grid: Res<HexGrid>,
    mut query: Query<(&mut Hex, &AxialCoord)>,
) {
    let capitals = pentagon_centers(grid.radius);

    for (mut hex, coord) in query.iter_mut() {
        let mut max_influence = -1.0;
        let mut new_owner = hex.owner;

        for (nation, center) in &capitals {
            let territory = metrics.0.get(nation).map_or(0.0, |m| m.territory);
            let dist = distance(coord.q, coord.r, center.q, center.r);

            // Avoid division by zero and handle distance 0 case
            let influence = if dist == 0 {
                f32::MAX
            } else {
                territory / (dist as f32).powi(2)
            };

            if influence > max_influence {
                max_influence = influence;
                new_owner = *nation;
            }
        }
        hex.owner = new_owner;
    }
}

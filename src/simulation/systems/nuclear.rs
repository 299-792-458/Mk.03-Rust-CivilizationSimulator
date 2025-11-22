use bevy_ecs::prelude::*;
use std::collections::HashSet;

use crate::simulation::{AxialCoord, Hex, NuclearBlasts};

fn axial_distance(a: AxialCoord, b: AxialCoord) -> i32 {
    ((a.q - b.q).abs() + (a.q + a.r - b.q - b.r).abs() + (a.r - b.r).abs()) / 2
}

/// Decays nuclear blast markers over time.
pub fn nuclear_decay_system(mut blasts: ResMut<NuclearBlasts>) {
    let mut to_remove = Vec::new();
    for (coord, timer) in blasts.0.iter_mut() {
        if *timer == 0 {
            to_remove.push(*coord);
        } else {
            *timer = timer.saturating_sub(1);
        }
    }
    for coord in to_remove {
        blasts.0.remove(&coord);
    }
}

/// Marks surrounding hexes when a nuclear strike hits.
pub fn mark_nuclear_blast(
    blasts: &mut NuclearBlasts,
    center: AxialCoord,
    radius: i32,
    duration: u8,
) -> HashSet<AxialCoord> {
    let mut impacted = HashSet::new();
    for (&coord, _) in blasts.0.iter() {
        if axial_distance(coord, center) <= radius {
            impacted.insert(coord);
        }
    }

    // If there are no existing blasts, add new ones within radius
    // We assume world hexes cover the coordinates passed separately.
    for q in -radius..=radius {
        for r in (-radius).max(-q - radius)..=radius.min(-q + radius) {
            let target = AxialCoord::new(center.q + q, center.r + r);
            if axial_distance(center, target) <= radius {
                blasts.0.insert(target, duration);
                impacted.insert(target);
            }
        }
    }
    impacted
}

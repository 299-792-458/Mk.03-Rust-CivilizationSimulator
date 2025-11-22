use bevy_ecs::prelude::*;

use crate::simulation::{AllNationCivState, AllNationMetrics, WorldMetadata, WorldTime};

/// Applies soft seasonal pulses to civ happiness/production and nation surface stats.
pub fn environment_system(
    mut metrics: ResMut<AllNationMetrics>,
    mut civ_state: ResMut<AllNationCivState>,
    time: Res<WorldTime>,
    meta: Res<WorldMetadata>,
) {
    let season = meta.epoch_for_tick(time.tick).1;
    let (morale_shift, yield_shift, risk_shift) = match season {
        "불꽃 절정" => (-4.0, 6.0, 5.0),
        "잿불 내림" => (-2.0, -2.0, 4.0),
        _ => (4.0, 3.0, -2.0),
    };

    for (_, state) in civ_state.0.iter_mut() {
        state.happiness = clamp(state.happiness + morale_shift * 0.1, 0.0, 120.0);
        state.production = clamp(state.production + yield_shift * 0.1, 0.0, 150.0);
        state.stability = clamp(state.stability - risk_shift * 0.05, 0.0, 120.0);
    }

    for (_, nation) in metrics.0.iter_mut() {
        nation.economy = clamp(nation.economy + yield_shift * 0.15, 0.0, 200.0);
        nation.culture = clamp(nation.culture + morale_shift * 0.1, 0.0, 200.0);
        nation.military = clamp(nation.military - risk_shift * 0.1, 0.0, 200.0);
    }
}

fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}

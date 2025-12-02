use bevy_ecs::prelude::*;

use crate::simulation::{
    AllNationCivState, AllNationMetrics, ClimateState, WorldMetadata, WorldTime, WorldRichness,
};

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

/// Simple richness overlay aggregator.
pub fn richness_overlay_system(mut richness: ResMut<WorldRichness>, all_metrics: Res<AllNationMetrics>) {
    let mut total = 0.0;
    let mut count = 0.0;
    for (_nation, metrics) in all_metrics.0.iter() {
        if !metrics.is_destroyed {
            total += metrics.economy * 0.5 + metrics.science * 0.5;
            count += 1.0;
        }
    }
    richness.richness = if count > 0.0 { (total / count) / 100.0 } else { 0.0 };
    let value = richness.richness;
    push_history(&mut richness.history, value * 100.0);
}

/// Applies climate penalties/bonuses to nation metrics based on global climate state.
pub fn climate_impact_system(
    climate: Res<ClimateState>,
    mut metrics: ResMut<AllNationMetrics>,
) {
    let risk = climate.climate_risk;
    let biodiversity = climate.biodiversity;
    for (_, m) in metrics.0.iter_mut() {
        if m.is_destroyed {
            continue;
        }
        // Productivity penalty from risk
        let penalty = (risk * 0.08).min(25.0);
        m.economy = (m.economy - penalty).max(0.0);
        // Science penalty but innovation spur if biodiversity is still healthy
        let science_penalty = penalty * 0.4;
        m.science = (m.science - science_penalty).max(0.0);
        if biodiversity > 30.0 {
            m.research_stock += biodiversity * 0.02;
        }
    }
}

fn push_history(history: &mut Vec<f32>, value: f32) {
    history.push(value);
    if history.len() > 256 {
        history.remove(0);
    }
}

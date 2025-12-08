use bevy_ecs::prelude::*;

use crate::simulation::{
    AllNationCivState, AllNationMetrics, ClimateState, WorldMetadata, WorldRichness, WorldTime,
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
        "Sunburst Peak" => (-4.0, 6.0, 5.0),
        "Ashfall" => (-2.0, -2.0, 4.0),
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
pub fn richness_overlay_system(
    mut richness: ResMut<WorldRichness>,
    all_metrics: Res<AllNationMetrics>,
) {
    let mut total = 0.0;
    let mut count = 0.0;
    for (_nation, metrics) in all_metrics.0.iter() {
        if !metrics.is_destroyed {
            total += metrics.economy * 0.5 + metrics.science * 0.5;
            count += 1.0;
        }
    }
    richness.richness = if count > 0.0 {
        (total / count) / 100.0
    } else {
        0.0
    };
    let value = richness.richness;
    push_history(&mut richness.history, value * 100.0);
}

/// Applies climate penalties/bonuses to nation metrics based on global climate state.
pub fn climate_impact_system(
    climate: Res<ClimateState>,
    mut metrics: ResMut<AllNationMetrics>,
    mut civ_state: ResMut<AllNationCivState>,
) {
    let risk = climate.climate_risk;
    let biodiversity = climate.biodiversity;
    let sea = climate.sea_level;
    let ice = climate.ice_line;
    // Sea level eats coastal productivity; ice line expansion reduces arable land.
    let land_loss_factor = (sea * 0.8 + (1.0 - ice) * 0.2).clamp(0.0, 1.0);
    let habitability = (1.0 - land_loss_factor).clamp(0.0, 1.0);
    for (nation, m) in metrics.0.iter_mut() {
        if m.is_destroyed {
            continue;
        }
        // Productivity penalty from risk
        let penalty = (risk * 0.08).min(25.0);
        m.economy = (m.economy - penalty).max(0.0);
        // Coastal loss reduces territory and economy softly.
        m.territory = (m.territory * (0.98 - sea * 0.25)).max(5.0);
        m.economy *= (0.99 - sea * 0.15).max(0.6);
        m.military *= (0.995 - land_loss_factor * 0.15).max(0.5);
        // Habitat and food stress
        m.population =
            (m.population as f32 * (0.999 - land_loss_factor * 0.08)).max(10_000.0) as u64;
        // Science penalty but innovation spur if biodiversity is still healthy
        let science_penalty = penalty * 0.4;
        m.science = (m.science - science_penalty).max(0.0);
        if biodiversity > 30.0 {
            m.research_stock += biodiversity * 0.02;
        }
        // Biodiversity collapse slows culture/diplomacy growth
        if biodiversity < 40.0 {
            m.culture = (m.culture - 0.6).max(0.0);
            m.diplomacy = (m.diplomacy - 0.4).max(0.0);
        }
        // Recovery when seas stabilize and ice retreats
        if habitability > 0.8 && risk < 25.0 {
            m.economy = (m.economy + 0.8).min(220.0);
            m.territory = (m.territory + 0.2).min(120.0);
        }

        if let Some(civ) = civ_state.0.get_mut(nation) {
            // Happiness drops with risk and land loss; production follows habitability.
            civ.happiness = clamp(
                civ.happiness - risk * 0.04 - land_loss_factor * 12.0 + habitability * 4.0,
                5.0,
                130.0,
            );
            civ.production = clamp(
                civ.production * (0.995 - sea * 0.08) + habitability * 1.2,
                0.0,
                160.0,
            );
            civ.stability = clamp(
                civ.stability - risk * 0.03 - land_loss_factor * 8.0,
                0.0,
                140.0,
            );
        }
    }
}

fn push_history(history: &mut Vec<f32>, value: f32) {
    history.push(value);
    if history.len() > 512 {
        let mut i = 0;
        history.retain(|_| {
            let keep = i % 2 == 0;
            i += 1;
            keep
        });
    }
}

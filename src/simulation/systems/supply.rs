use bevy_ecs::prelude::*;

use crate::simulation::{AllNationMetrics, ClimateState, SupplyState, WorldRichness};

/// Aggregates coarse supply chains (food/energy/rare). Penalizes economy/pop when deficits persist.
pub fn supply_chain_system(
    mut supply: ResMut<SupplyState>,
    metrics: Res<AllNationMetrics>,
    climate: Res<ClimateState>,
    richness: Res<WorldRichness>,
) {
    let mut food = 0.0;
    let mut energy = 0.0;
    let mut rare = 0.0;
    for (_nation, m) in metrics.0.iter() {
        if m.is_destroyed {
            continue;
        }
        food += m.territory * 0.5 + m.culture * 0.2;
        energy += m.economy * 0.4 + m.territory * 0.1;
        rare += m.science * 0.3 + m.economy * 0.2;
    }
    // Climate pressure reduces effective supply; richness boosts.
    let climate_drag =
        (climate.sea_level * 0.6 + (climate.climate_risk / 100.0) * 0.4).clamp(0.0, 1.2);
    let boost = (richness.richness * 0.5).clamp(0.0, 0.6);
    supply.food = (food * (1.0 - climate_drag) * (1.0 + boost)).max(0.0);
    supply.energy = (energy * (1.0 - climate_drag * 0.7) * (1.0 + boost)).max(0.0);
    supply.rare = (rare * (1.0 - climate_drag * 0.5) * (1.0 + boost)).max(0.0);

    // Simple deficits if any channel is below threshold per nation count.
    let nations = metrics
        .0
        .values()
        .filter(|m| !m.is_destroyed)
        .count()
        .max(1) as f32;
    let demand = nations * 80.0;
    let deficit =
        supply.food < demand || supply.energy < demand * 0.8 || supply.rare < demand * 0.6;
    if deficit {
        supply.deficit_ticks += 1;
    } else if supply.deficit_ticks > 0 {
        supply.deficit_ticks -= 1;
    }

    let snapshot = (supply.food, supply.energy, supply.rare);
    supply.history.push(snapshot);
    if supply.history.len() > 256 {
        let excess = supply.history.len() - 256;
        supply.history.drain(0..excess);
    }
}

/// Applies supply deficits to nation metrics and civ state.
pub fn supply_impact_system(
    supply: Res<SupplyState>,
    mut metrics: ResMut<AllNationMetrics>,
    mut civ: ResMut<crate::simulation::AllNationCivState>,
) {
    if supply.deficit_ticks == 0 {
        return;
    }
    let deficit_strength = (supply.deficit_ticks as f32 * 0.05).min(1.0);
    for (nation, m) in metrics.0.iter_mut() {
        if m.is_destroyed {
            continue;
        }
        m.economy = (m.economy * (1.0 - 0.02 * deficit_strength)).max(5.0);
        m.population = (m.population as f32 * (1.0 - 0.001 * deficit_strength)).max(5_000.0) as u64;
        m.military = (m.military * (1.0 - 0.015 * deficit_strength)).max(5.0);
        if let Some(cstate) = civ.0.get_mut(nation) {
            cstate.happiness = (cstate.happiness - 1.5 * deficit_strength).max(0.0);
            cstate.production = (cstate.production * (1.0 - 0.01 * deficit_strength)).max(0.0);
        }
    }
}

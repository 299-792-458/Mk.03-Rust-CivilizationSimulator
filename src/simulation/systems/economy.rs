//! Trade and economy placeholder system.

use bevy_ecs::prelude::*;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::simulation::{
    AllNationMetrics, Behavior, BehaviorState, BlocKind, WorldBlocs, Identity, Inventory, Position,
    WorldMetadata, WorldTime,
};

fn season_trade_modifier(season: &str) -> f32 {
    match season {
        "꽃피움 계절" => 1.1,
        "불꽃 절정" => 1.0,
        "잿불 내림" => 0.95,
        _ => 1.0,
    }
}

fn season_gather_modifier(season: &str) -> f32 {
    match season {
        "꽃피움 계절" => 1.25,
        "불꽃 절정" => 1.05,
        "잿불 내림" => 0.9,
        _ => 1.0,
    }
}

fn segment_trade_modifier(segment: &str) -> f32 {
    match segment {
        "한낮" => 1.2,
        "해질녘" => 0.85,
        _ => 1.0,
    }
}

fn upkeep_penalty(base: f32, upkeep: f32) -> f32 {
    base * upkeep.max(0.5)
}

pub fn economy_system(
    mut query: Query<(&Identity, &Position, &Behavior, &mut Inventory)>,
    mut all_metrics: ResMut<AllNationMetrics>,
    blocs: Res<WorldBlocs>,
    world_meta: Res<WorldMetadata>,
    time: Res<WorldTime>,
) {
    let (segment, season) = world_meta.epoch_for_tick(time.tick);

    // First, handle nation-level economic updates (upkeep, investment, growth, decay)
    for (nation_key, metrics) in all_metrics.0.iter_mut() {
        // 1. Military Upkeep
        let military_upkeep = metrics.military * 0.05;
        metrics.economy -= military_upkeep;

        // 2. Other Metrics Upkeep (they cost a little bit of economy)
        let science_upkeep = metrics.science * 0.02;
        let culture_upkeep = metrics.culture * 0.01;
        let diplomacy_upkeep = metrics.diplomacy * 0.02;
        metrics.economy -= science_upkeep + culture_upkeep + diplomacy_upkeep;

        // If economy is negative after upkeep, it hurts the military
        if metrics.economy < 0.0 {
            metrics.military += metrics.economy * 0.2; // Negative economy reduces military
            metrics.economy = 0.0;
        }

        // 3. Investment and Growth based on Economy
        if metrics.economy > 70.0 {
            // High Economy Boosts
            metrics.military += 0.5;
            metrics.science += 0.4;
            metrics.diplomacy += 0.3;
            metrics.economy -= 5.0; // Investment cost
        } else if metrics.economy > 40.0 {
            // Medium Economy
            metrics.military += 0.1;
            metrics.science += 0.15;
            metrics.culture += 0.2;
            metrics.religion += 0.1;
            metrics.diplomacy += 0.1;
            metrics.economy -= 1.0;
        }

        // 4. General Decay
        metrics.science *= 0.999;
        metrics.culture *= 0.998;
        metrics.diplomacy *= 0.999;
        metrics.religion *= 0.9995;

        // Bloc influences
        if let Some(research) = blocs.blocs.get(&BlocKind::ResearchPact) {
            if research.members.contains(nation_key) {
                metrics.science += research.strength * 0.3;
                metrics.diplomacy += research.strength * 0.15;
            }
        }
        if let Some(sanction) = blocs.blocs.get(&BlocKind::Sanction) {
            if sanction.members.contains(nation_key) {
                metrics.economy -= sanction.strength * 0.6;
                metrics.science -= sanction.strength * 0.25;
            }
        }
    }

    // Second, handle individual NPC actions contributing to economy
    for (identity, position, behavior, mut inventory) in &mut query {
        let nation = identity.nation;
        let metrics = all_metrics.0.get_mut(&nation).unwrap();

        let biome = position.biome;
        let faction = identity.faction;

        let base_trade_yield = 6.0;
        let base_gather_value = 3.0;
        let trade_multiplier = world_meta.biome_trade_opportunity(biome)
            * world_meta.faction_trade_yield(faction)
            * season_trade_modifier(season)
            * segment_trade_modifier(segment);
        let resource_multiplier =
            world_meta.biome_resource_abundance(biome) * season_gather_modifier(season);
        let risk_factor =
            world_meta.biome_risk_factor(biome) / world_meta.faction_volatility_resistance(faction);

        let upkeep = world_meta.faction_upkeep_burden(faction);

        let mut rng = SmallRng::seed_from_u64(
            time.tick
                .wrapping_mul(131)
                .wrapping_add(identity.id * 7)
                .wrapping_mul(59),
        );

        if matches!(behavior.state, BehaviorState::Trade) {
            let volatility: f32 = rng.gen_range(-2.0..2.0) * risk_factor;
            let trade_gain =
                base_trade_yield * trade_multiplier - upkeep_penalty(0.75, upkeep) + volatility;
            inventory.currency =
                (inventory.currency + trade_gain.max(-inventory.currency)).max(0.0);

            // NPC actions now contribute less directly, but still add to the economy
            metrics.economy += trade_gain * 0.05;
        }

        if matches!(behavior.state, BehaviorState::Gather) {
            let gather_gain = base_gather_value * resource_multiplier
                - upkeep_penalty(0.35, upkeep)
                + rng.gen_range(0.0..2.0);
            inventory.currency += gather_gain.max(0.0);

            metrics.economy += gather_gain * 0.03;
        }
    }

    // Finally, clamp all metrics to a 0-100 range
    for metrics in all_metrics.0.values_mut() {
        metrics.economy = metrics.economy.clamp(0.0, 100.0);
        metrics.science = metrics.science.clamp(0.0, 100.0);
        metrics.culture = metrics.culture.clamp(0.0, 100.0);
        metrics.diplomacy = metrics.diplomacy.clamp(0.0, 100.0);
        metrics.religion = metrics.religion.clamp(0.0, 100.0);
        metrics.military = metrics.military.clamp(0.0, 100.0);
    }
}

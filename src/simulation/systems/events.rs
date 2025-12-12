//! Generates structured world events tied to metadata influences.

use bevy_ecs::prelude::*;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::simulation::{
    AllNationMetrics, Attributes, Behavior, BehaviorState, EventActor, Inventory, Position,
    WorldEvent, WorldEventLog, WorldMetadata, WorldTime, behavior_label, faction_label,
};

pub fn event_generation_system(
    time: Res<WorldTime>,
    world_meta: Res<WorldMetadata>,
    mut event_log: ResMut<WorldEventLog>,
    mut all_metrics: ResMut<AllNationMetrics>,
    query: Query<(
        &crate::simulation::Identity,
        &Behavior,
        &Position,
        &Inventory,
        &Attributes,
    )>,
) {
    let tick = time.tick;
    let (epoch, season) = world_meta.epoch_for_tick(tick);
    let mut rng = SmallRng::seed_from_u64(tick.wrapping_mul(421) + 17);

    // Trade event sampling
    let mut trade_choice: Option<(crate::simulation::Identity, Position, f32, BehaviorState)> =
        None;
    let mut trade_count = 0;

    for (identity, behavior, position, inventory, _) in &query {
        if !matches!(behavior.state, BehaviorState::Trade) {
            continue;
        }
        trade_count += 1;
        if rng.gen_range(0..trade_count) == 0 {
            trade_choice = Some((
                (*identity).clone(),
                *position,
                inventory.currency,
                behavior.state,
            ));
        }
    }

    if let Some((identity, position, currency, behavior_state)) = trade_choice {
        let biome_profile = world_meta.biomes.get(&position.biome);
        let focus = biome_profile
            .and_then(|meta| {
                if meta.resource_profile.is_empty() {
                    None
                } else {
                    Some(
                        meta.resource_profile[rng.gen_range(0..meta.resource_profile.len())]
                            .to_string(),
                    )
                }
            })
            .unwrap_or_else(|| "General Goods".to_string());

        let pressure = world_meta
            .faction_profile(identity.faction)
            .and_then(|f| {
                if f.influence_vectors.is_empty() {
                    None
                } else {
                    Some(
                        f.influence_vectors[rng.gen_range(0..f.influence_vectors.len())]
                            .to_string(),
                    )
                }
            })
            .unwrap_or_else(|| "Local Demand Index".to_string());

        let actor = EventActor {
            id: identity.id,
            name: identity.name.clone(),
            nation: identity.nation,
            faction: identity.faction,
            faction_label: faction_label(identity.faction).to_string(),
            biome: position.biome,
            biome_label: biome_profile
                .map(|meta| meta.label.to_string())
                .unwrap_or_else(|| format!("{:?}", position.biome)),
            behavior_hint: behavior_state,
            behavior_hint_label: behavior_label(behavior_state).to_string(),
        };

        let market_label = biome_profile
            .map(|meta| meta.label)
            .unwrap_or("Unknown Market");
        let trade_summary = format!(
            "{} brokers {} flow at {} (liquidity {:.1})",
            actor.name, focus, market_label, currency
        );

        event_log.push(WorldEvent::trade(
            tick,
            epoch,
            season,
            actor,
            trade_summary,
            pressure,
        ));
    }

    // Social event sampling (Idle or Rest)
    let mut social_choice: Option<(
        crate::simulation::Identity,
        Position,
        Attributes,
        BehaviorState,
    )> = None;
    let mut social_count = 0;

    for (identity, behavior, position, _, attributes) in &query {
        if !matches!(behavior.state, BehaviorState::Idle | BehaviorState::Rest) {
            continue;
        }
        social_count += 1;
        if rng.gen_range(0..social_count) == 0 {
            social_choice = Some((
                (*identity).clone(),
                *position,
                (*attributes).clone(),
                behavior.state,
            ));
        }
    }

    if let Some((identity, position, attributes, behavior_state)) = social_choice {
        let biome_profile = world_meta.biomes.get(&position.biome);
        let gathering_theme = biome_profile
            .and_then(|meta| {
                if meta.tensions.is_empty() {
                    None
                } else {
                    Some(meta.tensions[rng.gen_range(0..meta.tensions.len())].to_string())
                }
            })
            .unwrap_or_else(|| "Story circle".to_string());

        let cohesion_level = if attributes.fame >= 60.0 {
            "Legendary crowd"
        } else if attributes.fame >= 35.0 {
            "Packed hall"
        } else if attributes.fame >= 15.0 {
            "Cozy group"
        } else {
            "Small circle"
        }
        .to_string();

        let actor = EventActor {
            id: identity.id,
            name: identity.name.clone(),
            nation: identity.nation,
            faction: identity.faction,
            faction_label: faction_label(identity.faction).to_string(),
            biome: position.biome,
            biome_label: biome_profile
                .map(|meta| meta.label.to_string())
                .unwrap_or_else(|| format!("{:?}", position.biome)),
            behavior_hint: behavior_state,
            behavior_hint_label: behavior_label(behavior_state).to_string(),
        };

        event_log.push(WorldEvent::social(
            tick,
            epoch,
            season,
            actor,
            gathering_theme,
            cohesion_level,
        ));
    }

    // Macro shock event (pulse each tick)
    let stressors = &world_meta.economy.stressors;
    let catalysts = &world_meta.economy.catalysts;
    let circulation = &world_meta.economy.circulation_cycle;

    if !(stressors.is_empty() || catalysts.is_empty() || circulation.is_empty()) {
        let stressor = stressors[(tick as usize) % stressors.len()].to_string();
        let catalyst =
            catalysts[((tick as usize) + circulation.len()) % catalysts.len()].to_string();
        let circulation_stage =
            circulation[(tick as usize + catalysts.len()) % circulation.len()].to_string();

        let casualties = if stressor.contains("plague") || stressor.contains("disease") {
            // Plagues now hit much harder — large, sporadic die-off events.
            Some(rng.gen_range(50_000..750_000))
        } else {
            None
        };

        if let Some(total) = casualties {
            let per = total / (all_metrics.0.len() as u64).max(1);
            for metrics in all_metrics.0.values_mut() {
                if !metrics.is_destroyed {
                    metrics.population = metrics.population.saturating_sub(per);
                }
            }
        }

        let impact = format!(
            "{} is pressuring the world | Catalyst: {} | Phase {}",
            stressor, catalyst, circulation_stage
        );

        event_log.push(WorldEvent::macro_shock(
            tick, epoch, season, stressor, catalyst, impact, casualties,
        ));
    }

    // Catastrophic pulse: rare megaplague / cataclysm that slashes populations.
    if rng.gen_bool(0.05) {
        let severity = rng.gen_range(0.12..0.35); // 12–35% population loss
        let mut total_casualties = 0u64;
        for metrics in all_metrics.0.values_mut() {
            if metrics.is_destroyed {
                continue;
            }
            let loss = (metrics.population as f32 * severity) as u64;
            total_casualties = total_casualties.saturating_add(loss);
            metrics.population = metrics.population.saturating_sub(loss).max(10_000);
        }
        let catalyst = if rng.gen_bool(0.5) {
            "Mutating pandemic sweeps megacities"
        } else {
            "Cascade disaster chain (quake/tsunami/famine)"
        };
        event_log.push(WorldEvent::macro_shock(
            tick,
            epoch,
            season,
            "Global die-off".to_string(),
            catalyst.to_string(),
            format!(
                "Pop crash {:.0}% | Infrastructure disruption severe",
                severity * 100.0
            ),
            Some(total_casualties),
        ));
    }

    // Omega-level disaster: meteor strike, gamma-ray burst, or supervolcano chain.
    if rng.gen_bool(0.01) {
        let roll = rng.gen_range(0..3);
        let (stressor, catalyst, severity) = match roll {
            0 => (
                "Planet-killer meteor strike",
                "Impact winter + ejecta plume",
                rng.gen_range(0.90..0.999), // up to 99.9% loss
            ),
            1 => (
                "Gamma-ray burst exposure",
                "Atmospheric stripping / radiation storm",
                0.999, // force ~99.9% loss
            ),
            _ => (
                "Supervolcano cascade",
                "Ash cloud + famine + climate dive",
                rng.gen_range(0.70..0.92), // 70–92% loss
            ),
        };

        let mut total_casualties = 0u64;
        for metrics in all_metrics.0.values_mut() {
            if metrics.is_destroyed {
                continue;
            }
            let loss = ((metrics.population as f32) * severity) as u64;
            total_casualties = total_casualties.saturating_add(loss);
            metrics.population = metrics.population.saturating_sub(loss).max(1_000);
        }

        event_log.push(WorldEvent::macro_shock(
            tick,
            epoch,
            season,
            stressor.to_string(),
            catalyst.to_string(),
            format!(
                "Omega event — population crash {:.1}% | infrastructure collapse",
                severity * 100.0
            ),
            Some(total_casualties),
        ));
    }
}

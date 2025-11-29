use crate::simulation::{
    AllNationCivState, AllNationMetrics, Hex, Nation, WeaponTier, WorldTime,
    components::{Combatants, InCombat},
    grid::AxialCoord,
};
use bevy_ecs::prelude::*;
use rand::prelude::SliceRandom;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::collections::{HashMap, HashSet};

struct BattleRequest {
    nation_a: Nation,
    nation_b: Nation,
}

fn apply_war_science_penalty(metrics: &mut crate::simulation::NationMetrics, casualties: u64) {
    if metrics.population == 0 {
        return;
    }
    let population_before = metrics.population.max(1) as f32;
    let casualty_ratio = (casualties as f32 / population_before).clamp(0.0, 0.75);
    let science_loss = metrics.science * casualty_ratio * 0.6 + casualty_ratio * 12.0;
    metrics.science = (metrics.science - science_loss).max(0.0);
    metrics.research_stock *= (1.0 - casualty_ratio * 0.65).clamp(0.0, 1.0);

    // War undermines economic surplus and production base that feeds science.
    let econ_loss = metrics.economy * casualty_ratio * 0.45 + casualty_ratio * 8.0;
    metrics.economy = (metrics.economy - econ_loss).max(0.0);
    metrics.culture = (metrics.culture - casualty_ratio * 6.0).max(0.0);
}

// System to clean up finished combat encounters
pub fn combat_cleanup_system(mut commands: Commands, mut query: Query<(Entity, &mut InCombat)>) {
    for (entity, mut in_combat) in query.iter_mut() {
        in_combat.ticks_remaining = in_combat.ticks_remaining.saturating_sub(1);
        if in_combat.ticks_remaining == 0 {
            commands.entity(entity).remove::<InCombat>();
            commands.entity(entity).remove::<Combatants>();
        }
    }
}

pub fn warfare_system(
    mut commands: Commands,
    mut all_metrics: ResMut<AllNationMetrics>,
    mut civ_state: ResMut<AllNationCivState>,
    mut blasts: ResMut<crate::simulation::NuclearBlasts>,
    time: Res<WorldTime>,
    mut event_log: ResMut<crate::simulation::WorldEventLog>,
    world_meta: Res<crate::simulation::WorldMetadata>,
    science_victory: Res<crate::simulation::ScienceVictory>,
    hex_query: Query<(Entity, &Hex, &AxialCoord)>,
) {
    if science_victory.finished {
        return;
    }
    let mut rng = SmallRng::seed_from_u64(time.tick.wrapping_mul(257));
    let mut battle_requests = Vec::new();

    // 1. Identify potential battles
    let nations: Vec<Nation> = all_metrics.0.keys().cloned().collect();
    for i in 0..nations.len() {
        for j in (i + 1)..nations.len() {
            let nation_a_key = nations[i];
            let nation_b_key = nations[j];

            let (metrics_a, metrics_b) = (
                all_metrics.0.get(&nation_a_key).unwrap(),
                all_metrics.0.get(&nation_b_key).unwrap(),
            );

            if metrics_a.is_destroyed
                || metrics_b.is_destroyed
                || metrics_a.military <= 1.0
                || metrics_b.military <= 1.0
            {
                continue;
            }

            // --- War Prevention Logic ---
            // Higher diplomacy, culture, and religion reduce the chance of war.
            let peace_factor = (metrics_a.diplomacy + metrics_b.diplomacy)
                + (metrics_a.culture + metrics_b.culture) * 0.5
                + (metrics_a.religion + metrics_b.religion) * 0.5;

            // Base probability of war is 20%, reduced by the peace factor.
            let war_prob = (0.2 - peace_factor * 0.001).max(0.01);

            if rng.gen_bool(war_prob as f64) {
                battle_requests.push(BattleRequest {
                    nation_a: nation_a_key,
                    nation_b: nation_b_key,
                });
            }
        }
    }

    let (epoch, season) = world_meta.epoch_for_tick(time.tick);
    let nation_hexes: HashMap<Nation, HashSet<AxialCoord>> = {
        let mut map: HashMap<Nation, HashSet<AxialCoord>> = HashMap::new();
        for (_, hex, coord) in hex_query.iter() {
            map.entry(hex.owner).or_default().insert(*coord);
        }
        map
    };

    // 2. Process battles
    for request in battle_requests {
        let (winner, loser) = {
            let metrics_a = all_metrics.0.get(&request.nation_a).unwrap();
            let metrics_b = all_metrics.0.get(&request.nation_b).unwrap();

            // --- Battle Outcome Logic ---
            // Science acts as a multiplier for military strength.
            let military_a_effective = metrics_a.military
                * (1.0 + metrics_a.science / 100.0)
                * metrics_a.weapon_tier.combat_multiplier();
            let military_b_effective = metrics_b.military
                * (1.0 + metrics_b.science / 100.0)
                * metrics_b.weapon_tier.combat_multiplier();

            let roll_a = rng.gen_range(0.0..1.0) * military_a_effective;
            let roll_b = rng.gen_range(0.0..1.0) * military_b_effective;

            if roll_a > roll_b {
                (request.nation_a, request.nation_b)
            } else {
                (request.nation_b, request.nation_a)
            }
        };

        // Update metrics
        let territory_change = 0.5;
        let military_loss = 2.0;
        let base_force = {
            let metrics_a = all_metrics.0.get(&request.nation_a).unwrap();
            let metrics_b = all_metrics.0.get(&request.nation_b).unwrap();
            (metrics_a.military + metrics_b.military).max(1.0)
        };
        let mut raw_casualties = (base_force * rng.gen_range(6000.0..12000.0)) as u64;
        let (metrics_a, metrics_b) = (
            all_metrics.0.get(&request.nation_a).unwrap(),
            all_metrics.0.get(&request.nation_b).unwrap(),
        );
        let nuclear_a = matches!(metrics_a.weapon_tier, WeaponTier::NuclearArsenal);
        let nuclear_b = matches!(metrics_b.weapon_tier, WeaponTier::NuclearArsenal);
        let late_game = time.tick > 800 || (nuclear_a && nuclear_b);
        let nuke_probability = if late_game { 0.9 } else { 0.6 };
        let nuclear = if nuclear_a || nuclear_b {
            // Escalate to nuclear more often late game; if both have nukes, high mutual retaliation chance.
            let escalate = rng.gen_bool(nuke_probability);
            if escalate {
                let nuke_count = if late_game {
                    rng.gen_range(200..1001)
                } else {
                    rng.gen_range(3..13)
                };
                let mutual = nuclear_a && nuclear_b && rng.gen_bool(0.8);
                let multiplier = if mutual {
                    5.0 * nuke_count as f32
                } else {
                    5.0 * (nuke_count as f32 * 0.5)
                };
                raw_casualties = (raw_casualties as f32 * multiplier) as u64;
            }
            escalate
        } else {
            false
        };
        let total_casualties = raw_casualties.clamp(15_000, 2_000_000);
        let winner_casualties = (total_casualties as f32 * 0.35) as u64;
        let loser_casualties = total_casualties.saturating_sub(winner_casualties);

        if let Some(winner_metrics) = all_metrics.0.get_mut(&winner) {
            winner_metrics.territory += territory_change;
            winner_metrics.military -= military_loss;
            winner_metrics.territory = winner_metrics.territory.max(0.0);
            winner_metrics.military = winner_metrics.military.max(0.0);
            apply_war_science_penalty(winner_metrics, winner_casualties);
            winner_metrics.population = winner_metrics.population.saturating_sub(winner_casualties);
            // Post-war rebuilding boosts diplomacy/culture for victors that avoid annihilation
            winner_metrics.diplomacy =
                (winner_metrics.diplomacy + (territory_change * 0.8)).min(100.0);
            winner_metrics.culture = (winner_metrics.culture + 0.6).min(100.0);
        }

        if let Some(loser_metrics) = all_metrics.0.get_mut(&loser) {
            loser_metrics.territory -= territory_change;
            loser_metrics.military -= military_loss;
            loser_metrics.territory = loser_metrics.territory.max(0.0);
            loser_metrics.military = loser_metrics.military.max(0.0);
            apply_war_science_penalty(loser_metrics, loser_casualties);
            loser_metrics.population = loser_metrics.population.saturating_sub(loser_casualties);

            if loser_metrics.territory <= 0.0 {
                loser_metrics.is_destroyed = true;
                loser_metrics.population = 0;
                if let Some(civ) = civ_state.0.get_mut(&loser) {
                    civ.cities = 0;
                    civ.production = 0.0;
                    civ.happiness = 0.0;
                    civ.stability = 0.0;
                }
            }
        }

        // Log the event
        event_log.push(crate::simulation::WorldEvent::warfare(
            time.tick,
            epoch,
            season,
            winner,
            loser,
            territory_change,
            total_casualties,
            nuclear,
        ));

        if nuclear {
            if let Some((_, _, center)) = hex_query.iter().find(|(_, h, _)| h.owner == loser) {
                let radius = 2;
                let duration = 8;
                let loser_hexes = nation_hexes.get(&loser).cloned().unwrap_or_default();
                let mut targets: Vec<AxialCoord> = loser_hexes.into_iter().collect();
                targets.shuffle(&mut rng);
                for blast_center in targets.into_iter().take(20).chain(std::iter::once(*center)) {
                    for dq in -radius..=radius {
                        for dr in (-radius).max(-dq - radius)..=radius.min(-dq + radius) {
                            let target = AxialCoord::new(blast_center.q + dq, blast_center.r + dr);
                            blasts.0.insert(target, duration);
                        }
                    }
                }
            }
        }

        // 3. Find border hexes and mark them as in combat
        let mut border_hex_entities = HashSet::new();
        let loser_hexes = nation_hexes.get(&loser).cloned().unwrap_or_default();

        for (entity, hex, coord) in hex_query.iter() {
            if hex.owner == winner {
                for neighbor_coord in coord.neighbors() {
                    if loser_hexes.contains(&neighbor_coord) {
                        border_hex_entities.insert(entity);
                    }
                }
            }
        }

        for entity in border_hex_entities {
            commands.entity(entity).insert((
                InCombat { ticks_remaining: 5 },
                Combatants {
                    nation_a: winner,
                    nation_b: loser,
                },
            ));
        }
    }
}

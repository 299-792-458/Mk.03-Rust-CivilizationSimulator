use bevy_ecs::prelude::*;

use crate::simulation::{
    AllNationMetrics, BehaviorState, Biome, Faction, IdeologyMatrix, Nation, WorldEvent,
    WorldEventKind, WorldEventLog, WorldTime,
};

/// Spreads ideology leaning and cohesion through adjacency of power (economy/culture)
/// and recent conflict/trade events. Keeps volatility as a rebellion pressure.
pub fn ideology_system(
    mut matrix: ResMut<IdeologyMatrix>,
    metrics: Res<AllNationMetrics>,
    time: Res<WorldTime>,
    mut log: ResMut<WorldEventLog>,
) {
    // Seed defaults if empty
    for nation in metrics.0.keys() {
        matrix.leaning.entry(*nation).or_insert(50.0);
        matrix.cohesion.entry(*nation).or_insert(55.0);
        matrix.volatility.entry(*nation).or_insert(20.0);
    }

    // Influence: stronger economies and cultures pull neighbors in ideology space
    let mut updates: Vec<(Nation, f32)> = Vec::new();
    for (nation, m) in metrics.0.iter() {
        if m.is_destroyed {
            continue;
        }
        let leaning = *matrix.leaning.get(nation).unwrap_or(&50.0);
        let cohesion = *matrix.cohesion.get(nation).unwrap_or(&50.0);
        // Internal drift: prosperity nudges progressive, war fatigue (proxied by low diplomacy)
        let prosperity_push = (m.economy + m.culture) * 0.02;
        let conserv_push = (m.religion + m.military) * 0.01;
        let delta = (prosperity_push - conserv_push).clamp(-3.0, 3.0);
        updates.push((*nation, (leaning + delta).clamp(0.0, 100.0)));

        // Volatility rises if cohesion is low vs prosperity gap
        let vol = matrix.volatility.entry(*nation).or_insert(20.0);
        let prosperity = (m.economy + m.culture + m.science) / 3.0;
        *vol = (*vol + (50.0 - cohesion) * 0.05 + (60.0 - prosperity) * 0.02).clamp(0.0, 100.0);
    }
    for (nation, new_val) in updates {
        matrix.leaning.insert(nation, new_val);
    }

    // Cohesion adjusts toward midline; high volatility erodes it.
    let vol_snapshot = matrix.volatility.clone();
    for (nation, cohesion) in matrix.cohesion.iter_mut() {
        let vol = *vol_snapshot.get(nation).unwrap_or(&20.0);
        *cohesion = (*cohesion + (55.0 - *cohesion) * 0.05 - vol * 0.02).clamp(5.0, 100.0);
    }

    // Rebellion/narrative events if volatility spikes
    for (nation, vol) in matrix.volatility.iter_mut() {
        if *vol > 80.0 {
            *vol -= 10.0; // bleed off after an outburst
            log.push(WorldEvent {
                tick: time.tick,
                epoch: "사상".to_string(),
                season: "불안".to_string(),
                kind: WorldEventKind::Social {
                    convener: crate::simulation::EventActor {
                        id: 0,
                        name: format!("{} 내부 파벌", nation.name()),
                        nation: *nation,
                        faction: Faction::Neutral,
                        faction_label: "내부".to_string(),
                        biome: Biome::Plains,
                        biome_label: "Plains".to_string(),
                        behavior_hint: BehaviorState::Idle,
                        behavior_hint_label: "Idle".to_string(),
                    },
                    gathering_theme: "이념 충돌/폭동".to_string(),
                    cohesion_level: format!("vol {:.0}", *vol),
                },
            });
        }
    }
}

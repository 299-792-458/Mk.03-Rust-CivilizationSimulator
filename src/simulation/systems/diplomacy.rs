use bevy_ecs::prelude::*;

use crate::simulation::{
    AllNationMetrics, DiplomaticRelations, Nation, WorldEvent, WorldEventKind, WorldEventLog,
    WorldTime,
};

/// Evolves diplomatic relations, alliances, and sanctions based on power balance.
pub fn diplomacy_system(
    mut diplo: ResMut<DiplomaticRelations>,
    metrics: Res<AllNationMetrics>,
    time: Res<WorldTime>,
    mut log: ResMut<WorldEventLog>,
) {
    // Initialize trust/fear
    for nation in metrics.0.keys() {
        diplo.trust.entry(*nation).or_insert(40.0);
        diplo.fear.entry(*nation).or_insert(35.0);
    }

    // Update pairwise relations
    let nations: Vec<Nation> = metrics.0.keys().copied().collect();
    for i in 0..nations.len() {
        for j in (i + 1)..nations.len() {
            let a = nations[i];
            let b = nations[j];
            let key = ordered_pair(a, b);
            let entry = diplo.relations.entry(key).or_insert(10.0);
            let (a_power, b_power) = (
                metrics.0.get(&a).map(power_score).unwrap_or(0.0),
                metrics.0.get(&b).map(power_score).unwrap_or(0.0),
            );
            let balance = (a_power - b_power).abs();
            let parity_bonus = if balance < 8.0 { 2.0 } else { -1.5 };
            *entry = (*entry + parity_bonus).clamp(-100.0, 100.0);
            // trust/fear drift
            *diplo.trust.entry(a).or_insert(40.0) += parity_bonus * 0.5;
            *diplo.trust.entry(b).or_insert(40.0) += parity_bonus * 0.5;
            *diplo.fear.entry(a).or_insert(35.0) += (b_power - a_power) * 0.02;
            *diplo.fear.entry(b).or_insert(35.0) += (a_power - b_power) * 0.02;
        }
    }

    // Spawn alliances when relations strong and fear moderate
    for ((a, b), score) in diplo.relations.clone() {
        if score > 55.0 && !diplo.alliances.contains(&(a, b)) {
            diplo.alliances.push((a, b));
            log.push(WorldEvent {
                tick: time.tick,
                epoch: "외교".to_string(),
                season: "동맹".to_string(),
                kind: WorldEventKind::Social {
                    convener: crate::simulation::EventActor {
                        id: 0,
                        name: format!("{}-{} 조약", a.name(), b.name()),
                        nation: a,
                        faction: crate::simulation::Faction::Neutral,
                        faction_label: "조약".to_string(),
                        biome: crate::simulation::Biome::Plains,
                        biome_label: "Plains".to_string(),
                        behavior_hint: crate::simulation::BehaviorState::Idle,
                        behavior_hint_label: "Treaty".to_string(),
                    },
                    gathering_theme: "동맹 체결".to_string(),
                    cohesion_level: format!("score {:.0}", score),
                },
            });
        }
        if score < -45.0 && !diplo.sanctions.contains(&(a, b)) {
            diplo.sanctions.push((a, b));
            log.push(WorldEvent {
                tick: time.tick,
                epoch: "외교".to_string(),
                season: "제재".to_string(),
                kind: WorldEventKind::MacroShock {
                    stressor: format!("{}가 {}를 제재", a.name(), b.name()),
                    catalyst: "무역 차단".to_string(),
                    projected_impact: "경제 위축".to_string(),
                    casualties: None,
                },
            });
        }
    }
}

fn power_score(m: &crate::simulation::NationMetrics) -> f32 {
    (m.military * 0.4 + m.economy * 0.3 + m.science * 0.2 + m.diplomacy * 0.1) / 10.0
}

fn ordered_pair(a: Nation, b: Nation) -> (Nation, Nation) {
    if (a as u32) < (b as u32) {
        (a, b)
    } else {
        (b, a)
    }
}

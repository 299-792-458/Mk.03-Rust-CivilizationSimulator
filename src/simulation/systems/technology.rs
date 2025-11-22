use bevy_ecs::prelude::*;

use crate::simulation::{AllNationMetrics, WorldEvent, WorldEventLog, WorldMetadata, WorldTime};

/// Advances nations through eras and weapon tiers based on accumulated science/culture/military.
pub fn technology_system(
    mut all_metrics: ResMut<AllNationMetrics>,
    world_meta: Res<WorldMetadata>,
    mut event_log: ResMut<WorldEventLog>,
    time: Res<WorldTime>,
) {
    let tech_tree = &world_meta.tech_tree;
    let (epoch, season) = world_meta.epoch_for_tick(time.tick);

    for (nation, metrics) in all_metrics.0.iter_mut() {
        if metrics.is_destroyed {
            continue;
        }

        metrics.research_stock += metrics.science * 0.45 + metrics.economy * 0.1;
        metrics.culture_stock += metrics.culture * 0.35 + metrics.diplomacy * 0.05;

        metrics.research_stock *= 0.9985;
        metrics.culture_stock *= 0.9985;

        if let Some(next_tier) = tech_tree.next_tier(metrics.era) {
            let science_ready = metrics.research_stock >= next_tier.science_gate;
            let culture_ready = metrics.culture_stock >= next_tier.culture_gate;
            let military_ready = metrics.military >= next_tier.military_gate;

            if science_ready && culture_ready && military_ready {
                metrics.era = next_tier.era;
                metrics.weapon_tier = next_tier.weapon_tier;
                for tech in &next_tier.unlocks {
                    if !metrics.unlocked_techs.contains(tech) {
                        metrics.unlocked_techs.push(*tech);
                    }
                }
                metrics.research_stock -= next_tier.science_gate * 0.4;
                metrics.culture_stock -= next_tier.culture_gate * 0.3;

                event_log.push(WorldEvent::era_shift(
                    time.tick,
                    epoch,
                    season,
                    *nation,
                    next_tier.era,
                    next_tier.weapon_tier,
                ));
            }
        }
    }
}

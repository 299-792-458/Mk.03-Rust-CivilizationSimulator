use bevy_ecs::prelude::*;

use crate::simulation::{
    AllNationMetrics, Nation, ScienceVictory, WorldEvent, WorldEventLog, WorldMetadata, WorldTime,
};

/// 누가 달 탐사(과학 승리)를 선도하는지 세대별 기록을 쌓는다. 1틱=1세대.
pub fn science_victory_system(
    mut tracker: ResMut<ScienceVictory>,
    all_metrics: Res<AllNationMetrics>,
    mut event_log: ResMut<WorldEventLog>,
    time: Res<WorldTime>,
    world_meta: Res<WorldMetadata>,
) {
    // Phase 1: 달 탐사 (과학 승리)
    if !tracker.finished {
        let (epoch, season) = world_meta.epoch_for_tick(time.tick);
        let mut leader: Option<(Nation, f32)> = None;
        let goal = tracker.goal;

        for (nation, metrics) in all_metrics.0.iter() {
            if metrics.is_destroyed || metrics.population == 0 {
                continue;
            }

            let progress_now = {
                let entry = tracker.progress.entry(*nation).or_insert(0.0);
                let population_factor = (metrics.population as f32 / 5_000_000.0).clamp(0.35, 2.3);

                // Tech gates: later eras unlock stronger science throughput
                let era_bonus = match metrics.era {
                    crate::simulation::Era::Dawn => 0.8,
                    crate::simulation::Era::Ancient => 0.9,
                    crate::simulation::Era::Classical => 1.0,
                    crate::simulation::Era::Medieval => 1.05,
                    crate::simulation::Era::Industrial => 1.12,
                    crate::simulation::Era::Modern => 1.2,
                    crate::simulation::Era::Nuclear => 1.35,
                };

                // Collaboration: diplomacy + culture aid science momentum
                let diplomacy_bonus = (metrics.diplomacy * 0.006 + metrics.culture * 0.004).min(4.0);

                // War fatigue slows progress (modeled via military attrition and casualties elsewhere)
                let conflict_drag = (100.0 - metrics.military).max(0.0) * 0.0009;

                let momentum = (metrics.science * 0.042
                    + metrics.economy * 0.012
                    + metrics.culture * 0.007
                    + metrics.research_stock * 0.0025
                    + diplomacy_bonus)
                    * era_bonus
                    * (1.0 - conflict_drag).max(0.6);

                let progress_gain = momentum * population_factor * 0.01;
                *entry = (*entry + progress_gain).min(goal);
                *entry
            };

            if let Some((current_leader, value)) = leader {
                if progress_now > value {
                    leader = Some((*nation, progress_now));
                } else {
                    leader = Some((current_leader, value));
                }
            } else {
                leader = Some((*nation, progress_now));
            }

            // Milestone 이벤트 (25% 단위)
            let milestone_counter = tracker.milestones.entry(*nation).or_insert(0);
            let next_threshold = (*milestone_counter as f32 + 1.0) * 25.0;
            if progress_now >= next_threshold && *milestone_counter < 4 {
                *milestone_counter += 1;
                event_log.push(WorldEvent::science_progress(
                    time.tick,
                    epoch,
                    season,
                    *nation,
                    progress_now.min(goal),
                ));
            }
        }

        if let Some((nation, value)) = leader {
            tracker.leader_history.push(value.min(goal));
            if tracker.leader_history.len() > 256 {
                tracker.leader_history.remove(0);
            }

            if value >= goal && !tracker.finished {
                tracker.finished = true;
                tracker.winner = Some(nation);
                tracker.interstellar_mode = true;
                event_log.push(WorldEvent::science_victory(
                    time.tick,
                    epoch,
                    season,
                    nation,
                    value,
                ));
            }
        }
    } else if tracker.interstellar_mode {
        // Phase 2: 성간 확장
        let (epoch, season) = world_meta.epoch_for_tick(time.tick);
        let leader = tracker.winner.unwrap_or(Nation::Tera);
        let base = tracker.interstellar_progress;
        let growth = 0.6 + (base / tracker.interstellar_goal) * 0.8;
        tracker.interstellar_progress = (base + growth).min(tracker.interstellar_goal);

        if (time.tick % 8) == 0 {
            event_log.push(WorldEvent::interstellar_progress(
                time.tick,
                epoch,
                season,
                leader,
                tracker.interstellar_progress,
            ));
        }

        if tracker.interstellar_progress >= tracker.interstellar_goal {
            tracker.interstellar_mode = false;
            tracker.finished = true;
            event_log.push(WorldEvent::interstellar_victory(
                time.tick,
                epoch,
                season,
                leader,
                tracker.interstellar_progress,
            ));
        }
    }
}

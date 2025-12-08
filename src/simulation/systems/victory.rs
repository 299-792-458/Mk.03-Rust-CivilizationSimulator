use bevy_ecs::prelude::*;

use crate::simulation::{
    AllNationMetrics, Nation, ScienceVictory, WorldBlocs, WorldEvent, WorldEventLog, WorldMetadata,
    WorldTime,
};

/// Tracks moonshot (science victory) leader per tick. 1 tick = 1 generation.
pub fn science_victory_system(
    mut tracker: ResMut<ScienceVictory>,
    all_metrics: Res<AllNationMetrics>,
    blocs: Res<WorldBlocs>,
    mut event_log: ResMut<WorldEventLog>,
    time: Res<WorldTime>,
    world_meta: Res<WorldMetadata>,
) {
    // Phase 1: Lunar exploration (science victory)
    if !tracker.moon_done {
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
                let diplomacy_bonus =
                    (metrics.diplomacy * 0.006 + metrics.culture * 0.004).min(4.0);

                // War fatigue slows progress (modeled via military attrition and casualties elsewhere)
                let conflict_drag = (100.0 - metrics.military).max(0.0) * 0.0009;

                // Bloc bonus/penalty
                let bloc_bonus = blocs
                    .blocs
                    .get(&crate::simulation::BlocKind::ResearchPact)
                    .and_then(|bloc| {
                        if bloc.members.contains(nation) {
                            Some(bloc.strength * 0.3)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(0.0);
                let embargo_drag = blocs
                    .blocs
                    .get(&crate::simulation::BlocKind::Sanction)
                    .and_then(|bloc| {
                        if bloc.members.contains(nation) {
                            Some(bloc.strength * 0.2)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(0.0);

                let momentum = (metrics.science * 0.042
                    + metrics.economy * 0.012
                    + metrics.culture * 0.007
                    + metrics.research_stock * 0.0025
                    + diplomacy_bonus
                    + bloc_bonus
                    - embargo_drag)
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

            // Milestone event (every 25%)
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

            if value >= goal && !tracker.moon_done {
                tracker.moon_done = true;
                tracker.winner = Some(nation);
                tracker.space_stage = crate::simulation::SpaceStage::Mars;
                event_log.push(WorldEvent::science_victory(
                    time.tick, epoch, season, nation, value,
                ));
            }
        }
    } else if matches!(tracker.space_stage, crate::simulation::SpaceStage::Mars) {
        let (epoch, season) = world_meta.epoch_for_tick(time.tick);
        let leader = tracker.winner.unwrap_or(Nation::Tera);
        let base = tracker.mars_progress;
        let growth = 0.4 + (base / tracker.mars_goal) * 0.9;
        tracker.mars_progress = (base + growth).min(tracker.mars_goal);
        if (time.tick % 8) == 0 {
            event_log.push(WorldEvent::interstellar_progress(
                time.tick,
                epoch,
                season,
                leader,
                tracker.mars_progress,
            ));
        }
        if tracker.mars_progress >= tracker.mars_goal {
            tracker.mars_done = true;
            tracker.space_stage = crate::simulation::SpaceStage::Jovian;
        }
    } else if matches!(tracker.space_stage, crate::simulation::SpaceStage::Jovian) {
        let (epoch, season) = world_meta.epoch_for_tick(time.tick);
        let leader = tracker.winner.unwrap_or(Nation::Tera);
        let base = tracker.jovian_progress;
        let growth = 0.35 + (base / tracker.jovian_goal) * 0.8;
        tracker.jovian_progress = (base + growth).min(tracker.jovian_goal);
        if (time.tick % 10) == 0 {
            event_log.push(WorldEvent::interstellar_progress(
                time.tick,
                epoch,
                season,
                leader,
                tracker.jovian_progress,
            ));
        }
        if tracker.jovian_progress >= tracker.jovian_goal {
            tracker.jovian_done = true;
            tracker.space_stage = crate::simulation::SpaceStage::Interstellar;
            tracker.interstellar_mode = true;
        }
    } else if tracker.interstellar_mode {
        // Phase 3: Interstellar expansion
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

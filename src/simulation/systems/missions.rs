use bevy_ecs::prelude::*;

use crate::simulation::{
    WorldEvent, WorldEventKind, WorldEventLog, WorldTime,
    components::{Behavior, BehaviorState, Goals, Identity, Personality, Position},
};

/// Assigns simple missions based on goals/personality, logging highlights.
pub fn mission_system(
    mut query: Query<(&Identity, &Personality, &Goals, &mut Behavior, &Position)>,
    time: Res<WorldTime>,
    mut log: ResMut<WorldEventLog>,
) {
    for (id, personality, goals, mut behavior, position) in query.iter_mut() {
        let target_state = match goals.primary {
            crate::simulation::GoalKind::Wealth => {
                if personality.curious > 0.6 {
                    BehaviorState::Explore
                } else {
                    BehaviorState::Trade
                }
            }
            crate::simulation::GoalKind::Glory => BehaviorState::Hunt,
            crate::simulation::GoalKind::Survival => BehaviorState::Gather,
            crate::simulation::GoalKind::Influence => {
                if personality.social > 0.5 {
                    BehaviorState::Rest
                } else {
                    BehaviorState::Trade
                }
            }
        };
        behavior.state = target_state;

        // Occasionally log a mission note
        if time.tick % 48 == 0 && goals.intensity > 0.5 {
            log.push(WorldEvent {
                tick: time.tick,
                epoch: "임무".to_string(),
                season: format!("위치 {:.0},{:.0}", position.x, position.y),
                kind: WorldEventKind::Social {
                    convener: crate::simulation::EventActor {
                        id: id.id,
                        name: id.name.clone(),
                        nation: id.nation,
                        faction: id.faction,
                        faction_label: format!("{:?}", id.faction),
                        biome: position.biome,
                        biome_label: format!("{:?}", position.biome),
                        behavior_hint: target_state,
                        behavior_hint_label: format!("{:?}", target_state),
                    },
                    gathering_theme: format!("{} 임무", goal_label(goals.primary)),
                    cohesion_level: format!("강도 {:.1}", goals.intensity),
                },
            });
        }
    }
}

fn goal_label(goal: crate::simulation::GoalKind) -> &'static str {
    match goal {
        crate::simulation::GoalKind::Wealth => "부의 추구",
        crate::simulation::GoalKind::Glory => "명예 사냥",
        crate::simulation::GoalKind::Survival => "생존",
        crate::simulation::GoalKind::Influence => "영향력",
    }
}

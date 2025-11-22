//! Movement and combat placeholder system.

use bevy_ecs::prelude::*;

use crate::simulation::{Behavior, BehaviorState, Position};

pub fn movement_and_combat_system(mut query: Query<(&Behavior, &mut Position)>) {
    for (behavior, mut position) in &mut query {
        if matches!(
            behavior.state,
            BehaviorState::Explore | BehaviorState::Gather | BehaviorState::Hunt
        ) {
            position.x += 1.0;
            position.y += 1.0;
        }
    }
}

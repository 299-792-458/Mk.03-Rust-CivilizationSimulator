//! AI state transition system.

use bevy_ecs::prelude::*;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::simulation::{
    Behavior, BehaviorState, Identity, Personality, Position, ScienceVictory, WorldMetadata,
    WorldTime,
};

const IDLE_TRANSITIONS: &[(BehaviorState, f32)] = &[
    (BehaviorState::Idle, 0.2),
    (BehaviorState::Explore, 0.3),
    (BehaviorState::Trade, 0.25),
    (BehaviorState::Rest, 0.25),
];

const EXPLORE_TRANSITIONS: &[(BehaviorState, f32)] = &[
    (BehaviorState::Explore, 0.2),
    (BehaviorState::Gather, 0.35),
    (BehaviorState::Hunt, 0.25),
    (BehaviorState::Trade, 0.2),
];

const GATHER_TRANSITIONS: &[(BehaviorState, f32)] = &[
    (BehaviorState::Gather, 0.25),
    (BehaviorState::Trade, 0.4),
    (BehaviorState::Rest, 0.2),
    (BehaviorState::Explore, 0.15),
];

const TRADE_TRANSITIONS: &[(BehaviorState, f32)] = &[
    (BehaviorState::Trade, 0.35),
    (BehaviorState::Rest, 0.2),
    (BehaviorState::Idle, 0.15),
    (BehaviorState::Explore, 0.3),
];

const HUNT_TRANSITIONS: &[(BehaviorState, f32)] = &[
    (BehaviorState::Hunt, 0.25),
    (BehaviorState::Rest, 0.3),
    (BehaviorState::Gather, 0.2),
    (BehaviorState::Trade, 0.25),
];

const REST_TRANSITIONS: &[(BehaviorState, f32)] = &[
    (BehaviorState::Rest, 0.3),
    (BehaviorState::Idle, 0.35),
    (BehaviorState::Explore, 0.2),
    (BehaviorState::Trade, 0.15),
];

fn transition_options(state: BehaviorState) -> &'static [(BehaviorState, f32)] {
    match state {
        BehaviorState::Idle => IDLE_TRANSITIONS,
        BehaviorState::Explore => EXPLORE_TRANSITIONS,
        BehaviorState::Gather => GATHER_TRANSITIONS,
        BehaviorState::Trade => TRADE_TRANSITIONS,
        BehaviorState::Hunt => HUNT_TRANSITIONS,
        BehaviorState::Rest => REST_TRANSITIONS,
    }
}

fn personality_modifier(personality: &Personality, state: BehaviorState) -> f32 {
    let mut modifier = 1.0;

    modifier += match state {
        BehaviorState::Hunt => personality.aggressive * 0.6 - personality.cautious * 0.3,
        BehaviorState::Trade => personality.social * 0.65 + personality.curious * 0.15,
        BehaviorState::Explore => personality.curious * 0.6 - personality.cautious * 0.2,
        BehaviorState::Gather => personality.curious * 0.2 + personality.cautious * 0.2,
        BehaviorState::Rest => personality.cautious * 0.4 - personality.aggressive * 0.2,
        BehaviorState::Idle => (personality.cautious - personality.curious) * 0.1,
    };

    modifier.clamp(0.1, 2.5)
}

fn epoch_modifier(segment: &str, state: BehaviorState) -> f32 {
    match segment {
        "새벽" => match state {
            BehaviorState::Explore | BehaviorState::Gather => 1.15,
            BehaviorState::Rest => 0.85,
            _ => 1.0,
        },
        "한낮" => match state {
            BehaviorState::Trade => 1.25,
            BehaviorState::Idle => 0.75,
            _ => 1.0,
        },
        "해질녘" => match state {
            BehaviorState::Hunt => 1.25,
            BehaviorState::Rest => 1.1,
            BehaviorState::Trade => 0.75,
            _ => 1.0,
        },
        _ => 1.0,
    }
}

fn season_modifier(season: &str, state: BehaviorState) -> f32 {
    match season {
        "꽃피움 계절" => match state {
            BehaviorState::Gather => 1.2,
            BehaviorState::Trade => 1.05,
            _ => 1.0,
        },
        "불꽃 절정" => match state {
            BehaviorState::Explore | BehaviorState::Hunt => 1.1,
            BehaviorState::Rest => 0.95,
            _ => 1.0,
        },
        "잿불 내림" => match state {
            BehaviorState::Rest => 1.25,
            BehaviorState::Trade => 0.9,
            _ => 1.0,
        },
        _ => 1.0,
    }
}

pub fn ai_state_transition_system(
    mut query: Query<(&Identity, &Position, &Personality, &mut Behavior)>,
    world_meta: Res<WorldMetadata>,
    time: Res<WorldTime>,
    science: Res<ScienceVictory>,
) {
    let (segment, season) = world_meta.epoch_for_tick(time.tick);

    for (identity, position, personality, mut behavior) in &mut query {
        let options = transition_options(behavior.state);

        let mut weighted_options = Vec::with_capacity(options.len());
        for (next_state, base_weight) in options {
            let mut weight = *base_weight;

            weight *= personality_modifier(personality, *next_state);
            weight *= world_meta.biome_behavior_bias(position.biome, *next_state);
            weight *= world_meta.faction_behavior_modifier(identity.faction, *next_state);
            weight *= epoch_modifier(segment, *next_state);
            weight *= season_modifier(season, *next_state);
            // Macro goal tilt: during science race, prefer trade/gather over hunt.
            if !science.finished {
                match next_state {
                    BehaviorState::Trade => weight *= 1.25,
                    BehaviorState::Gather => weight *= 1.1,
                    BehaviorState::Hunt => weight *= 0.8,
                    _ => {}
                }
            }

            // Ensure we never end up with non-positive weights.
            weight = weight.max(0.01);
            weighted_options.push((*next_state, weight));
        }

        let mut rng = SmallRng::seed_from_u64(
            time.tick
                .wrapping_mul(97)
                .wrapping_add(identity.id)
                .wrapping_mul(53),
        );
        let total_weight: f32 = weighted_options.iter().map(|(_, w)| *w).sum();
        let mut threshold = rng.gen_range(0.0..total_weight);

        for (candidate, weight) in weighted_options {
            threshold -= weight;
            if threshold <= 0.0 {
                behavior.state = candidate;
                break;
            }
        }
    }
}

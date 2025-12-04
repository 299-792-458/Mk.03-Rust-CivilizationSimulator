use bevy_ecs::prelude::*;

use crate::simulation::{
    AllNationMetrics, NuclearBlasts, WarFatigue, WorldEventLog, WorldTime, events::WorldEventKind,
};

/// Tracks war fatigue and fallout intensity, decaying over time and spiking on wars/nukes.
pub fn war_fatigue_system(
    mut fatigue: ResMut<WarFatigue>,
    mut blasts: ResMut<NuclearBlasts>,
    _time: Res<WorldTime>,
    events: Res<WorldEventLog>,
    mut metrics: ResMut<AllNationMetrics>,
) {
    let decay = 0.98_f32;
    fatigue.intensity *= decay;
    if fatigue.intensity < 1.0 {
        fatigue.intensity = 0.0;
    }

    for value in blasts.0.values_mut() {
        if *value > 0 {
            *value = value.saturating_sub(1);
            fatigue.intensity += 1.5;
        }
    }

    // Event-driven spikes
    for evt in events.snapshot().iter().rev().take(8) {
        match &evt.kind {
            WorldEventKind::Warfare {
                nuclear,
                casualties,
                ..
            } => {
                fatigue.intensity += (*casualties as f32 / 100_000.0).min(12.0);
                if *nuclear {
                    fatigue.intensity += 8.0;
                }
            }
            WorldEventKind::MacroShock {
                projected_impact, ..
            } => {
                let magnitude = projected_impact
                    .chars()
                    .filter(|c| c.is_ascii_digit())
                    .count() as f32;
                fatigue.intensity += magnitude.min(8.0);
            }
            _ => {}
        }
    }

    fatigue.intensity = fatigue.intensity.clamp(0.0, 100.0);
    let intensity = fatigue.intensity;
    push_history(&mut fatigue.history, intensity);

    // Apply soft penalties to all nations to reflect global weariness
    let penalty = fatigue.intensity * 0.02;
    for (_, m) in metrics.0.iter_mut() {
        m.economy = (m.economy - penalty).max(0.0);
        m.culture = (m.culture - penalty * 0.5).max(0.0);
        m.military = (m.military - penalty * 0.25).max(0.0);
    }
}

fn push_history(history: &mut Vec<f32>, value: f32) {
    history.push(value);
    if history.len() > 512 {
        let mut i = 0;
        history.retain(|_| {
            let keep = i % 2 == 0;
            i += 1;
            keep
        });
    }
}

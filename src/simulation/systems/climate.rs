use bevy_ecs::prelude::*;

use crate::simulation::{
    ClimateState, NuclearBlasts, WarFatigue, WorldEvent, WorldEventKind, WorldEventLog,
    WorldRichness, WorldTime,
};

/// Updates global climate state based on war/industry signals.
pub fn climate_system(
    mut climate: ResMut<ClimateState>,
    richness: Res<WorldRichness>,
    blasts: Res<NuclearBlasts>,
    fatigue: Res<WarFatigue>,
    cosmic: Res<crate::simulation::CosmicTimeline>,
    time: Res<WorldTime>,
    mut log: ResMut<WorldEventLog>,
) {
    // Baseline drift follows geologic stage and prosperity.
    let stage_factor = match cosmic.geologic_stage.as_str() {
        "Primordial Crust" => 0.45,
        "Ancient Ocean" => 0.7,
        "Oxygen Bloom" => 1.0,
        "Cambrian/Continental Split" => 1.15,
        "Extinction Cycle" => 1.35,
        _ => 1.5,
    };
    let blast_total: u32 = blasts.0.values().map(|v| *v as u32).sum();

    // Emissions: prosperity + war + fallout
    let mut carbon_delta = 0.04 * stage_factor;
    carbon_delta += richness.richness * 0.75 * stage_factor;
    carbon_delta += fatigue.intensity * 0.02 * stage_factor;
    carbon_delta += blast_total as f32 * 0.06;

    // Biodiversity offers mild mitigation; riskier stages erode it faster.
    let mitigation = (climate.biodiversity / 100.0).clamp(0.1, 1.0);
    carbon_delta *= 1.0 - mitigation * 0.25;

    climate.carbon_ppm = (climate.carbon_ppm + carbon_delta).clamp(180.0, 1500.0);

    // Biodiversity erosion and slow recovery toward a ceiling.
    let erosion = (climate.carbon_ppm / 1200.0) * stage_factor * 0.8
        + fatigue.intensity * 0.01
        + blast_total as f32 * 0.05;
    let recovery = (1.0 - richness.richness).max(0.05) * 0.6;
    climate.biodiversity = (climate.biodiversity - erosion + recovery).clamp(5.0, 120.0);

    // Composite climate risk: carbon weight, war pressure, fallout, prosperity.
    let carbon_pressure = (climate.carbon_ppm / 10.0).powf(0.92);
    let war_pressure = fatigue.intensity * 0.5;
    let prosperity_pressure = richness.richness * 22.0;
    climate.climate_risk =
        (carbon_pressure + war_pressure + prosperity_pressure + blast_total as f32 * 1.2)
            .clamp(0.0, 140.0);

    // Sea/ice lines respond slowly toward targets to avoid jitter.
    let target_sea = ((climate.climate_risk / 140.0).powf(1.2) * 0.8 + richness.richness * 0.2)
        .clamp(0.02, 0.98);
    let target_ice = (1.0 - (climate.carbon_ppm / 1400.0).powf(0.85)).clamp(0.0, 0.9);
    climate.sea_level = lerp(climate.sea_level, target_sea, 0.08);
    climate.ice_line = lerp(climate.ice_line, target_ice, 0.08);

    let carbon_ppm = climate.carbon_ppm;
    let risk = climate.climate_risk;
    let bio = climate.biodiversity;
    push_history(&mut climate.carbon_history, carbon_ppm);
    push_history(&mut climate.climate_risk_history, risk);
    push_history(&mut climate.biodiversity_history, bio);

    // Event pulses occasionally
    if time.tick % 24 == 0 {
        log.push(WorldEvent {
            tick: time.tick,
            epoch: "Climate".to_string(),
            season: "Planet".to_string(),
            kind: WorldEventKind::MacroShock {
                stressor: "Climate shift warning".to_string(),
                catalyst: format!(
                    "Carbon {:.0}ppm | Risk {:.1}%",
                    climate.carbon_ppm, climate.climate_risk
                ),
                projected_impact: "Productivity loss / population risk".to_string(),
                casualties: None,
            },
        });
    }
}

fn lerp(current: f32, target: f32, alpha: f32) -> f32 {
    current + (target - current) * alpha
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

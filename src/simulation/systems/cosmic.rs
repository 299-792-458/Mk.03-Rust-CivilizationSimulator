use bevy_ecs::prelude::*;

use crate::simulation::{
    ClimateState, CosmicTimeline, WorldEvent, WorldEventKind, WorldEventLog, WorldMetadata,
    WorldRichness, WorldTime,
};

/// Advances cosmic/geologic time and sets stage labels.
pub fn cosmic_time_system(
    mut cosmic: ResMut<CosmicTimeline>,
    time: Res<WorldTime>,
    meta: Res<WorldMetadata>,
    mut log: ResMut<WorldEventLog>,
) {
    // Advance age
    cosmic.cosmic_age_years += cosmic.timescale_years_per_tick;
    cosmic.geologic_stage = stage_for_age(cosmic.cosmic_age_years);

    // Emit milestone events on big thresholds
    if (time.tick % 64) == 0 {
        let (epoch, season) = meta.epoch_for_tick(time.tick);
        log.push(WorldEvent {
            tick: time.tick,
            epoch: epoch.to_string(),
            season: season.to_string(),
            kind: WorldEventKind::MacroShock {
                stressor: "Geologic/space transition".to_string(),
                catalyst: format!(
                    "Age {:.2}e8 yrs | Stage {}",
                    cosmic.cosmic_age_years / 100_000_000.0,
                    cosmic.geologic_stage
                ),
                projected_impact: "Environment/biosphere reset possible".to_string(),
                casualties: None,
            },
        });
    }
}

/// Triggers extinction/reboot when climate or age thresholds are exceeded.
pub fn extinction_system(
    mut cosmic: ResMut<CosmicTimeline>,
    mut climate: ResMut<ClimateState>,
    mut richness: ResMut<WorldRichness>,
    mut metrics: ResMut<crate::simulation::AllNationMetrics>,
    mut civ: ResMut<crate::simulation::AllNationCivState>,
    mut fatigue: ResMut<crate::simulation::WarFatigue>,
    mut log: ResMut<WorldEventLog>,
    time: Res<WorldTime>,
    meta: Res<WorldMetadata>,
) {
    let extreme_climate = climate.climate_risk > 95.0 || climate.biodiversity < 1.0;
    let ancient_transition = cosmic.cosmic_age_years > 5_000_000_000.0
        && (cosmic.cosmic_age_years as u64 % 1_000_000_000 == 0);

    if !extreme_climate && !ancient_transition {
        return;
    }

    // Scale severity by timescale (faster time -> softer reset) and age (older -> harsher)
    let timescale_factor = (cosmic.timescale_years_per_tick / 1_000_000.0).clamp(0.1, 100.0);
    let age_factor = ((cosmic.cosmic_age_years / 1_000_000_000.0) / 5.0).clamp(0.5, 2.0);
    let severity = (1.0 / timescale_factor as f32) * age_factor as f32;

    // Increment extinction count and soften world state
    cosmic.extinction_events += 1;
    climate.biodiversity = (20.0 * severity).max(5.0).min(80.0);
    climate.climate_risk = (20.0 * severity).min(80.0);
    climate.carbon_ppm = (280.0 * severity).min(600.0);
    richness.richness *= (0.25 * severity as f32).min(0.8).max(0.05);
    fatigue.intensity = 0.0;

    for (_nation, m) in metrics.0.iter_mut() {
        if m.is_destroyed {
            continue;
        }
        m.population = (m.population as f32 * (0.2 * severity).min(0.6)) as u64;
        m.economy *= (0.25 * severity).min(0.8);
        m.culture *= (0.3 * severity).min(0.85);
        m.military *= (0.2 * severity).min(0.7);
        m.science *= (0.2 * severity).min(0.7);
        m.research_stock = 0.0;
        m.culture_stock = 0.0;
    }

    for (_nation, s) in civ.0.iter_mut() {
        s.cities = (s.cities as f32 * 0.3).max(1.0) as u32;
        s.happiness = 45.0;
        s.production *= 0.25;
        s.stability = 40.0;
    }

    let (epoch, season) = meta.epoch_for_tick(time.tick);
    log.push(WorldEvent {
        tick: time.tick,
        epoch: epoch.to_string(),
        season: season.to_string(),
        kind: WorldEventKind::MacroShock {
            stressor: "Mass extinction/reboot".to_string(),
            catalyst: format!(
                "Events {} | Age {:.2}e8 yrs",
                cosmic.extinction_events,
                cosmic.cosmic_age_years / 100_000_000.0
            ),
            projected_impact: "Population/resource reset then regrowth".to_string(),
            casualties: None,
        },
    });
}

fn stage_for_age(age_years: f64) -> String {
    let gy = age_years / 1_000_000_000.0;
    if gy < 0.5 {
        "Primordial Crust".to_string()
    } else if gy < 1.0 {
        "Ancient Ocean".to_string()
    } else if gy < 2.5 {
        "Oxygen Bloom".to_string()
    } else if gy < 3.5 {
        "Cambrian/Continental Split".to_string()
    } else if gy < 4.5 {
        "Extinction Cycle".to_string()
    } else {
        "Civilization/Space Age".to_string()
    }
}

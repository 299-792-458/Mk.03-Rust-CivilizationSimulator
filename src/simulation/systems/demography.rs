use bevy_ecs::prelude::*;

use crate::simulation::{AllNationMetrics, ClimateState, WorldTime};

/// Age structure & productivity cycle updates.
pub fn demography_system(
    mut metrics: ResMut<AllNationMetrics>,
    climate: Res<ClimateState>,
    time: Res<WorldTime>,
) {
    let tick = time.tick as f32;
    let climate_drag = (climate.climate_risk * 0.003).min(0.25);
    let cycle = (tick.sin() + 1.0) * 0.5; // simple business cycle proxy 0..1

    for (_, m) in metrics.0.iter_mut() {
        if m.is_destroyed {
            continue;
        }

        // Demographic flow
        let births = (m.adult as f32 * 0.0008).round() as u64;
        let aging_youth = (m.youth as f32 * 0.0015).round() as u64;
        let aging_adult = (m.adult as f32 * 0.0009).round() as u64;
        let elder_mortality = (m.elder as f32 * 0.0025).round() as u64;

        m.youth = m.youth.saturating_add(births).saturating_sub(aging_youth);
        m.adult = m
            .adult
            .saturating_add(aging_youth)
            .saturating_sub(aging_adult);
        m.elder = m
            .elder
            .saturating_add(aging_adult)
            .saturating_sub(elder_mortality);

        m.population = m.youth + m.adult + m.elder;

        // Productivity shaped by cycle and climate
        m.productivity = (1.0 + cycle * 0.2 - climate_drag).max(0.4);
        m.unemployment = (6.0 + (1.0 - cycle) * 5.0 + climate_drag * 10.0).min(40.0);

        // Tie back into economy/science
        let effective_workers = m.adult as f32 * (1.0 - m.unemployment / 100.0);
        m.economy += (effective_workers / 1_000_000.0) * m.productivity * 0.8;
        m.science += (m.elder as f32 / 1_000_000.0) * 0.3 * m.productivity;
        m.culture += (m.elder as f32 / 1_000_000.0) * 0.4;

        // Clamp
        m.economy = m.economy.min(250.0);
        m.science = m.science.min(250.0);
        m.culture = m.culture.min(250.0);
    }
}

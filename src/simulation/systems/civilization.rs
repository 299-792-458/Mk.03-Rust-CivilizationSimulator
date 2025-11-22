use bevy_ecs::prelude::*;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::simulation::{AllNationCivState, AllNationMetrics, WorldTime};

/// Civilization-style progression: cities drive production, growth, and happiness.
pub fn civilization_system(
    mut civ: ResMut<AllNationCivState>,
    mut metrics: ResMut<AllNationMetrics>,
    time: Res<WorldTime>,
) {
    let mut rng = SmallRng::seed_from_u64(time.tick.wrapping_mul(911));

    for (nation, civ_state) in civ.0.iter_mut() {
        if let Some(m) = metrics.0.get_mut(nation) {
            if m.is_destroyed {
                m.population = 0;
                civ_state.cities = 0;
                civ_state.production = 0.0;
                civ_state.happiness = 0.0;
                civ_state.stability = 0.0;
                continue;
            }

            // Population growth scaled by cities
            let growth =
                (civ_state.cities as u64 * 5_000) + ((m.population as f32 * 0.0005) as u64);
            m.population = m.population.saturating_add(growth);

            // Production feeds economy, science, culture
            let prod = civ_state.production.max(0.0);
            m.economy = (m.economy + prod * 0.15).clamp(0.0, 120.0);
            m.science = (m.science + prod * 0.12).clamp(0.0, 120.0);
            m.culture = (m.culture + prod * 0.10).clamp(0.0, 120.0);

            // Happiness affects economy and stability
            if civ_state.happiness < 40.0 {
                m.economy *= 0.995;
                civ_state.stability -= 0.2;
            } else if civ_state.happiness > 75.0 {
                m.economy *= 1.003;
                civ_state.stability += 0.15;
            }

            // Diminishing unrest
            civ_state.happiness = civ_state.happiness.clamp(0.0, 100.0);
            civ_state.stability = civ_state.stability.clamp(0.0, 100.0);

            // City founding chance if there is space (territory proportional)
            let max_cities = (m.territory / 8.0).ceil() as u32 + 1;
            if civ_state.cities < max_cities {
                let chance = 0.04 + m.culture.max(10.0) / 1000.0;
                if rng.gen_bool(chance as f64) {
                    civ_state.cities += 1;
                    civ_state.production += 3.0;
                    civ_state.happiness += 2.0;
                    m.population = m.population.saturating_add(120_000);
                }
            }

            // Small war-weariness decay
            civ_state.happiness += 0.05;
            civ_state.happiness = civ_state.happiness.clamp(0.0, 100.0);
        }
    }
}

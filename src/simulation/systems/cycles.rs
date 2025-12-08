use bevy_ecs::prelude::*;

use crate::simulation::{AllNationCivState, AllNationMetrics, CivilizationalCycles, WorldTime};

/// Applies golden age / decline cycles per nation based on prosperity and shocks.
pub fn cycle_system(
    mut cycles: ResMut<CivilizationalCycles>,
    mut metrics: ResMut<AllNationMetrics>,
    mut civ_state: ResMut<AllNationCivState>,
    time: Res<WorldTime>,
) {
    for nation in metrics.0.keys() {
        cycles.golden_age.entry(*nation).or_insert(0.0);
        cycles.decline.entry(*nation).or_insert(0.0);
    }

    for (nation, m) in metrics.0.iter_mut() {
        if m.is_destroyed {
            continue;
        }
        let golden_val = *cycles.golden_age.get(nation).unwrap_or(&0.0);
        let decline_val = *cycles.decline.get(nation).unwrap_or(&0.0);

        // Prosperity push into golden age
        let prosperity = (m.economy + m.culture + m.science) / 3.0;
        let stress = (100.0 - m.diplomacy).max(0.0) + (80.0 - m.stability()).max(0.0);
        let mut golden =
            (golden_val + (prosperity - 55.0) * 0.02 - decline_val * 0.01).clamp(0.0, 120.0);
        let mut decline = (decline_val + (stress) * 0.015 - golden_val * 0.01).clamp(0.0, 120.0);

        // Apply modifiers
        let golden_factor = 1.0 + (golden / 120.0) * 0.25;
        let decline_factor = 1.0 - (decline / 120.0) * 0.3;
        m.economy = (m.economy * golden_factor * decline_factor).clamp(10.0, 240.0);
        m.culture = (m.culture * golden_factor * decline_factor).clamp(10.0, 240.0);
        m.science = (m.science * golden_factor * decline_factor).clamp(5.0, 240.0);
        m.military = (m.military * decline_factor + golden * 0.05).clamp(5.0, 220.0);

        if let Some(civ) = civ_state.0.get_mut(nation) {
            civ.happiness = (civ.happiness * golden_factor - decline * 0.15).clamp(5.0, 150.0);
            civ.production = (civ.production * golden_factor * decline_factor).clamp(5.0, 180.0);
            civ.stability = (civ.stability - decline * 0.2 + golden * 0.05).clamp(5.0, 150.0);
        }

        // Slow decay over time to avoid runaway
        if time.tick % 12 == 0 {
            golden *= 0.97;
            decline *= 0.98;
        }

        cycles.golden_age.insert(*nation, golden);
        cycles.decline.insert(*nation, decline);
    }
}

trait StabilityLens {
    fn stability(&self) -> f32;
}

impl StabilityLens for crate::simulation::NationMetrics {
    fn stability(&self) -> f32 {
        // Approximate stability using happiness/diplomacy/culture proxies
        (self.culture * 0.3 + self.diplomacy * 0.4 + self.religion * 0.3) / 3.0
    }
}

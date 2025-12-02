use bevy_ecs::prelude::*;
use rand::SeedableRng;
use rand::{Rng, rngs::SmallRng};

use crate::simulation::{AllNationMetrics, BlocKind, WorldBlocs, WorldTime};

/// Periodically adjusts blocs for cooperation/embargo effects.
pub fn bloc_system(
    mut blocs: ResMut<WorldBlocs>,
    metrics: Res<AllNationMetrics>,
    fatigue: Res<crate::simulation::WarFatigue>,
    climate: Res<crate::simulation::ClimateState>,
    time: Res<WorldTime>,
) {
    let mut rng = SmallRng::seed_from_u64(time.tick.wrapping_mul(313));
    let war_tension = (fatigue.intensity / 100.0).clamp(0.0, 1.0);
    let climate_stress = (climate.climate_risk / 100.0).clamp(0.0, 1.0);
    let coop_bias = 0.5 + (1.0 - war_tension) * 0.2 + climate_stress * 0.1;
    let sanction_bias = 0.3 + war_tension * 0.4 + climate_stress * 0.1;

    // Build desired blocs without holding the map borrow
    let research_template = crate::simulation::Bloc {
        kind: BlocKind::ResearchPact,
        leader: None,
        members: metrics
            .0
            .keys()
            .cloned()
            .filter(|_| rng.gen_bool(coop_bias as f64))
            .collect(),
        strength: 1.0,
    };
    let sanction_template = crate::simulation::Bloc {
        kind: BlocKind::Sanction,
        leader: None,
        members: metrics
            .0
            .keys()
            .cloned()
            .filter(|_| rng.gen_bool(sanction_bias as f64))
            .collect(),
        strength: 1.0,
    };

    let research_bloc_ptr = {
        let entry = blocs
            .blocs
            .entry(BlocKind::ResearchPact)
            .or_insert(research_template);
        entry as *mut _
    };
    let sanction_bloc_ptr = {
        let entry = blocs
            .blocs
            .entry(BlocKind::Sanction)
            .or_insert(sanction_template);
        entry as *mut _
    };
    // SAFETY: We only use raw pointers to avoid simultaneous mutable borrow on HashMap entries,
    // and dereference sequentially below.
    let research_bloc: &mut crate::simulation::Bloc = unsafe { &mut *research_bloc_ptr };
    let sanction_bloc: &mut crate::simulation::Bloc = unsafe { &mut *sanction_bloc_ptr };

    // Recompute strength based on member science
    for bloc in [&mut *research_bloc, &mut *sanction_bloc] {
        let mut total_science = 0.0;
        let mut total_econ = 0.0;
        for nation in bloc.members.iter() {
            if let Some(m) = metrics.0.get(nation) {
                if !m.is_destroyed {
                    total_science += m.science;
                    total_econ += m.economy;
                }
            }
        }
        bloc.strength = (total_science * 0.01 + total_econ * 0.005).min(10.0);

        // Pick leader as highest science member
        bloc.leader = bloc
            .members
            .iter()
            .filter_map(|n| metrics.0.get(n).map(|m| (n, m.science)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(n, _)| *n);
    }
}

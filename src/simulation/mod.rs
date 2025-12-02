use std::sync::{Arc, RwLock};

use bevy_ecs::prelude::*;
use bevy_ecs::schedule::Schedule;
use std::collections::{HashMap, HashSet};

pub mod components;
pub mod events;
pub mod grid;
pub mod localization;
pub mod nation;
pub mod observer;
pub mod resources;
pub mod systems;
pub mod technology;
pub mod world;
pub mod blocs;

pub use components::*;
pub use events::*;
pub use grid::*;
pub use localization::*;
pub use nation::*;
pub use observer::*;
pub use resources::*;
pub use systems::*;
pub use technology::*;
pub use world::*;
pub use blocs::*;

pub struct SimulationWorld {
    world: World,
    schedule: Schedule,
    observer: Arc<RwLock<ObserverSnapshot>>,
}

impl SimulationWorld {
    #[allow(dead_code)]
    pub fn new(config: SimulationConfig) -> Self {
        Self::with_observer(config, Arc::new(RwLock::new(ObserverSnapshot::default())))
    }

    pub fn with_observer(
        config: SimulationConfig,
        observer: Arc<RwLock<ObserverSnapshot>>,
    ) -> Self {
        let mut world = World::default();
        world.insert_resource(config);
        world.insert_resource(AllNationMetrics::default());
        world.insert_resource(AllNationCivState::default());
        world.insert_resource(NuclearBlasts::default());
        world.insert_resource(WarFatigue::default());
        world.insert_resource(WorldRichness::default());
        world.insert_resource(ClimateState::default());
        world.insert_resource(WorldBlocs::default());
        world.insert_resource(WorldTime::default());
        world.insert_resource(WorldMetadata::default());
        world.insert_resource(WorldEventLog::default());
        world.insert_resource(ScienceVictory::default());

        seed_entities(&mut world);
        seed_grid(&mut world);

        let mut schedule = Schedule::default();
        schedule.add_systems(
            (
                ai_state_transition_system,
                combat_cleanup_system, // Clean up combat from previous tick
                economy_system,
                environment_system,
                civilization_system,
                technology_system,
                warfare_system, // Handles starting new combat
                science_victory_system,
                climate_system,
                nuclear_decay_system,
                peace_recovery_system,
                richness_overlay_system,
                climate_impact_system,
                bloc_system,
                war_fatigue_system,
                territory_system,
                demography_system,
                event_generation_system,
                logging_system,
            )
                .chain(),
        );

        Self {
            world,
            schedule,
            observer,
        }
    }

    pub fn tick(&mut self) {
        {
            let mut time = self.world.resource_mut::<WorldTime>();
            time.tick += 1;
        }

        self.schedule.run(&mut self.world);
        self.refresh_observer_snapshot();
    }

    fn refresh_observer_snapshot(&mut self) {
        let tick = self.world.resource::<WorldTime>().tick;
        let world_meta = self.world.resource::<WorldMetadata>().clone();
        let metrics = self.world.resource::<AllNationMetrics>().clone();
        let civ_state = self.world.resource::<AllNationCivState>().clone();
        let nuclear = self.world.resource::<NuclearBlasts>().0.clone();
        let war_fatigue = self.world.resource::<WarFatigue>().clone();
        let richness = self.world.resource::<WorldRichness>().clone();
        let climate = self.world.resource::<ClimateState>().clone();
        let science_victory_snapshot = {
            let tracker = self.world.resource::<ScienceVictory>();
            let mut ordered: Vec<_> = tracker.progress.iter().collect();
            ordered.sort_by(|a, b| {
                b.1.partial_cmp(a.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            let leader = ordered.get(0).map(|(nation, progress)| (**nation, **progress));
            let runner_up = ordered.get(1).map(|(_, progress)| **progress);

            observer::ScienceVictorySnapshot {
                leader: leader.map(|(nation, _)| nation),
                leader_progress: leader.map(|(_, progress)| progress).unwrap_or(0.0),
                runner_up_progress: runner_up.unwrap_or(0.0),
                history: tracker.leader_history.clone(),
                goal: tracker.goal,
                finished: tracker.finished,
                winner: tracker.winner,
                interstellar_mode: tracker.interstellar_mode,
                interstellar_progress: tracker.interstellar_progress,
                interstellar_goal: tracker.interstellar_goal,
                carbon_ppm: climate.carbon_ppm,
                climate_risk: climate.climate_risk,
                biodiversity: climate.biodiversity,
            }
        };

        // We need to construct a new HexGrid snapshot because the resource now holds entities.
        let grid_snapshot = {
            let mut hexes = HashMap::new();
            let mut query = self.world.query::<(&AxialCoord, &Hex)>();
            for (coord, hex) in query.iter(&self.world) {
                hexes.insert(*coord, observer::HexSnapshot { owner: hex.owner });
            }
            observer::HexGridSnapshot {
                hexes,
                radius: self.world.resource::<HexGrid>().radius,
            }
        };

        let (epoch, season) = {
            let (epoch, season) = world_meta.epoch_for_tick(tick);
            (epoch.to_string(), season.to_string())
        };
        let season_effect = seasonal_effect_for(&season, tick);

        let events = {
            let log = self.world.resource::<WorldEventLog>();
            log.snapshot()
        };

        let mut entity_query = self
            .world
            .query::<(&Identity, &Position, &Behavior, &Inventory, &Attributes)>();

        let entities = entity_query
            .iter(&self.world)
            .map(
                |(identity, position, behavior, inventory, attributes)| EntitySnapshot {
                    id: identity.id,
                    name: identity.name.clone(),
                    faction: identity.faction,
                    faction_label: faction_label(identity.faction).to_string(),
                    biome: position.biome,
                    biome_label: world_meta
                        .biomes
                        .get(&position.biome)
                        .map(|meta| meta.label.to_string())
                        .unwrap_or_else(|| format!("{:?}", position.biome)),
                    behavior_state: behavior.state,
                    behavior_label: behavior_label(behavior.state).to_string(),
                    currency: inventory.currency,
                    wealth: attributes.wealth,
                    fame: attributes.fame,
                },
            )
            .collect::<Vec<_>>();

        let combat_hexes = {
            let mut combat_hexes = HashSet::new();
            let mut query = self.world.query::<(&AxialCoord, &InCombat)>();
            for (coord, _) in query.iter(&self.world) {
                combat_hexes.insert(*coord);
            }
            combat_hexes
        };

        if let Ok(mut snapshot) = self.observer.write() {
            snapshot.update(
                tick,
                epoch,
                season,
                season_effect,
                &metrics,
                civ_state,
                grid_snapshot,
                observer::WorldOverlaySnapshot {
                    war_fatigue: war_fatigue.intensity,
                    fallout: nuclear.values().map(|v| *v as f32).sum::<f32>(),
                    resource_richness: richness.richness,
                    war_fatigue_history: war_fatigue.history.clone(),
                    richness_history: richness.history.clone(),
                    carbon_history: climate.carbon_history.clone(),
                    climate_risk_history: climate.climate_risk_history.clone(),
                    biodiversity_history: climate.biodiversity_history.clone(),
                },
                science_victory_snapshot,
                entities,
                events,
                combat_hexes,
                nuclear.keys().cloned().collect(),
            );
        }
    }
}

fn seasonal_effect_for(season: &str, tick: u64) -> observer::SeasonEffectSnapshot {
    // Animated seasonal shifts to drive UI and minor simulation flavor.
    // Uses deterministic wave so tick speed affects intensity.
    let wave = ((tick % 32) as f32 / 32.0 * std::f32::consts::TAU).sin();
    match season {
        "꽃피움 계절" => observer::SeasonEffectSnapshot {
            label: "꽃가루 축제".to_string(),
            temperature: 0.2 + 0.1 * wave,
            morale_shift: 5.0,
            yield_shift: 3.0,
            risk_shift: -2.0,
        },
        "불꽃 절정" => observer::SeasonEffectSnapshot {
            label: "태양 쇄도".to_string(),
            temperature: 0.65 + 0.2 * wave,
            morale_shift: -3.0,
            yield_shift: 6.0,
            risk_shift: 4.0,
        },
        "잿불 내림" => observer::SeasonEffectSnapshot {
            label: "연기 어린 밤".to_string(),
            temperature: -0.15 + 0.1 * wave,
            morale_shift: -1.0,
            yield_shift: -2.0,
            risk_shift: 3.0,
        },
        _ => observer::SeasonEffectSnapshot {
            label: "평온".to_string(),
            temperature: 0.0,
            morale_shift: 0.0,
            yield_shift: 0.0,
            risk_shift: 0.0,
        },
    }
}

fn seed_grid(world: &mut World) {
    let config = world.resource::<SimulationConfig>().clone();
    let radius = config.grid_radius;
    let mut hex_entities = HashMap::new();

    let sectors = [
        Nation::Tera,
        Nation::Sora,
        Nation::Aqua,
        Nation::Solar,
        Nation::Luna,
    ];

    for q in -radius..=radius {
        for r in (-radius).max(-q - radius)..=radius.min(-q + radius) {
            let coord = AxialCoord { q, r };

            // Circular land mask to keep the world round and leave ocean border
            let axial_distance = |a: AxialCoord, b: AxialCoord| {
                ((a.q - b.q).abs() + (a.q + a.r - b.q - b.r).abs() + (a.r - b.r).abs()) / 2
            };
            let land_radius = (radius - 2).max(1);
            if axial_distance(coord, AxialCoord::new(0, 0)) > land_radius {
                continue;
            }

            // 5-way wedge (pentagon) for equal land division
            let angle = (r as f32 * (3.0_f32).sqrt() / 2.0)
                .atan2(q as f32 + r as f32 / 2.0)
                .to_degrees();
            let angle = (angle + 360.0) % 360.0;
            let sector_size = 360.0 / sectors.len() as f32;
            let index =
                ((angle + sector_size / 2.0) / sector_size).floor() as usize % sectors.len();
            let owner = sectors[index];

            let hex_entity = world.spawn((coord, Hex { owner })).id();
            hex_entities.insert(coord, hex_entity);
        }
    }
    world.insert_resource(HexGrid {
        hexes: hex_entities,
        radius,
    });
}

fn seed_entities(world: &mut World) {
    use BehaviorState::*;
    use Nation::*;

    let world_meta = world.resource::<WorldMetadata>().clone();

    let npc_templates = [
        (
            Identity {
                id: 1,
                name: "Calix".to_string(),
                faction: Faction::MerchantGuild,
                nation: Tera,
            },
            world_meta.anchor_position(Biome::Market),
            Inventory {
                items: vec![ItemStack {
                    item: ItemKind::Resource("약초".into()),
                    quantity: 10,
                }],
                currency: 100.0,
            },
            Attributes {
                health: 100.0,
                stamina: 80.0,
                wealth: 120.0,
                fame: 20.0,
            },
            Personality {
                aggressive: 0.1,
                cautious: 0.4,
                social: 0.6,
                curious: 0.5,
            },
            Behavior { state: Idle },
        ),
        (
            Identity {
                id: 2,
                name: "Rena".to_string(),
                faction: Faction::BanditClans,
                nation: Sora,
            },
            world_meta.anchor_position(Biome::Forest),
            Inventory {
                items: vec![ItemStack {
                    item: ItemKind::Equipment("단검".into()),
                    quantity: 1,
                }],
                currency: 45.0,
            },
            Attributes {
                health: 110.0,
                stamina: 95.0,
                wealth: 60.0,
                fame: 45.0,
            },
            Personality {
                aggressive: 0.6,
                cautious: 0.2,
                social: 0.3,
                curious: 0.4,
            },
            Behavior { state: Explore },
        ),
        (
            Identity {
                id: 3,
                name: "Aria".to_string(),
                faction: Faction::ExplorersLeague,
                nation: Aqua,
            },
            world_meta.anchor_position(Biome::Plains),
            Inventory {
                items: vec![],
                currency: 70.0,
            },
            Attributes {
                health: 95.0,
                stamina: 100.0,
                wealth: 80.0,
                fame: 35.0,
            },
            Personality {
                aggressive: 0.2,
                cautious: 0.3,
                social: 0.5,
                curious: 0.7,
            },
            Behavior { state: Gather },
        ),
        (
            Identity {
                id: 4,
                name: "Lys".to_string(),
                faction: Faction::TempleOfSuns,
                nation: Tera,
            },
            world_meta.anchor_position(Biome::Village),
            Inventory {
                items: vec![ItemStack {
                    item: ItemKind::Artifact("태양 성물함".into()),
                    quantity: 1,
                }],
                currency: 30.0,
            },
            Attributes {
                health: 90.0,
                stamina: 70.0,
                wealth: 50.0,
                fame: 65.0,
            },
            Personality {
                aggressive: 0.1,
                cautious: 0.5,
                social: 0.7,
                curious: 0.6,
            },
            Behavior { state: Idle },
        ),
    ];

    for (identity, position, inventory, attributes, personality, behavior) in npc_templates {
        world.spawn((
            identity,
            position,
            inventory,
            attributes,
            personality,
            behavior,
        ));
    }
}

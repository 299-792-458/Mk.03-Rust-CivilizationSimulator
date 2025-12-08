//! Structured metadata describing TERA's worldbuilding fabric.

use std::collections::HashMap;

use bevy_ecs::prelude::Resource;

use crate::simulation::{BehaviorState, Biome, Faction, Position, TechTree};

#[derive(Debug, Clone)]
pub struct BiomeMetadata {
    pub label: &'static str,
    pub epithet: &'static str,
    pub description: &'static str,
    pub anchor: (f32, f32),
    pub resource_profile: Vec<&'static str>,
    pub tensions: Vec<&'static str>,
    pub behavior_bias: HashMap<BehaviorState, f32>,
    pub economic_shift: EconomicShift,
}

#[derive(Debug, Clone)]
pub struct FactionMetadata {
    pub motto: &'static str,
    pub doctrine: &'static str,
    pub influence_vectors: Vec<&'static str>,
    pub strongholds: Vec<Biome>,
    pub behavior_modifiers: HashMap<BehaviorState, f32>,
    pub economy_profile: EconomyProfile,
}

#[derive(Debug, Clone)]
pub struct EconomyMetadata {
    pub circulation_cycle: Vec<&'static str>,
    pub stressors: Vec<&'static str>,
    pub catalysts: Vec<&'static str>,
}

#[derive(Debug, Clone)]
pub struct EconomicShift {
    pub trade_opportunity: f32,
    pub resource_abundance: f32,
    pub risk_factor: f32,
}

impl EconomicShift {
    pub fn trade_opportunity(&self) -> f32 {
        self.trade_opportunity
    }

    pub fn resource_abundance(&self) -> f32 {
        self.resource_abundance
    }

    pub fn risk_factor(&self) -> f32 {
        self.risk_factor
    }
}

#[derive(Debug, Clone)]
pub struct EconomyProfile {
    pub trade_yield: f32,
    pub volatility_resistance: f32,
    pub upkeep_burden: f32,
}

impl EconomyProfile {
    pub fn trade_yield(&self) -> f32 {
        self.trade_yield
    }

    pub fn volatility_resistance(&self) -> f32 {
        self.volatility_resistance
    }

    pub fn upkeep_burden(&self) -> f32 {
        self.upkeep_burden
    }
}

#[derive(Debug, Clone)]
pub struct EpochCadence {
    pub day_segments: Vec<&'static str>,
    pub seasons: Vec<&'static str>,
}

#[derive(Debug, Clone, Resource)]
pub struct WorldMetadata {
    pub biomes: HashMap<Biome, BiomeMetadata>,
    pub factions: HashMap<Faction, FactionMetadata>,
    pub economy: EconomyMetadata,
    pub epochs: EpochCadence,
    pub tech_tree: TechTree,
}

impl WorldMetadata {
    pub fn anchor_position(&self, biome: Biome) -> Position {
        if let Some(metadata) = self.biomes.get(&biome) {
            Position {
                x: metadata.anchor.0,
                y: metadata.anchor.1,
                biome,
            }
        } else {
            Position {
                x: 0.0,
                y: 0.0,
                biome,
            }
        }
    }

    pub fn faction_profile(&self, faction: Faction) -> Option<&FactionMetadata> {
        self.factions.get(&faction)
    }

    pub fn biome_behavior_bias(&self, biome: Biome, state: BehaviorState) -> f32 {
        self.biomes
            .get(&biome)
            .and_then(|meta| meta.behavior_bias.get(&state))
            .copied()
            .unwrap_or(1.0)
    }

    pub fn faction_behavior_modifier(&self, faction: Faction, state: BehaviorState) -> f32 {
        self.factions
            .get(&faction)
            .and_then(|meta| meta.behavior_modifiers.get(&state))
            .copied()
            .unwrap_or(1.0)
    }

    pub fn biome_trade_opportunity(&self, biome: Biome) -> f32 {
        self.biomes
            .get(&biome)
            .map(|meta| meta.economic_shift.trade_opportunity())
            .unwrap_or(1.0)
    }

    pub fn biome_resource_abundance(&self, biome: Biome) -> f32 {
        self.biomes
            .get(&biome)
            .map(|meta| meta.economic_shift.resource_abundance())
            .unwrap_or(1.0)
    }

    pub fn biome_risk_factor(&self, biome: Biome) -> f32 {
        self.biomes
            .get(&biome)
            .map(|meta| meta.economic_shift.risk_factor())
            .unwrap_or(1.0)
    }

    pub fn faction_trade_yield(&self, faction: Faction) -> f32 {
        self.factions
            .get(&faction)
            .map(|meta| meta.economy_profile.trade_yield())
            .unwrap_or(1.0)
    }

    pub fn faction_volatility_resistance(&self, faction: Faction) -> f32 {
        self.factions
            .get(&faction)
            .map(|meta| meta.economy_profile.volatility_resistance())
            .unwrap_or(1.0)
    }

    pub fn faction_upkeep_burden(&self, faction: Faction) -> f32 {
        self.factions
            .get(&faction)
            .map(|meta| meta.economy_profile.upkeep_burden())
            .unwrap_or(1.0)
    }

    pub fn epoch_for_tick(&self, tick: u64) -> (&'static str, &'static str) {
        let day_segments = &self.epochs.day_segments;
        let seasons = &self.epochs.seasons;

        let day_segment = day_segments[(tick as usize) % day_segments.len()];
        let season = seasons[((tick / day_segments.len() as u64) as usize) % seasons.len()];

        (day_segment, season)
    }
}

impl Default for WorldMetadata {
    fn default() -> Self {
        use BehaviorState::*;

        let biomes = [
            (
                Biome::Forest,
                BiomeMetadata {
                    label: "Silken Veil Forest",
                    epithet: "Land of Whispering Canopies",
                    description:
                        "Ancient forest where herbs, hidden shrines, and fierce spirits coexist.",
                    anchor: (6.0, 4.5),
                    resource_profile: vec!["Herbs", "Lumber", "Rare Animals"],
                    tensions: vec!["Bandit Ambush", "Expedition Venture", "Shrine Guardian"],
                    behavior_bias: HashMap::from([
                        (Explore, 1.25),
                        (Gather, 1.2),
                        (Hunt, 1.15),
                        (Rest, 0.95),
                    ]),
                    economic_shift: EconomicShift {
                        trade_opportunity: 0.9,
                        resource_abundance: 1.2,
                        risk_factor: 1.1,
                    },
                },
            ),
            (
                Biome::Plains,
                BiomeMetadata {
                    label: "Silverwind Plains",
                    epithet: "Caravan Procession Under Vast Skies",
                    description:
                        "Vast grasslands with ceaseless caravans, crop rotation, and mounted patrols.",
                    anchor: (1.0, 2.0),
                    resource_profile: vec!["Grain", "Livestock", "Fiber"],
                    tensions: vec!["Harvest Dispute", "Beast Migration", "Caravan Toll"],
                    behavior_bias: HashMap::from([
                        (Trade, 1.2),
                        (Gather, 1.1),
                        (Explore, 0.95),
                        (Rest, 1.05),
                    ]),
                    economic_shift: EconomicShift {
                        trade_opportunity: 1.15,
                        resource_abundance: 1.05,
                        risk_factor: 0.9,
                    },
                },
            ),
            (
                Biome::Desert,
                BiomeMetadata {
                    label: "Ashen Mirage",
                    epithet: "Ruins Sleeping Under Dunes",
                    description:
                        "Desert intertwined with ancient ruins and dangerous mirages, testing all expeditions.",
                    anchor: (-4.0, -1.5),
                    resource_profile: vec!["Relics", "Minerals", "Glassroots"],
                    tensions: vec!["Water Scarcity", "Sandstorm", "Relic Scramble"],
                    behavior_bias: HashMap::from([
                        (Explore, 1.1),
                        (Hunt, 1.25),
                        (Gather, 0.85),
                        (Rest, 0.9),
                    ]),
                    economic_shift: EconomicShift {
                        trade_opportunity: 0.95,
                        resource_abundance: 0.8,
                        risk_factor: 1.35,
                    },
                },
            ),
            (
                Biome::Village,
                BiomeMetadata {
                    label: "Hearthfire Corridor",
                    epithet: "Heart of the Community",
                    description:
                        "A ring of villages where workshops, granaries, and temples are tightly connected.",
                    anchor: (3.5, -3.0),
                    resource_profile: vec!["Goods", "Craftsmanship", "Rituals"],
                    tensions: vec!["Civil Conflict", "Disease Spread", "Supply Shortage"],
                    behavior_bias: HashMap::from([
                        (Trade, 1.1),
                        (Rest, 1.2),
                        (Idle, 1.05),
                        (Gather, 1.0),
                    ]),
                    economic_shift: EconomicShift {
                        trade_opportunity: 1.05,
                        resource_abundance: 1.1,
                        risk_factor: 0.85,
                    },
                },
            ),
            (
                Biome::Market,
                BiomeMetadata {
                    label: "Golden Confluence",
                    epithet: "Pulse of Commerce",
                    description:
                        "Tiered market city where the Guild Council coordinates trade, tariffs, and truces.",
                    anchor: (0.0, 0.0),
                    resource_profile: vec!["Currency", "Contracts", "Intel"],
                    tensions: vec!["Tariff War", "Speculative Crash", "Guild Infighting"],
                    behavior_bias: HashMap::from([
                        (Trade, 1.35),
                        (Idle, 0.9),
                        (Explore, 0.95),
                        (Rest, 0.9),
                    ]),
                    economic_shift: EconomicShift {
                        trade_opportunity: 1.4,
                        resource_abundance: 0.9,
                        risk_factor: 1.05,
                    },
                },
            ),
        ]
        .into_iter()
        .collect();

        let factions = [
            (
                Faction::MerchantGuild,
                FactionMetadata {
                    motto: "Balance the ledger, stabilize the world.",
                    doctrine: "Focuses on trade diplomacy, caravan escort, and price adjustment.",
                    influence_vectors: vec!["Tariff Adjustment", "Supply Contract", "Credit Issuance"],
                    strongholds: vec![Biome::Market, Biome::Plains],
                    behavior_modifiers: HashMap::from([
                        (BehaviorState::Trade, 1.4),
                        (BehaviorState::Idle, 0.9),
                        (BehaviorState::Explore, 0.95),
                    ]),
                    economy_profile: EconomyProfile {
                        trade_yield: 1.35,
                        volatility_resistance: 1.1,
                        upkeep_burden: 1.0,
                    },
                },
            ),
            (
                Faction::BanditClans,
                FactionMetadata {
                    motto: "Seize what the world hides.",
                    doctrine: "Expands influence via asymmetric raids, terror tactics, and relic monopolies.",
                    influence_vectors: vec!["Ambush Threat", "Black Market", "Smuggling Network"],
                    strongholds: vec![Biome::Forest, Biome::Desert],
                    behavior_modifiers: HashMap::from([
                        (BehaviorState::Hunt, 1.45),
                        (BehaviorState::Explore, 1.1),
                        (BehaviorState::Trade, 0.7),
                        (BehaviorState::Rest, 0.85),
                    ]),
                    economy_profile: EconomyProfile {
                        trade_yield: 0.85,
                        volatility_resistance: 0.9,
                        upkeep_burden: 0.8,
                    },
                },
            ),
            (
                Faction::ExplorersLeague,
                FactionMetadata {
                    motto: "Map the unknown, grasp the invisible.",
                    doctrine: "Conducts reconnaissance, anomaly recording, and relic appraisal.",
                    influence_vectors: vec!["Discovery Rights", "Map Intel", "Relic Appraisal"],
                    strongholds: vec![Biome::Forest, Biome::Desert],
                    behavior_modifiers: HashMap::from([
                        (BehaviorState::Explore, 1.5),
                        (BehaviorState::Gather, 1.25),
                        (BehaviorState::Trade, 0.9),
                        (BehaviorState::Rest, 0.95),
                    ]),
                    economy_profile: EconomyProfile {
                        trade_yield: 1.05,
                        volatility_resistance: 0.95,
                        upkeep_burden: 1.1,
                    },
                },
            ),
            (
                Faction::SettlersUnion,
                FactionMetadata {
                    motto: "Rooted in labor, growing through craft.",
                    doctrine: "Leads cooperative labor, agricultural planning, and urban reconstruction.",
                    influence_vectors: vec!["Infrastructure", "Harvest Mgmt", "Community Festivals"],
                    strongholds: vec![Biome::Plains, Biome::Village],
                    behavior_modifiers: HashMap::from([
                        (BehaviorState::Gather, 1.35),
                        (BehaviorState::Trade, 1.1),
                        (BehaviorState::Idle, 1.05),
                        (BehaviorState::Hunt, 0.85),
                    ]),
                    economy_profile: EconomyProfile {
                        trade_yield: 1.15,
                        volatility_resistance: 1.05,
                        upkeep_burden: 1.2,
                    },
                },
            ),
            (
                Faction::TempleOfSuns,
                FactionMetadata {
                    motto: "Three suns, one harmonious light.",
                    doctrine: "Handles peace mediation, relic purification, and public welfare.",
                    influence_vectors: vec!["Healing Rituals", "Pilgrimage Net", "Moral Authority"],
                    strongholds: vec![Biome::Village, Biome::Market],
                    behavior_modifiers: HashMap::from([
                        (BehaviorState::Rest, 1.4),
                        (BehaviorState::Trade, 1.05),
                        (BehaviorState::Explore, 0.9),
                        (BehaviorState::Hunt, 0.7),
                    ]),
                    economy_profile: EconomyProfile {
                        trade_yield: 1.0,
                        volatility_resistance: 1.25,
                        upkeep_burden: 1.05,
                    },
                },
            ),
        ]
        .into_iter()
        .collect();

        let economy = EconomyMetadata {
            circulation_cycle: vec![
                "Market Auction",
                "Guild Bidding",
                "Village Services",
                "Desert Expedition",
                "Market Reflux",
            ],
            stressors: vec![
                "Drought Pressure",
                "Bandit Raid",
                "Currency Devaluation",
                "Relic Shortage",
                "Plague Spread",
            ],
            catalysts: vec![
                "Temple Festival",
                "Explorer Breakthrough",
                "Guild Tariff Cut",
                "Union Harvest",
            ],
        };

        let epochs = EpochCadence {
            day_segments: vec!["Dawn", "Midday", "Dusk"],
            seasons: vec!["Flower Bloom", "Sunburst Peak", "Ashfall"],
        };

        Self {
            biomes,
            factions,
            economy,
            epochs,
            tech_tree: TechTree::default(),
        }
    }
}

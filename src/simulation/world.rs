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
                    label: "비단숲 장막",
                    epithet: "수관이 속삭이는 땅",
                    description:
                        "약초와 숨겨진 성소, 사나운 정령이 공존하는 고대의 숲입니다.",
                    anchor: (6.0, 4.5),
                    resource_profile: vec!["약초", "목재", "희귀 동물"],
                    tensions: vec!["산적 매복", "탐험단 원정", "성소 수호령"],
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
                    label: "은바람 평야",
                    epithet: "드넓은 하늘 아래 대상 행렬",
                    description:
                        "대상 행렬과 윤작, 기마 순찰이 끊이지 않는 광활한 초원입니다.",
                    anchor: (1.0, 2.0),
                    resource_profile: vec!["곡물", "가축", "섬유"],
                    tensions: vec!["수확 분쟁", "맹수 이동", "대상 통행세"],
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
                    label: "잿빛 신기루",
                    epithet: "모래언덕 아래 잠든 유적",
                    description:
                        "고대 유적과 위험한 신기루가 뒤엉킨 사막으로, 모든 원정을 시험합니다.",
                    anchor: (-4.0, -1.5),
                    resource_profile: vec!["유물", "광물", "유리뿌리"],
                    tensions: vec!["물 부족", "모래폭풍", "유물 쟁탈"],
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
                    label: "난롯불 회랑",
                    epithet: "공동체의 심장부",
                    description:
                        "작업장과 곡창, 사원 의원이 촘촘히 연결된 마을의 고리입니다.",
                    anchor: (3.5, -3.0),
                    resource_profile: vec!["가공품", "공예 기술", "신앙 의례"],
                    tensions: vec!["시민 갈등", "질병 확산", "보급 부족"],
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
                    label: "황금 합류지",
                    epithet: "상업의 맥박",
                    description:
                        "길드 평의회가 거래·관세·외교 휴전을 조율하는 층층이 쌓인 시장 도시입니다.",
                    anchor: (0.0, 0.0),
                    resource_profile: vec!["화폐", "계약서", "정보"],
                    tensions: vec!["관세 전쟁", "투기 붕괴", "길드 암투"],
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
                    motto: "장부를 맞추고 세상을 안정시킨다.",
                    doctrine: "거래 외교, 대상 호위, 가격 조정을 핵심으로 삼습니다.",
                    influence_vectors: vec!["관세 조정", "공급 계약", "신용 발행"],
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
                    motto: "세상이 숨긴 것을 탈취하라.",
                    doctrine: "비대칭 기습과 공포 전술, 유물 독점으로 영향력을 넓힙니다.",
                    influence_vectors: vec!["매복 위협", "암시장", "밀수망"],
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
                    motto: "미지를 그리고 보이지 않는 것을 손에 쥔다.",
                    doctrine: "정찰 임무, 이상 지형 기록, 유물 감정을 수행합니다.",
                    influence_vectors: vec!["발견권", "지도 정보", "유물 감정"],
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
                    motto: "노동에 뿌리내리고 공예로 성장한다.",
                    doctrine: "협동 노동과 농업 계획, 도시 재건을 주도합니다.",
                    influence_vectors: vec!["인프라 건설", "수확 관리", "공동체 축제"],
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
                    motto: "세 개의 태양, 하나의 조화로운 빛.",
                    doctrine: "평화 중재와 유물 정화, 공공 복지를 맡습니다.",
                    influence_vectors: vec!["치유 의식", "순례망", "도덕적 권위"],
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
                "시장 경매",
                "상단 대상",
                "마을 서비스",
                "사막 원정",
                "시장 환류",
            ],
            stressors: vec![
                "가뭄 압박",
                "산적 급습",
                "화폐 절하",
                "유물 부족",
                "역병 확산",
            ],
            catalysts: vec!["사원 축제", "탐험가 돌파구", "길드 관세 인하", "연합 풍년"],
        };

        let epochs = EpochCadence {
            day_segments: vec!["새벽", "한낮", "해질녘"],
            seasons: vec!["꽃피움 계절", "불꽃 절정", "잿불 내림"],
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

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Era {
    Dawn,
    Ancient,
    Classical,
    Medieval,
    Industrial,
    Modern,
    Nuclear,
}

impl Era {
    pub fn label(&self) -> &'static str {
        match self {
            Era::Dawn => "Stone Age",
            Era::Ancient => "Bronze Age",
            Era::Classical => "Classical Era",
            Era::Medieval => "Medieval Era",
            Era::Industrial => "Industrial Era",
            Era::Modern => "Modern Era",
            Era::Nuclear => "Nuclear/Future",
        }
    }

    pub fn next(&self) -> Option<Era> {
        match self {
            Era::Dawn => Some(Era::Ancient),
            Era::Ancient => Some(Era::Classical),
            Era::Classical => Some(Era::Medieval),
            Era::Medieval => Some(Era::Industrial),
            Era::Industrial => Some(Era::Modern),
            Era::Modern => Some(Era::Nuclear),
            Era::Nuclear => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeaponTier {
    KnappedStone,
    PolishedStone,
    Bow,
    Crossbow,
    Gunpowder,
    SteelArmor,
    ModernArmor,
    NuclearArsenal,
}

impl WeaponTier {
    pub fn label(&self) -> &'static str {
        match self {
            WeaponTier::KnappedStone => "Knapped Stone",
            WeaponTier::PolishedStone => "Polished Stone",
            WeaponTier::Bow => "Bow",
            WeaponTier::Crossbow => "Crossbow",
            WeaponTier::Gunpowder => "Gunpowder",
            WeaponTier::SteelArmor => "Steel Armor",
            WeaponTier::ModernArmor => "Modern Tank",
            WeaponTier::NuclearArsenal => "Nuclear Arsenal",
        }
    }

    pub fn combat_multiplier(&self) -> f32 {
        match self {
            WeaponTier::KnappedStone => 0.75,
            WeaponTier::PolishedStone => 0.9,
            WeaponTier::Bow => 1.0,
            WeaponTier::Crossbow => 1.1,
            WeaponTier::Gunpowder => 1.25,
            WeaponTier::SteelArmor => 1.35,
            WeaponTier::ModernArmor => 1.55,
            WeaponTier::NuclearArsenal => 1.8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tech {
    Knapping,
    PolishedTools,
    Archery,
    Siegecraft,
    Metallurgy,
    GunpowderChemistry,
    Ballistics,
    NuclearPhysics,
}

impl Tech {
    pub fn label(&self) -> &'static str {
        match self {
            Tech::Knapping => "Knapping",
            Tech::PolishedTools => "Polished Tools",
            Tech::Archery => "Archery",
            Tech::Siegecraft => "Siegecraft",
            Tech::Metallurgy => "Metallurgy",
            Tech::GunpowderChemistry => "Gunpowder Chemistry",
            Tech::Ballistics => "Ballistics",
            Tech::NuclearPhysics => "Nuclear Physics",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EraTechTier {
    pub era: Era,
    pub science_gate: f32,
    pub culture_gate: f32,
    pub military_gate: f32,
    pub weapon_tier: WeaponTier,
    pub unlocks: Vec<Tech>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechTree {
    pub tiers: Vec<EraTechTier>,
}

impl TechTree {
    pub fn tier(&self, era: Era) -> Option<&EraTechTier> {
        self.tiers.iter().find(|tier| tier.era == era)
    }

    pub fn next_tier(&self, era: Era) -> Option<&EraTechTier> {
        era.next().and_then(|next| self.tier(next))
    }
}

impl Default for TechTree {
    fn default() -> Self {
        Self {
            tiers: vec![
                EraTechTier {
                    era: Era::Dawn,
                    science_gate: 0.0,
                    culture_gate: 0.0,
                    military_gate: 0.0,
                    weapon_tier: WeaponTier::KnappedStone,
                    unlocks: vec![Tech::Knapping],
                },
                EraTechTier {
                    era: Era::Ancient,
                    science_gate: 35.0,
                    culture_gate: 20.0,
                    military_gate: 10.0,
                    weapon_tier: WeaponTier::PolishedStone,
                    unlocks: vec![Tech::PolishedTools],
                },
                EraTechTier {
                    era: Era::Classical,
                    science_gate: 50.0,
                    culture_gate: 30.0,
                    military_gate: 18.0,
                    weapon_tier: WeaponTier::Bow,
                    unlocks: vec![Tech::Archery],
                },
                EraTechTier {
                    era: Era::Medieval,
                    science_gate: 65.0,
                    culture_gate: 40.0,
                    military_gate: 25.0,
                    weapon_tier: WeaponTier::Crossbow,
                    unlocks: vec![Tech::Siegecraft],
                },
                EraTechTier {
                    era: Era::Industrial,
                    science_gate: 75.0,
                    culture_gate: 55.0,
                    military_gate: 35.0,
                    weapon_tier: WeaponTier::Gunpowder,
                    unlocks: vec![Tech::Metallurgy, Tech::GunpowderChemistry],
                },
                EraTechTier {
                    era: Era::Modern,
                    science_gate: 85.0,
                    culture_gate: 65.0,
                    military_gate: 50.0,
                    weapon_tier: WeaponTier::ModernArmor,
                    unlocks: vec![Tech::Ballistics],
                },
                EraTechTier {
                    era: Era::Nuclear,
                    science_gate: 95.0,
                    culture_gate: 75.0,
                    military_gate: 70.0,
                    weapon_tier: WeaponTier::NuclearArsenal,
                    unlocks: vec![Tech::NuclearPhysics],
                },
            ],
        }
    }
}

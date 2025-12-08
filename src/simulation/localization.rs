//! Localization helpers for Korean display strings and color semantics.

use colored::Color;

use crate::simulation::{BehaviorState, Faction, Sentiment};

pub fn behavior_label(state: BehaviorState) -> &'static str {
    match state {
        BehaviorState::Idle => "Idle",
        BehaviorState::Explore => "Explore",
        BehaviorState::Gather => "Gather",
        BehaviorState::Trade => "Trade",
        BehaviorState::Hunt => "Hunt",
        BehaviorState::Rest => "Rest",
    }
}

pub fn faction_label(faction: Faction) -> &'static str {
    match faction {
        Faction::Neutral => "Neutral Alliance",
        Faction::MerchantGuild => "Merchant Guild",
        Faction::BanditClans => "Bandit Clans",
        Faction::ExplorersLeague => "Explorers League",
        Faction::SettlersUnion => "Settlers Union",
        Faction::TempleOfSuns => "Temple of Suns",
    }
}

pub fn behavior_color(state: BehaviorState) -> Color {
    match state {
        BehaviorState::Idle => Color::BrightBlack,
        BehaviorState::Explore => Color::BrightBlue,
        BehaviorState::Gather => Color::BrightGreen,
        BehaviorState::Trade => Color::BrightCyan,
        BehaviorState::Hunt => Color::BrightRed,
        BehaviorState::Rest => Color::Magenta,
    }
}

pub fn faction_color(faction: Faction) -> Color {
    match faction {
        Faction::Neutral => Color::White,
        Faction::MerchantGuild => Color::BrightYellow,
        Faction::BanditClans => Color::Red,
        Faction::ExplorersLeague => Color::Blue,
        Faction::SettlersUnion => Color::Green,
        Faction::TempleOfSuns => Color::Magenta,
    }
}

pub fn sentiment_label(sentiment: Sentiment) -> &'static str {
    match sentiment {
        Sentiment::Positive => "Positive",
        Sentiment::Neutral => "Neutral",
        Sentiment::Negative => "Negative",
    }
}

pub fn sentiment_color(sentiment: Sentiment) -> Color {
    match sentiment {
        Sentiment::Positive => Color::BrightGreen,
        Sentiment::Neutral => Color::Yellow,
        Sentiment::Negative => Color::BrightRed,
    }
}

pub fn format_number_commas(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    let mut count = 0;
    for ch in s.chars().rev() {
        if count > 0 && count % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
        count += 1;
    }
    out.chars().rev().collect()
}

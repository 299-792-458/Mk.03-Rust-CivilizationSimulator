//! Localization helpers for Korean display strings and color semantics.

use colored::Color;

use crate::simulation::{BehaviorState, Faction, Sentiment};

pub fn behavior_label(state: BehaviorState) -> &'static str {
    match state {
        BehaviorState::Idle => "휴식 대기",
        BehaviorState::Explore => "탐험",
        BehaviorState::Gather => "채집",
        BehaviorState::Trade => "거래",
        BehaviorState::Hunt => "사냥",
        BehaviorState::Rest => "회복",
    }
}

pub fn faction_label(faction: Faction) -> &'static str {
    match faction {
        Faction::Neutral => "중립 연합",
        Faction::MerchantGuild => "상인 길드",
        Faction::BanditClans => "산적 연맹",
        Faction::ExplorersLeague => "탐험가 연맹",
        Faction::SettlersUnion => "개척민 연합",
        Faction::TempleOfSuns => "태양의 성전",
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
        Sentiment::Positive => "긍정",
        Sentiment::Neutral => "중립",
        Sentiment::Negative => "부정",
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

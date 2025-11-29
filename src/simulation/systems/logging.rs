//! Colorized world pulse logging for quick CLI scanning.

use bevy_ecs::prelude::*;
use colored::{Color, Colorize};
use tracing::info;

use crate::simulation::{
    Behavior, Identity, Position, Sentiment, WorldEvent, WorldEventLog, WorldMetadata, WorldTime,
    behavior_color, behavior_label, faction_color, faction_label, format_number_commas,
    sentiment_color, sentiment_label,
};

fn badge(label: &str, color: Color) -> String {
    format!("[{}]", label).color(color).to_string()
}

fn category_color(category: &str) -> Color {
    match category {
        "무역" => Color::BrightCyan,
        "사회" => Color::BrightMagenta,
        "거시충격" => Color::BrightRed,
        "전쟁" => Color::Red,
        "시대" => Color::BrightBlue,
        "과학" => Color::BrightCyan,
        _ => Color::White,
    }
}

fn sentiment_tag(sentiment: Sentiment) -> String {
    badge(sentiment_label(sentiment), sentiment_color(sentiment))
}

fn format_event_line(event: &WorldEvent) -> String {
    let category_badge = badge(event.category(), category_color(event.category()));
    let sentiment_badge = sentiment_tag(event.sentiment());
    let tick_badge = badge(&format!("Tick {}", event.tick), Color::BrightBlack);
    let season_badge = badge(&event.season, Color::BrightBlue);
    let epoch_badge = badge(&event.epoch, Color::BrightCyan);

    match &event.kind {
        crate::simulation::WorldEventKind::Trade {
            actor,
            trade_focus,
            market_pressure,
        } => {
            let faction_badge = badge(&actor.faction_label, faction_color(actor.faction));
            let behavior_badge = badge(
                &actor.behavior_hint_label,
                behavior_color(actor.behavior_hint),
            );
            let actor_name = actor
                .name
                .color(faction_color(actor.faction))
                .bold()
                .to_string();
            let focus = trade_focus.color(Color::BrightCyan).to_string();
            let pressure = market_pressure.color(Color::Yellow).to_string();

            format!(
                "{} {} {} {} {} {} {} {} 님이 {} 거래를 주도합니다 | 압력: {}",
                category_badge,
                sentiment_badge,
                tick_badge,
                epoch_badge,
                season_badge,
                faction_badge,
                behavior_badge,
                actor_name,
                focus,
                pressure
            )
        }
        crate::simulation::WorldEventKind::Social {
            convener,
            gathering_theme,
            cohesion_level,
        } => {
            let faction_badge = badge(&convener.faction_label, faction_color(convener.faction));
            let behavior_badge = badge(
                &convener.behavior_hint_label,
                behavior_color(convener.behavior_hint),
            );
            let convener_name = convener
                .name
                .color(faction_color(convener.faction))
                .bold()
                .to_string();
            let theme = format!("\"{}\"", gathering_theme)
                .color(Color::BrightMagenta)
                .to_string();
            let cohesion = cohesion_level.color(Color::BrightGreen).to_string();

            format!(
                "{} {} {} {} {} {} {} {} 님이 {} 주제로 모임을 엽니다 | 응집도: {}",
                category_badge,
                sentiment_badge,
                tick_badge,
                epoch_badge,
                season_badge,
                faction_badge,
                behavior_badge,
                convener_name,
                theme,
                cohesion
            )
        }
        crate::simulation::WorldEventKind::MacroShock {
            stressor,
            catalyst,
            projected_impact,
            casualties,
        } => {
            let stress = stressor.color(Color::BrightRed).bold().to_string();
            let catalyst = catalyst.color(Color::Yellow).to_string();
            let impact = projected_impact.color(Color::White).to_string();
            let casualty = casualties
                .map(|c| format_number_commas(c))
                .map(|c| format!(" | 피해 {}명", c))
                .unwrap_or_default();

            format!(
                "{} {} {} {} {} {} | 촉발: {} | 영향: {}{}",
                category_badge,
                sentiment_badge,
                tick_badge,
                epoch_badge,
                season_badge,
                stress,
                catalyst,
                impact,
                casualty
            )
        }
        crate::simulation::WorldEventKind::Warfare {
            winner,
            loser,
            territory_change,
            casualties,
            nuclear,
        } => {
            let winner_badge = badge(winner.name(), winner.logging_color());
            let loser_badge = badge(loser.name(), loser.logging_color());
            let casualty_badge = badge(
                &format!("사상자 {}명", format_number_commas(*casualties)),
                Color::BrightRed,
            );
            let nuke_badge = if *nuclear {
                format!(" {} \u{0007}", badge("핵 공격", Color::Yellow))
            } else {
                String::new()
            };

            format!(
                "{} {} {} {} {} {}가 {}와의 전쟁에서 승리하여 영토 {:.2}를 획득했습니다. {}{}",
                category_badge,
                sentiment_badge,
                tick_badge,
                epoch_badge,
                season_badge,
                winner_badge,
                loser_badge,
                territory_change,
                casualty_badge,
                nuke_badge
            )
        }
        crate::simulation::WorldEventKind::EraShift {
            nation,
            era,
            weapon,
        } => {
            let nation_badge = badge(nation.name(), nation.logging_color());
            let era_badge = badge(era.label(), Color::BrightBlue);
            let weapon_badge = badge(weapon.label(), Color::Yellow);

            format!(
                "{} {} {} {} {} {} {} 시대로 진입 | 주력 무기 {}",
                category_badge,
                sentiment_badge,
                tick_badge,
                epoch_badge,
                season_badge,
                nation_badge,
                era_badge,
                weapon_badge
            )
        }
        crate::simulation::WorldEventKind::ScienceProgress { nation, progress } => {
            let nation_badge = badge(nation.name(), nation.logging_color());
            let progress_badge = badge(
                &format!("{:.1}% / 100%", progress.min(100.0)),
                Color::BrightCyan,
            );
            format!(
                "{} {} {} {} {} {} 달 탐사 진행",
                category_badge,
                sentiment_badge,
                tick_badge,
                epoch_badge,
                season_badge,
                format!("{} {}", nation_badge, progress_badge)
            )
        }
        crate::simulation::WorldEventKind::ScienceVictory { winner, progress } => {
            let winner_badge = badge(winner.name(), winner.logging_color());
            let progress_badge = badge(
                &format!("{:.1}% / 100%", progress.min(100.0)),
                Color::BrightGreen,
            );
            format!(
                "{} {} {} {} {} {} 인류 달 착륙 달성!",
                category_badge,
                sentiment_badge,
                tick_badge,
                epoch_badge,
                season_badge,
                format!("{} {}", winner_badge, progress_badge)
            )
        }
    }
}

fn format_sample_line(
    world_meta: &WorldMetadata,
    identity: &Identity,
    behavior: &Behavior,
    position: &Position,
) -> String {
    let sentiment_badge = sentiment_tag(Sentiment::Neutral);
    let category_badge = badge("상황", Color::BrightWhite);
    let faction_badge = badge(
        faction_label(identity.faction),
        faction_color(identity.faction),
    );
    let behavior_badge = badge(
        behavior_label(behavior.state),
        behavior_color(behavior.state),
    );

    let biome_meta = world_meta.biomes.get(&position.biome);
    let biome_label = biome_meta
        .map(|b| b.label.to_string())
        .unwrap_or_else(|| format!("{:?}", position.biome));
    let biome_badge = badge(&biome_label, Color::BrightBlue);
    let entity_name = identity
        .name
        .color(faction_color(identity.faction))
        .bold()
        .to_string();

    let mut line = format!(
        "{} {} {} {} {} {} 의 현재 상태를 관측 중",
        category_badge, sentiment_badge, faction_badge, behavior_badge, biome_badge, entity_name,
    );

    if let Some(meta) = biome_meta {
        let epithet_badge = badge(meta.epithet, Color::BrightBlue);
        let description = meta.description.color(Color::BrightBlack).to_string();
        line.push_str(&format!(" | {} {}", epithet_badge, description));
    }

    if let Some(faction_meta) = world_meta.faction_profile(identity.faction) {
        let motto_badge = badge(faction_meta.motto, Color::BrightYellow);
        let doctrine_badge = badge(faction_meta.doctrine, Color::Yellow);

        line.push_str(&format!(" | {} {}", motto_badge, doctrine_badge));

        if !faction_meta.strongholds.is_empty() {
            let stronghold_names = faction_meta
                .strongholds
                .iter()
                .map(|biome| {
                    world_meta
                        .biomes
                        .get(biome)
                        .map(|b| b.label)
                        .unwrap_or("미확인 거점")
                })
                .collect::<Vec<_>>()
                .join(", ");
            let stronghold_badge = badge(&format!("거점 {}", stronghold_names), Color::BrightGreen);
            line.push_str(&format!(" | {}", stronghold_badge));
        }
    }

    line
}

pub fn logging_system(
    time: Res<WorldTime>,
    world_meta: Res<WorldMetadata>,
    events: Res<WorldEventLog>,
    query: Query<(&Identity, &Behavior, &Position)>,
) {
    let (epoch, season) = world_meta.epoch_for_tick(time.tick);
    let catalyst_index = (time.tick as usize) % world_meta.economy.catalysts.len();
    let catalyst = world_meta.economy.catalysts[catalyst_index];
    let circulation_stage = world_meta
        .economy
        .circulation_cycle
        .get(catalyst_index % world_meta.economy.circulation_cycle.len())
        .copied()
        .unwrap_or("균형 거래");
    let stressor = world_meta
        .economy
        .stressors
        .get(catalyst_index % world_meta.economy.stressors.len())
        .copied()
        .unwrap_or("안정 국면");

    let header_line = format!(
        "{} {} {} {} {} {}",
        badge("세계", Color::BrightWhite),
        badge(&format!("Tick {}", time.tick), Color::BrightBlack),
        badge(epoch, Color::BrightCyan),
        badge(season, Color::BrightBlue),
        badge("순환", Color::BrightGreen),
        badge(circulation_stage, Color::BrightGreen),
    );

    let stress_line = format!(
        "{} {} {}",
        badge("촉매", Color::Yellow),
        badge(catalyst, Color::Yellow),
        badge(stressor, Color::BrightRed),
    );

    let mut lines = vec![header_line, stress_line];

    if let Some((identity, behavior, position)) = query.iter().next() {
        lines.push(format_sample_line(
            &world_meta,
            identity,
            behavior,
            position,
        ));
    }

    let recent_events = events
        .snapshot()
        .into_iter()
        .rev()
        .take(3)
        .map(|event| format_event_line(&event));

    let mut has_event = false;
    for line in recent_events {
        has_event = true;
        lines.push(line);
    }

    if !has_event {
        lines.push(
            "[이벤트] 최근 등록된 세계 이벤트가 없습니다"
                .color(Color::BrightBlack)
                .to_string(),
        );
    }

    let output = lines.join("\n");
    info!("\n{}", output);
}

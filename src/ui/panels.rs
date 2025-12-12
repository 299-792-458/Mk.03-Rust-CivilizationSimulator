use std::collections::HashMap;

use ratatui::{
    prelude::*,
    style::Stylize,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, Wrap},
};

use super::{ControlState, MODERN_THEME};
use crate::simulation::events::WorldEventKind;
use crate::simulation::{Nation, ObserverSnapshot, format_number_commas};
use crate::ui::charts::{heat_bar, render_evolutionary_charts, render_science_progress_panel};

pub fn render_world_state_panel(
    frame: &mut Frame,
    area: Rect,
    snapshot: &ObserverSnapshot,
    control: &ControlState,
) {
    let outer_block = Block::bordered()
        .title(" TACTICAL STATUS ")
        .title_style(Style::default().fg(MODERN_THEME.accent_a).bold())
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(MODERN_THEME.border));

    frame.render_widget(outer_block.clone(), area);
    let area = outer_block.inner(area); // Use inner area for content

    let panel_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(7),
            Constraint::Length(7),
            Constraint::Length(12),
            Constraint::Length(5),
            Constraint::Length(6),
            Constraint::Min(0),
            Constraint::Length(4),
        ])
        .split(area);

    let total_entities = snapshot.entities.len();
    let tick = snapshot.tick;
    let leader_name = snapshot
        .science_victory
        .leader
        .map(|n| n.name().to_string())
        .unwrap_or_else(|| "TBD".to_string());
    let leader_progress = snapshot
        .science_victory
        .leader_progress
        .min(snapshot.science_victory.goal);
    let gap = (snapshot.science_victory.leader_progress
        - snapshot.science_victory.runner_up_progress)
        .abs();
    let active_preset = control
        .preset_status
        .iter()
        .find(|p| p.active)
        .map(|p| format!("{} [{}]", p.label, p.key))
        .unwrap_or_else(|| "Custom".to_string());

    let info_lines = vec![
        Line::from(format!(
            "Ticks: {} | Entities: {} | Goal: Moon 100%",
            tick, total_entities
        )),
        Line::from(vec![
            Span::styled(
                if control.paused { "PAUSED" } else { "LIVE" },
                Style::default()
                    .fg(if control.paused {
                        Color::Yellow
                    } else {
                        Color::LightGreen
                    })
                    .bold(),
            ),
            Span::raw(" | "),
            Span::styled(
                format!("{} ms/tick", control.tick_duration.as_millis()),
                Style::default().fg(Color::White),
            ),
            Span::raw(" | "),
            Span::styled(
                format!("{:.0}y/tick", control.years_per_tick),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw(" | "),
            Span::styled(
                format!("Preset {}", active_preset),
                Style::default().fg(Color::Magenta),
            ),
        ]),
        Line::from(format!(
            "Epoch: {} | Season: {}",
            snapshot.epoch, snapshot.season
        )),
        Line::from(format!(
            "Atmosphere: {} (ΔT {:+.1}, Morale {:+.1}%, Yield {:+.1}%, Risk {:+.1}%)",
            snapshot.season_effect.label,
            snapshot.season_effect.temperature * 10.0,
            snapshot.season_effect.morale_shift,
            snapshot.season_effect.yield_shift,
            snapshot.season_effect.risk_shift
        )),
        Line::from(vec![
            Span::styled(
                format!("War Fatigue {:>5.1} ", snapshot.overlay.war_fatigue),
                Style::default().fg(Color::Red),
            ),
            Span::raw("| "),
            Span::styled(
                format!("Fallout {:>4.0} ", snapshot.overlay.fallout),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw("| "),
            Span::styled(
                format!(
                    "Richness {:>4.0}%",
                    snapshot.overlay.resource_richness * 100.0
                ),
                Style::default().fg(Color::Green),
            ),
            Span::raw("| "),
            Span::styled(
                format!(
                    "Climate {:.0}ppm / Risk {:.1}% / Bio {:.1}",
                    snapshot.science_victory.carbon_ppm,
                    snapshot.science_victory.climate_risk,
                    snapshot.science_victory.biodiversity
                ),
                Style::default().fg(Color::LightBlue),
            ),
        ]),
        Line::from(format!(
            "Science Victory: {} {:.1}% / 100% (Gap {:.1}p) | Interstellar {:.1}% / {:.0}%",
            leader_name,
            leader_progress,
            gap,
            snapshot.science_victory.interstellar_progress,
            snapshot.science_victory.interstellar_goal
        )),
        Line::from(format!(
            "World Portfolio: Population {} | GDP {:.1} | Events {}",
            format_number_commas(snapshot.science_victory.total_population),
            snapshot.science_victory.total_economy,
            snapshot.events.len()
        )),
        Line::from(match control.selected_hex {
            Some(hex) => {
                let owner = control
                    .selected_owner
                    .map(|n| n.name().to_string())
                    .unwrap_or_else(|| "Unclaimed".to_string());
                let war = snapshot.combat_hexes.contains(&hex);
                let nuke = snapshot.nuclear_hexes.contains(&hex);
                format!(
                    "Selected hex q:{} r:{} | Owner {} | Front {} | Nuke {}",
                    hex.q,
                    hex.r,
                    owner,
                    if war { "Yes" } else { "None" },
                    if nuke { "Yes" } else { "None" }
                )
            }
            None => "Selected hex None — Click map to select".to_string(),
        }),
    ];
    let info_paragraph = Paragraph::new(info_lines);
    frame.render_widget(info_paragraph, panel_layout[0]);

    render_science_progress_panel(frame, panel_layout[1], snapshot);
    render_evolutionary_charts(frame, panel_layout[2], snapshot);
    render_glory_tiles(frame, panel_layout[3], snapshot);
    render_war_theater_panel(frame, panel_layout[4], snapshot);

    let mut nations: Vec<_> = snapshot.all_metrics.0.keys().copied().collect();
    if let Some(focus) = control.pinned_nation.or(control.selected_owner) {
        nations.sort_by_key(|n| if *n == focus { 0 } else { 1 });
    } else {
        nations.sort_by_key(|a| a.name());
    }

    let nations_len = nations.len().max(1) as u32;
    let constraints: Vec<Constraint> = (0..nations.len())
        .map(|_| Constraint::Ratio(1, nations_len))
        .collect();
    let nations_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(panel_layout[5]);

    for (i, &nation) in nations.iter().enumerate() {
        if i >= nations_layout.len() {
            break;
        }

        if let Some(metrics) = snapshot.all_metrics.0.get(&nation) {
            let nation_color = nation.color();
            let is_selected = control.selected_owner == Some(nation);
            let mut nation_lines = vec![];
            let pin_label = if control.pinned_nation == Some(nation) {
                " [PIN]"
            } else {
                ""
            };
            nation_lines.push(Line::from(Span::styled(
                nation.name(),
                Style::default()
                    .bold()
                    .underlined()
                    .fg(nation_color)
                    .bg(if is_selected {
                        Color::Rgb(30, 30, 60)
                    } else {
                        Color::Reset
                    }),
            )));
            if !pin_label.is_empty() {
                nation_lines.push(Line::from(Span::styled(
                    pin_label,
                    Style::default().fg(Color::LightCyan).bold(),
                )));
            }

            add_diplomacy_lines(snapshot, &mut nation_lines, nation);
            nation_lines.push(Line::from(Span::styled(
                format!(
                    "  Era: {} | Weapon: {}",
                    metrics.era.label(),
                    metrics.weapon_tier.label()
                ),
                Style::default().fg(Color::Cyan),
            )));
            nation_lines.push(Line::from(Span::styled(
                format!("  Population: {}", format_number_commas(metrics.population)),
                Style::default().fg(Color::White),
            )));
            if let Some(civ_state) = snapshot.civ_state.0.get(&nation) {
                nation_lines.push(Line::from(Span::styled(
                    format!(
                        "  Cities: {} | Happiness: {:.1} | Production: {:.1}",
                        civ_state.cities, civ_state.happiness, civ_state.production
                    ),
                    Style::default().fg(Color::Yellow),
                )));
            }
            let tech_list = if metrics.unlocked_techs.is_empty() {
                "Tech None".to_string()
            } else {
                metrics
                    .unlocked_techs
                    .iter()
                    .map(|tech| tech.label())
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            nation_lines.push(Line::from(Span::styled(
                format!("  Techs: {}", tech_list),
                Style::default().fg(Color::White),
            )));

            if metrics.is_destroyed {
                nation_lines.push(Line::from(Span::styled(
                    "-- DESTROYED --",
                    Style::default().fg(Color::Red).italic(),
                )));
            } else {
                push_metric_bar(
                    &mut nation_lines,
                    "  Economy",
                    metrics.economy,
                    nation_color,
                );
                push_metric_bar(
                    &mut nation_lines,
                    "  Science (Science)",
                    metrics.science,
                    nation_color,
                );
                push_metric_bar(
                    &mut nation_lines,
                    "  Culture",
                    metrics.culture,
                    nation_color,
                );
                push_metric_bar(
                    &mut nation_lines,
                    "  Diplomacy (Diplomacy)",
                    metrics.diplomacy,
                    nation_color,
                );
                push_metric_bar(
                    &mut nation_lines,
                    "  Religion",
                    metrics.religion,
                    nation_color,
                );
                push_metric_bar(
                    &mut nation_lines,
                    "  Military",
                    metrics.military,
                    nation_color,
                );
                push_metric_bar(
                    &mut nation_lines,
                    "  Territory",
                    metrics.territory,
                    nation_color,
                );
            }
            let nation_paragraph = Paragraph::new(nation_lines).scroll((0, 0));
            frame.render_widget(nation_paragraph, nations_layout[i]);
        }
    }

    render_speed_panel(frame, panel_layout[6], control);
}

fn add_diplomacy_lines(
    snapshot: &ObserverSnapshot,
    lines: &mut Vec<Line<'static>>,
    nation: Nation,
) {
    let trust = snapshot
        .diplomacy
        .trust
        .iter()
        .find(|(n, _)| *n == nation)
        .map(|(_, v)| *v)
        .unwrap_or(40.0);
    let fear = snapshot
        .diplomacy
        .fear
        .iter()
        .find(|(n, _)| *n == nation)
        .map(|(_, v)| *v)
        .unwrap_or(35.0);
    let leaning = snapshot
        .overlay
        .ideology_leaning
        .iter()
        .find(|(n, _)| *n == nation)
        .map(|(_, v)| *v)
        .unwrap_or(50.0);
    let cohesion = snapshot
        .overlay
        .ideology_cohesion
        .iter()
        .find(|(n, _)| *n == nation)
        .map(|(_, v)| *v)
        .unwrap_or(50.0);
    let volatility = snapshot
        .overlay
        .ideology_volatility
        .iter()
        .find(|(n, _)| *n == nation)
        .map(|(_, v)| *v)
        .unwrap_or(20.0);
    lines.push(Line::from(Span::styled(
        format!(
            "  Trust {:.0} | Fear {:.0} | Ideology {:.0} / Cohesion {:.0} / Volatility {:.0}",
            trust, fear, leaning, cohesion, volatility
        ),
        Style::default().fg(Color::Gray),
    )));
}

fn push_metric_bar(lines: &mut Vec<Line<'static>>, label: &str, value: f32, color: Color) {
    lines.push(Line::from(Span::styled(
        label.to_string(),
        Style::default(),
    )));
    lines.push(create_bar(value, 100.0, 10, color));
}

pub fn render_event_leaderboard(frame: &mut Frame, area: Rect, snapshot: &ObserverSnapshot) {
    let mut counts: HashMap<&'static str, u64> = HashMap::new();
    let mut sentiment_score: HashMap<&'static str, i64> = HashMap::new();
    let mut casualties_score: HashMap<&'static str, u64> = HashMap::new();
    let mut recent_counts: HashMap<&'static str, u64> = HashMap::new();

    for (idx, event) in snapshot.events.iter().rev().take(120).enumerate() {
        let cat = event.category();
        *counts.entry(cat).or_default() += 1;
        let delta = match event.sentiment() {
            crate::simulation::Sentiment::Positive => 1,
            crate::simulation::Sentiment::Negative => -2,
            crate::simulation::Sentiment::Neutral => 0,
        };
        *sentiment_score.entry(cat).or_default() += delta;
        *casualties_score.entry(cat).or_default() += casualties_from_event(event);
        if idx < 20 {
            *recent_counts.entry(cat).or_default() += 1;
        }
    }

    let categories = [
        "War",
        "Trade",
        "Social",
        "MacroShock",
        "Science",
        "Space",
        "Era",
    ];
    let max_count = counts.values().cloned().max().unwrap_or(1);
    let max_casualties = casualties_score.values().cloned().max().unwrap_or(1).max(1);
    let total_events: u64 = counts.values().sum::<u64>().max(1);

    let rows: Vec<Row> = categories
        .iter()
        .map(|cat| {
            let count = counts.get(cat).cloned().unwrap_or(0);
            let score = sentiment_score.get(cat).cloned().unwrap_or(0);
            let casualties = casualties_score.get(cat).cloned().unwrap_or(0);
            let recent = recent_counts.get(cat).cloned().unwrap_or(0);
            let share = (count as f32 / total_events as f32) * 100.0;
            let bar = heat_bar(count, max_count, 12);
            let casualty_bar = heat_bar(
                // scale down casualty display to thousands for readability
                (casualties / 1_000).max(1),
                (max_casualties / 1_000).max(1),
                12,
            );
            Row::new(vec![
                Cell::from(*cat),
                Cell::from(count.to_string()),
                Cell::from(recent.to_string()),
                Cell::from(score.to_string()),
                Cell::from(format_number_commas(casualties)),
                Cell::from(format!("{share:4.1}%")),
                Cell::from(format!("{bar} | {casualty_bar}")),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(12),
            Constraint::Length(9),
            Constraint::Min(22),
        ],
    )
    .header(
        Row::new(vec![
            "Type",
            "Count",
            "Recent20",
            "Sentiment",
            "Casualties",
            "Share",
            "Count ▓ | Casualties ░",
        ])
        .style(Style::default().fg(Color::White).bold()),
    )
    .block(
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(MODERN_THEME.border))
            .title(" Event Leaderboard ")
            .title_style(Style::default().fg(MODERN_THEME.accent_b).bold()),
    );

    frame.render_widget(table, area);
}

fn casualties_from_event(event: &crate::simulation::WorldEvent) -> u64 {
    match &event.kind {
        WorldEventKind::Warfare { casualties, .. } => *casualties,
        WorldEventKind::MacroShock { casualties, .. } => casualties.unwrap_or(0),
        _ => 0,
    }
}

pub fn render_glory_tiles(frame: &mut Frame, area: Rect, snapshot: &ObserverSnapshot) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(MODERN_THEME.border))
        .title(" Hall of Fame ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let tiles_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
        ])
        .split(inner);

    let top_pop = snapshot
        .all_metrics
        .0
        .iter()
        .max_by_key(|(_, m)| m.population)
        .map(|(n, m)| (n, m.population));
    let top_gdp = snapshot.all_metrics.0.iter().max_by(|a, b| {
        a.1.economy
            .partial_cmp(&b.1.economy)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let science_leader = snapshot
        .science_victory
        .leader
        .map(|n| (n, snapshot.science_victory.leader_progress));

    let mut war_wins: HashMap<String, u32> = HashMap::new();
    for event in snapshot.events.iter().rev().take(200) {
        if let WorldEventKind::Warfare { winner, .. } = &event.kind {
            *war_wins.entry(winner.name().to_string()).or_default() += 1;
        }
    }
    let war_champ = war_wins
        .iter()
        .max_by_key(|(_, v)| **v)
        .map(|(name, wins)| (name.clone(), *wins));

    let cards = vec![
        (
            "Population Peak",
            top_pop
                .map(|(n, pop)| format!("{} | {}", n.name(), format_number_commas(pop)))
                .unwrap_or_else(|| "Data None".to_string()),
            Color::LightCyan,
        ),
        (
            "Economic Hegemon",
            top_gdp
                .map(|(n, m)| format!("{} | Economy {:.1}", n.name(), m.economy))
                .unwrap_or_else(|| "Data None".to_string()),
            Color::LightGreen,
        ),
        (
            "Science Leader",
            science_leader
                .map(|(n, p)| format!("{} | {:.1}% Moon", n.name(), p))
                .unwrap_or_else(|| "TBD".to_string()),
            Color::Yellow,
        ),
        (
            "War Win Rate",
            war_champ
                .map(|(n, w)| format!("{} | {} Wins", n, w))
                .unwrap_or_else(|| "Peace".to_string()),
            Color::LightRed,
        ),
    ];

    for (i, (title, body, color)) in cards.into_iter().enumerate() {
        if i < tiles_layout.len() {
            let lines = vec![
                Line::from(Span::styled(title, Style::default().fg(color).bold())),
                Line::from(body),
            ];
            frame.render_widget(Paragraph::new(lines), tiles_layout[i]);
        }
    }
}

pub fn render_war_theater_panel(frame: &mut Frame, area: Rect, snapshot: &ObserverSnapshot) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(MODERN_THEME.border))
        .title(" War Theater ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = vec![];
    lines.push(Line::from(vec![
        Span::styled(
            format!("War Fatigue {:.1}", snapshot.overlay.war_fatigue),
            Style::default().fg(Color::LightRed),
        ),
        Span::raw(" | Active Nukes "),
        Span::styled(
            snapshot.nuclear_hexes.len().to_string(),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" | Fronts "),
        Span::styled(
            snapshot.combat_hexes.len().to_string(),
            Style::default().fg(Color::Red),
        ),
    ]));

    let mut armies: Vec<_> = snapshot
        .all_metrics
        .0
        .iter()
        .map(|(nation, metrics)| (nation, metrics.military, metrics.territory))
        .collect();
    armies.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    for (nation, mil, terr) in armies.into_iter().take(3) {
        lines.push(Line::from(Span::styled(
            format!("{} Mil {:.1} / Terr {:.1}", nation.name(), mil, terr),
            Style::default().fg(nation.color()),
        )));
    }

    let mut recent_battles = snapshot
        .events
        .iter()
        .rev()
        .filter_map(|e| {
            if let WorldEventKind::Warfare {
                winner,
                loser,
                nuclear,
                casualties,
                ..
            } = &e.kind
            {
                Some((
                    winner.name().to_string(),
                    loser.name().to_string(),
                    *nuclear,
                    *casualties,
                ))
            } else {
                None
            }
        })
        .take(3)
        .collect::<Vec<_>>();

    if recent_battles.is_empty() {
        lines.push(Line::from("Recent Battles None — Focus on Moon Landing"));
    } else {
        lines.push(Line::from(Span::styled(
            "Recent Battles",
            Style::default().bold().fg(Color::White),
        )));
        for (win, lose, nuclear, casualties) in recent_battles.drain(..) {
            lines.push(Line::from(format!(
                "{} vs {}{} | Casualties {}",
                win,
                lose,
                if nuclear { " (Nuke)" } else { "" },
                format_number_commas(casualties)
            )));
        }
    }

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, inner);
}

pub fn render_speed_panel(frame: &mut Frame, area: Rect, control: &ControlState) {
    let mut speed_lines = vec![];
    speed_lines.push(Line::from(Span::styled(
        "Tick & Timescale",
        Style::default().bold(),
    )));
    speed_lines.push(Line::from(format!(
        "{} ms/tick | {:.0} y/tick",
        control.tick_duration.as_millis(),
        control.years_per_tick
    )));
    let ms = control.tick_duration.as_millis() as f32;
    let slider_pos = (ms.log10() / 4.0).clamp(0.0, 1.0);
    let slider_width = 20;
    let filled = (slider_pos * slider_width as f32).round() as usize;
    let mut bar = "━".repeat(filled.min(slider_width));
    bar.push_str(&"─".repeat(slider_width.saturating_sub(filled)));
    speed_lines.push(Line::from(format!("[{}] pace", bar)));
    speed_lines.push(Line::from(vec![
        Span::from("["),
        Span::styled("-", Style::default().fg(Color::Red).bold()),
        Span::from("] ["),
        Span::styled("+", Style::default().fg(Color::Green).bold()),
        Span::from("]  "),
        Span::styled("< >", Style::default().fg(Color::Cyan).bold()),
        Span::from("  "),
        Span::styled("Space/P", Style::default().fg(Color::Yellow).bold()),
        Span::from(" PAUSED/RESUME"),
    ]));
    speed_lines.push(Line::from(vec![
        Span::raw("1~4 Preset  |  "),
        Span::styled("R", Style::default().fg(Color::LightYellow).bold()),
        Span::raw(" Reset  |  Q Quit"),
    ]));
    let speed_paragraph = Paragraph::new(speed_lines);
    frame.render_widget(speed_paragraph, area);
}

fn create_bar(value: f32, max_value: f32, max_width: usize, color: Color) -> Line<'static> {
    let percentage = (value / max_value).clamp(0.0, 1.0);
    let width = (percentage * max_width as f32) as usize;
    let bar_text = "█".repeat(width);
    let padding = " ".repeat(max_width - width);

    let bar_span = Span::styled(bar_text, Style::default().fg(color));
    let padding_span = Span::raw(padding);
    let text_span = Span::from(format!(" {:.1}%", percentage * 100.0));

    Line::from(vec![
        Span::raw("["),
        bar_span,
        padding_span,
        Span::raw("]"),
        text_span,
    ])
}

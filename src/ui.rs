mod charts;
mod control;
mod map;
mod panels;

use crate::simulation::events::WorldEventKind;
use crate::simulation::{AxialCoord, Nation, ObserverSnapshot, format_number_commas};
use charts::render_indicator_grid;
use control::render_control_deck;
use map::MapWidget;
use panels::{
    render_event_leaderboard, render_glory_tiles, render_war_theater_panel, render_world_state_panel,
};
use ratatui::{
    prelude::*,
    style::Stylize,
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ControlState {
    pub paused: bool,
    pub tick_duration: Duration,
    pub years_per_tick: f64,
    pub preset_status: Vec<PresetStatus>,
    pub map_overlay: MapOverlay,
    pub selected_hex: Option<AxialCoord>,
    pub selected_owner: Option<Nation>,
    pub log_filter: LogFilter,
    pub pinned_nation: Option<Nation>,
    pub log_pin_selected: bool,
    pub focus_mode: bool,
}

#[derive(Debug, Clone)]
pub struct PresetStatus {
    pub key: char,
    pub label: &'static str,
    pub intent: &'static str,
    pub tick_ms: u64,
    pub years_per_tick: f64,
    pub active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapOverlay {
    Ownership,
    Climate,
    Conflict,
}

impl MapOverlay {
    pub fn label(&self) -> &'static str {
        match self {
            MapOverlay::Ownership => "Territory/Leader",
            MapOverlay::Climate => "Climate/Sea",
            MapOverlay::Conflict => "Conflict/Fatigue",
        }
    }

    pub fn next(self) -> Self {
        match self {
            MapOverlay::Ownership => MapOverlay::Climate,
            MapOverlay::Climate => MapOverlay::Conflict,
            MapOverlay::Conflict => MapOverlay::Ownership,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            MapOverlay::Ownership => MapOverlay::Conflict,
            MapOverlay::Climate => MapOverlay::Ownership,
            MapOverlay::Conflict => MapOverlay::Climate,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFilter {
    All,
    War,
    TradeSocial,
    ScienceSpace,
    Diplomacy,
}

impl LogFilter {
    pub fn label(&self) -> &'static str {
        match self {
            LogFilter::All => "All",
            LogFilter::War => "War",
            LogFilter::TradeSocial => "Trade/Social",
            LogFilter::ScienceSpace => "Science/Space",
            LogFilter::Diplomacy => "Diplomacy",
        }
    }

    pub fn next(self) -> Self {
        match self {
            LogFilter::All => LogFilter::War,
            LogFilter::War => LogFilter::TradeSocial,
            LogFilter::TradeSocial => LogFilter::ScienceSpace,
            LogFilter::ScienceSpace => LogFilter::Diplomacy,
            LogFilter::Diplomacy => LogFilter::All,
        }
    }
}

/// Renders UI and returns the map area used for click mapping.
pub fn render(frame: &mut Frame, snapshot: &ObserverSnapshot, control: &ControlState) -> Rect {
    // Main layout
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(6),
            Constraint::Min(0),
        ])
        .split(frame.size());

    // Header
    let mut header_lines = vec![
        Line::from(vec![
            Span::styled(" Mk.03 Rust Studio - TERA ", Style::default().bold()),
            Span::raw(" | "),
            Span::styled(
                format!("{} / {}", snapshot.epoch, snapshot.season),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::raw("Atmosphere: "),
            Span::styled(
                &snapshot.season_effect.label,
                Style::default().fg(Color::Yellow).bold(),
            ),
            Span::raw("  "),
            Span::styled(
                format!(
                    "Temp {:+.1}  Morale {:+.1}%  Yield {:+.1}%  Risk {:+.1}%",
                    snapshot.season_effect.temperature * 10.0,
                    snapshot.season_effect.morale_shift,
                    snapshot.season_effect.yield_shift,
                    snapshot.season_effect.risk_shift
                ),
                Style::default().fg(Color::White),
            ),
        ]),
    ];
    if snapshot.science_victory.finished {
        header_lines.push(Line::from(Span::styled(
            "Spacefaring civilization achieved — simulation stabilizing",
            Style::default().fg(Color::LightGreen).bold(),
        )));
    } else if snapshot.science_victory.interstellar_mode {
        header_lines.push(Line::from(Span::styled(
            "Interstellar expansion underway",
            Style::default().fg(Color::Cyan).bold(),
        )));
    }
    header_lines.push(Line::from(vec![
        Span::styled("Chronicle ", Style::default().fg(Color::LightYellow).bold()),
        Span::raw("→ "),
        Span::styled(
            narrative_ticker(snapshot),
            Style::default().fg(Color::White),
        ),
    ]));
    header_lines.push(Line::from(vec![
        Span::styled(
            format!(
                "Cosmic {:.2}e8 yrs",
                snapshot.cosmic_age_years / 100_000_000.0
            ),
            Style::default().fg(Color::LightBlue),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("Stage {}", snapshot.geologic_stage),
            Style::default().fg(Color::Cyan),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("Extinctions {}", snapshot.extinction_events),
            Style::default().fg(Color::Magenta),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("Scale {:.0}y/tick", snapshot.timescale_years_per_tick),
            Style::default().fg(Color::Gray),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("Sea {:.0}%", snapshot.overlay.sea_level * 100.0),
            Style::default().fg(Color::Blue),
        ),
        Span::raw(" | "),
        Span::styled(
            format!(
                "Alliances {} · Sanctions {}",
                snapshot.diplomacy.alliances.len(),
                snapshot.diplomacy.sanctions.len()
            ),
            Style::default().fg(Color::Magenta),
        ),
    ]));
    if let Some(pin) = control.pinned_nation {
        header_lines.push(Line::from(vec![
            Span::styled("PIN ", Style::default().fg(Color::LightCyan).bold()),
            Span::raw(pin.name()),
        ]));
    }

    let header_paragraph = Paragraph::new(header_lines).block(Block::new().borders(Borders::TOP));
    frame.render_widget(header_paragraph, main_layout[0]);
    render_control_deck(frame, main_layout[1], snapshot, control);

    // Create a vertical layout for the main content area
    let content_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(52),
            Constraint::Length(12),
            Constraint::Percentage(36),
        ])
        .split(main_layout[2]);

    // Top layout for world state and map
    let top_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(content_layout[0]);

    // World State Panel
    render_world_state_panel(frame, top_layout[0], snapshot, control);

    // Map Widget
    let map_widget = MapWidget {
        snapshot,
        overlay: control.map_overlay,
        selected_hex: control.selected_hex,
        focus: control
            .focus_mode
            .then(|| control.pinned_nation.or(control.selected_owner))
        .flatten(),
    };
    frame.render_widget(map_widget, top_layout[1]);

    render_indicator_grid(frame, content_layout[1], snapshot);

    // Event Log Panel - Using a Table for alignment
    let header_cells = [
        "Nation",
        "Tick",
        "Category",
        "Actor/Source",
        "Details",
        "Impact/Level",
    ]
    .iter()
    .map(|h| Cell::from(*h).style(Style::default().fg(Color::White).bold()));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows: Vec<Row> = snapshot
        .events
        .iter()
        .rev()
        .filter(|e| {
            filter_event(
                e,
                control.log_filter,
                control
                    .log_pin_selected
                    .then(|| control.pinned_nation)
                    .flatten(),
            )
        })
        .take(20)
        .map(|event| {
            let (nation_cell, style) = match &event.kind {
                WorldEventKind::Trade { actor, .. } => {
                    let color = actor.nation.color();
                    (
                        Cell::from(actor.nation.name()).style(Style::default().fg(color)),
                        Style::default().fg(Color::Green),
                    )
                }
                WorldEventKind::Social { convener, .. } => {
                    let color = convener.nation.color();
                    (
                        Cell::from(convener.nation.name()).style(Style::default().fg(color)),
                        Style::default().fg(Color::Green),
                    )
                }
                WorldEventKind::MacroShock { .. } => {
                    (Cell::from("System"), Style::default().fg(Color::Yellow))
                }
                WorldEventKind::Warfare { winner, .. } => {
                    let color = winner.color();
                    (
                        Cell::from(winner.name()).style(Style::default().fg(color)),
                        Style::default().fg(Color::Red),
                    )
                }
                WorldEventKind::EraShift { nation, .. } => {
                    let color = nation.color();
                    (
                        Cell::from(nation.name()).style(Style::default().fg(color)),
                        Style::default().fg(Color::Cyan),
                    )
                }
                WorldEventKind::ScienceProgress { nation, .. } => {
                    let color = nation.color();
                    (
                        Cell::from(nation.name()).style(Style::default().fg(color)),
                        Style::default().fg(Color::LightCyan),
                    )
                }
                WorldEventKind::ScienceVictory { winner, .. } => {
                    let color = winner.color();
                    (
                        Cell::from(winner.name()).style(Style::default().fg(color)),
                        Style::default().fg(Color::Green),
                    )
                }
                WorldEventKind::InterstellarProgress { leader, .. } => {
                    let color = leader.color();
                    (
                        Cell::from(leader.name()).style(Style::default().fg(color)),
                        Style::default().fg(Color::Cyan),
                    )
                }
                WorldEventKind::InterstellarVictory { winner, .. } => {
                    let color = winner.color();
                    (
                        Cell::from(winner.name()).style(Style::default().fg(color)),
                        Style::default().fg(Color::LightGreen),
                    )
                }
            };

            let pinned_hit = control
                .pinned_nation
                .map(|p| event_involves(event, p))
                .unwrap_or(false);

            let (actor, details, impact) = match &event.kind {
                WorldEventKind::Trade {
                    actor,
                    trade_focus,
                    market_pressure,
                } => (
                    actor.name.clone(),
                    trade_focus.clone(),
                    market_pressure.clone(),
                ),
                WorldEventKind::Social {
                    convener,
                    gathering_theme,
                    cohesion_level,
                } => (
                    convener.name.clone(),
                    gathering_theme.clone(),
                    cohesion_level.clone(),
                ),
                WorldEventKind::MacroShock {
                    stressor,
                    catalyst,
                    projected_impact,
                    casualties,
                } => {
                    let casualty_str = casualties
                        .map(|c| format!("Casualties {}", format_number_commas(c)))
                        .unwrap_or_else(|| "Casualties None".to_string());
                    (
                        stressor.clone(),
                        catalyst.clone(),
                        format!("{projected_impact} | {casualty_str}"),
                    )
                }
                WorldEventKind::Warfare {
                    winner,
                    loser,
                    territory_change,
                    casualties,
                    nuclear,
                } => (
                    winner.name().to_string(),
                    format!("vs {}", loser.name()),
                    format!(
                        "+{:.2} territory | Casualties {}{}",
                        territory_change,
                        format_number_commas(*casualties),
                        if *nuclear { " | Nuke" } else { "" }
                    ),
                ),
                WorldEventKind::EraShift {
                    nation,
                    era,
                    weapon,
                } => (
                    nation.name().to_string(),
                    era.label().to_string(),
                    weapon.label().to_string(),
                ),
                WorldEventKind::ScienceProgress { nation, progress } => (
                    nation.name().to_string(),
                    "Moon Exploration".to_string(),
                    format!("{progress:.1}% Achieved"),
                ),
                WorldEventKind::ScienceVictory { winner, progress } => (
                    winner.name().to_string(),
                    "Science Victory".to_string(),
                    format!("{progress:.1}% Complete"),
                ),
                WorldEventKind::InterstellarProgress { leader, progress } => (
                    leader.name().to_string(),
                    "Interstellar Migration".to_string(),
                    format!("{progress:.1}% Achieved"),
                ),
                WorldEventKind::InterstellarVictory { winner, progress } => (
                    winner.name().to_string(),
                    "Interstellar Victory".to_string(),
                    format!("{progress:.1}% Complete"),
                ),
            };

            let cells = vec![
                nation_cell,
                Cell::from(event.tick.to_string()),
                Cell::from(event.category()),
                Cell::from(actor),
                Cell::from(details),
                Cell::from(impact),
            ];

            let mut row_style = style;
            if pinned_hit {
                row_style = row_style.bold().fg(Color::White);
            }
            Row::new(cells).height(1).style(row_style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Length(5),
            Constraint::Length(10),
            Constraint::Length(15),
            Constraint::Min(22),
            Constraint::Length(18),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title("Event Log — Check Left Panel for War Fatigue/Resource Richness")
            .borders(Borders::ALL),
    );

    let event_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Min(0),
        ])
        .split(content_layout[1]);

    render_diagnostics_strip(frame, event_layout[0], snapshot, control);
    render_event_leaderboard(frame, event_layout[1], snapshot);
    frame.render_widget(table, event_layout[2]);
    top_layout[1]
}

fn render_diagnostics_strip(
    frame: &mut Frame,
    area: Rect,
    snapshot: &ObserverSnapshot,
    control: &ControlState,
) {
    let war_delta = snapshot
        .overlay
        .war_fatigue_history
        .iter()
        .rev()
        .take(2)
        .collect::<Vec<_>>();
    let war_trend = if war_delta.len() == 2 {
        war_delta[0] - war_delta[1]
    } else {
        0.0
    };
    let carbon_delta = snapshot
        .overlay
        .carbon_history
        .iter()
        .rev()
        .take(2)
        .collect::<Vec<_>>();
    let carbon_trend = if carbon_delta.len() == 2 {
        carbon_delta[0] - carbon_delta[1]
    } else {
        0.0
    };
    let pop_delta = snapshot
        .science_victory
        .population_history
        .iter()
        .rev()
        .take(2)
        .collect::<Vec<_>>();
    let pop_trend = if pop_delta.len() == 2 {
        (*pop_delta[0] as i64 - *pop_delta[1] as i64) as f32
    } else {
        0.0
    };

    let lines = vec![Line::from(vec![
        Span::styled(
            format!(
                "Log {}{}",
                control.log_filter.label(),
                if control.log_pin_selected {
                    " (PIN)"
                } else {
                    ""
                }
            ),
            Style::default().fg(Color::Cyan).bold(),
        ),
        Span::raw(" · PIN "),
        Span::styled(
            control
                .pinned_nation
                .map(|n| n.name().to_string())
                .unwrap_or_else(|| "None".to_string()),
            Style::default().fg(Color::Magenta),
        ),
        Span::raw(" · War Fatigue Δ "),
        Span::styled(
            format!("{:+.2}", war_trend),
            Style::default().fg(Color::Red),
        ),
        Span::raw(" · Carbon Δ "),
        Span::styled(
            format!("{:+.1}", carbon_trend),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" · Population Δ "),
        Span::styled(
            format_number_commas(pop_trend.max(0.0) as u64),
            Style::default().fg(Color::Green),
        ),
        Span::raw(" · Alliances "),
        Span::styled(
            snapshot.diplomacy.alliances.len().to_string(),
            Style::default().fg(Color::Magenta),
        ),
        Span::raw(" · Sanctions "),
        Span::styled(
            snapshot.diplomacy.sanctions.len().to_string(),
            Style::default().fg(Color::Gray),
        ),
    ])];
    frame.render_widget(Paragraph::new(lines), area);
}

fn narrative_ticker(snapshot: &ObserverSnapshot) -> String {
    let mut snippets = Vec::new();
    for event in snapshot.events.iter().rev().take(3) {
        let snippet = match &event.kind {
            WorldEventKind::Trade {
                actor, trade_focus, ..
            } => {
                format!("{} Trade — {}", actor.nation.name(), trade_focus)
            }
            WorldEventKind::Social {
                convener,
                gathering_theme,
                ..
            } => {
                format!("{} Gathering — {}", convener.nation.name(), gathering_theme)
            }
            WorldEventKind::MacroShock {
                stressor,
                projected_impact,
                ..
            } => {
                format!("Shock {} → {}", stressor, projected_impact)
            }
            WorldEventKind::Warfare {
                winner,
                loser,
                nuclear,
                ..
            } => {
                format!(
                    "{} {} {}{}",
                    winner.name(),
                    if *nuclear { "Nuke" } else { "War" },
                    loser.name(),
                    if *nuclear { "!" } else { "" }
                )
            }
            WorldEventKind::EraShift { nation, era, .. } => {
                format!("{} Era Rise → {}", nation.name(), era.label())
            }
            WorldEventKind::ScienceProgress { nation, progress } => {
                format!("{} Moon {:.0}%", nation.name(), progress)
            }
            WorldEventKind::ScienceVictory { winner, .. } => {
                format!("{} Science Victory", winner.name())
            }
            WorldEventKind::InterstellarProgress { leader, progress } => {
                format!("{} Interstellar {:.0}%", leader.name(), progress)
            }
            WorldEventKind::InterstellarVictory { winner, .. } => {
                format!("{} Space Civ", winner.name())
            }
        };
        snippets.push(snippet);
    }

    if snippets.is_empty() {
        "Quiet Moment — collecting data".to_string()
    } else {
        snippets.join(" · ")
    }
}

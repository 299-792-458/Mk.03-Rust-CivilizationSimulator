mod charts;
mod control;
mod map;
mod panels;

use crate::simulation::events::WorldEventKind;
use crate::simulation::{AxialCoord, Nation, ObserverSnapshot, format_number_commas};
use charts::render_indicator_grid;
use control::render_control_deck;
use map::MapWidget;
use panels::{render_event_leaderboard, render_world_state_panel};
use ratatui::{
    prelude::*,
    style::Stylize,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table},
};
use std::time::Duration;

// --- MODERN THEME DEFINITION ---
pub struct Theme {
    pub bg: Color,
    pub panel_bg: Color,
    pub border: Color,
    pub text_main: Color,
    pub text_dim: Color,
    pub accent_a: Color, // Cyan-ish
    pub accent_b: Color, // Pink-ish
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
}

pub const MODERN_THEME: Theme = Theme {
    bg: Color::Rgb(10, 10, 15), // Deep dark blue-black
    panel_bg: Color::Rgb(15, 15, 22),
    border: Color::Rgb(60, 70, 90),       // Slate gray
    text_main: Color::Rgb(225, 230, 240), // Off-white
    text_dim: Color::Rgb(120, 130, 150),  // Dim gray
    accent_a: Color::Rgb(0, 190, 255),    // Cyan
    accent_b: Color::Rgb(255, 50, 150),   // Pink/Magenta
    success: Color::Rgb(50, 255, 120),
    warning: Color::Rgb(255, 200, 50),
    danger: Color::Rgb(255, 60, 80),
};

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
    // Force a uniform dark background using Clear + Block
    let full_bg = Block::new().style(Style::default().bg(MODERN_THEME.bg));
    frame.render_widget(Clear, frame.size());
    frame.render_widget(full_bg, frame.size());

    // Main layout with slight margin to frame the UI
    let root_area = frame.size();
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header (Compact)
            Constraint::Length(5), // Control Deck
            Constraint::Min(0),    // Main Content
        ])
        .split(root_area);

    // --- Header ---
    let mut header_lines = vec![
        Line::from(vec![
            Span::styled(
                " COMMAND BRIDGE ",
                Style::default()
                    .fg(MODERN_THEME.bg)
                    .bg(MODERN_THEME.accent_a)
                    .bold(),
            ),
            Span::raw(" "),
            Span::styled(
                format!("Epoch {} / {}", snapshot.epoch, snapshot.season),
                Style::default().fg(MODERN_THEME.text_main),
            ),
            Span::raw("  "),
            Span::styled(
                format!("COSMIC {:.2}e8y", snapshot.cosmic_age_years / 100_000_000.0),
                Style::default().fg(MODERN_THEME.text_dim),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                format!(" {} ", snapshot.season_effect.label),
                Style::default().fg(MODERN_THEME.warning).bold(),
            ),
            Span::styled(
                format!(
                    " ΔT {:+.1}  Morale {:+.1}%  Yield {:+.1}%  Risk {:+.1}%",
                    snapshot.season_effect.temperature * 10.0,
                    snapshot.season_effect.morale_shift,
                    snapshot.season_effect.yield_shift,
                    snapshot.season_effect.risk_shift
                ),
                Style::default().fg(MODERN_THEME.text_dim),
            ),
            Span::raw("  "),
            Span::styled(
                narrative_ticker(snapshot),
                Style::default().fg(MODERN_THEME.text_main).italic(),
            ),
        ]),
    ];

    // Add victory status if applicable
    if snapshot.science_victory.finished {
        header_lines[0].spans.push(Span::styled(
            " [VICTORY ACHIEVED]",
            Style::default().fg(MODERN_THEME.success).bold(),
        ));
    }

    let header_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(MODERN_THEME.border))
        .style(Style::default().bg(MODERN_THEME.bg)); // Blend with bg

    let header_paragraph = Paragraph::new(header_lines).block(header_block);
    frame.render_widget(header_paragraph, main_layout[0]);

    // --- Control Deck ---
    render_control_deck(frame, main_layout[1], snapshot, control);

    // --- Main Content (Map + Panels) ---
    // Split into Left (Map + Graphs) and Right (Status + Events)
    let content_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main_layout[2]);

    // Left Column: Map (Top) + Graphs (Bottom)
    let left_column = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(content_layout[0]);

    // Right Column: World State (Top) + Events (Bottom)
    let right_column = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(content_layout[1]);

    // --- Map Widget ---
    let map_widget = MapWidget {
        snapshot,
        overlay: control.map_overlay,
        selected_hex: control.selected_hex,
        focus: control
            .focus_mode
            .then(|| control.pinned_nation.or(control.selected_owner))
            .flatten(),
    };
    let map_block = Block::bordered()
        .border_type(BorderType::Rounded)
        .title(" THEATER MAP ")
        .title_style(Style::default().fg(MODERN_THEME.accent_a).bold())
        .border_style(Style::default().fg(MODERN_THEME.border))
        .style(Style::default().bg(MODERN_THEME.bg)); // Ensure map bg is dark

    let map_area = map_block.inner(left_column[0]);
    frame.render_widget(map_block, left_column[0]);
    frame.render_widget(map_widget, map_area);

    // --- Graphs ---
    // Wrap graphs in a block for cleaner look
    let graph_block = Block::bordered()
        .border_type(BorderType::Rounded)
        .title(" METRICS ")
        .border_style(Style::default().fg(MODERN_THEME.border));
    let graph_area = graph_block.inner(left_column[1]);
    frame.render_widget(graph_block, left_column[1]);
    render_indicator_grid(frame, graph_area, snapshot);

    // --- World State Panel ---
    render_world_state_panel(frame, right_column[0], snapshot, control);

    // --- Event Log Panel ---
    // Event Log - Using a Table for alignment
    let header_cells = [
        "Nation",
        "Tick",
        "Category",
        "Actor/Source",
        "Details",
        "Impact",
    ]
    .iter()
    .map(|h| Cell::from(*h).style(Style::default().fg(MODERN_THEME.text_dim).bold()));
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
            let (nation_cell, base_color) = match &event.kind {
                WorldEventKind::Trade { actor, .. } => (
                    Cell::from(actor.nation.name())
                        .style(Style::default().fg(actor.nation.color())),
                    MODERN_THEME.success,
                ),
                WorldEventKind::Social { convener, .. } => (
                    Cell::from(convener.nation.name())
                        .style(Style::default().fg(convener.nation.color())),
                    MODERN_THEME.accent_a,
                ),
                WorldEventKind::MacroShock { .. } => (Cell::from("System"), MODERN_THEME.warning),
                WorldEventKind::Warfare { winner, .. } => (
                    Cell::from(winner.name()).style(Style::default().fg(winner.color())),
                    MODERN_THEME.danger,
                ),
                WorldEventKind::EraShift { nation, .. } => (
                    Cell::from(nation.name()).style(Style::default().fg(nation.color())),
                    MODERN_THEME.accent_b,
                ),
                WorldEventKind::ScienceProgress { nation, .. } => (
                    Cell::from(nation.name()).style(Style::default().fg(nation.color())),
                    MODERN_THEME.accent_a,
                ),
                WorldEventKind::ScienceVictory { winner, .. } => (
                    Cell::from(winner.name()).style(Style::default().fg(winner.color())),
                    MODERN_THEME.success,
                ),
                WorldEventKind::InterstellarProgress { leader, .. } => (
                    Cell::from(leader.name()).style(Style::default().fg(leader.color())),
                    MODERN_THEME.accent_a,
                ),
                WorldEventKind::InterstellarVictory { winner, .. } => (
                    Cell::from(winner.name()).style(Style::default().fg(winner.color())),
                    MODERN_THEME.success,
                ),
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
                        .map(|c| format!("Kill {}", format_number_commas(c)))
                        .unwrap_or_else(|| "".to_string());
                    (
                        stressor.clone(),
                        catalyst.clone(),
                        format!("{} {}", projected_impact, casualty_str),
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
                        "+{:.1}km² | Kill {}{}",
                        territory_change,
                        format_number_commas(*casualties),
                        if *nuclear { " [NUKE]" } else { "" }
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
                    "Moon Project".to_string(),
                    format!("{progress:.1}%"),
                ),
                WorldEventKind::ScienceVictory { winner, progress } => (
                    winner.name().to_string(),
                    "Science Win".to_string(),
                    format!("{progress:.1}%"),
                ),
                WorldEventKind::InterstellarProgress { leader, progress } => (
                    leader.name().to_string(),
                    "Interstellar".to_string(),
                    format!("{progress:.1}%"),
                ),
                WorldEventKind::InterstellarVictory { winner, progress } => (
                    winner.name().to_string(),
                    "Galactic Win".to_string(),
                    format!("{progress:.1}%"),
                ),
            };

            let cells = vec![
                nation_cell,
                Cell::from(event.tick.to_string()),
                Cell::from(event.category()).style(Style::default().fg(base_color)),
                Cell::from(actor),
                Cell::from(details),
                Cell::from(impact),
            ];

            let mut row_style = Style::default().fg(MODERN_THEME.text_main);
            if pinned_hit {
                row_style = row_style.bg(Color::Rgb(20, 20, 40)).bold();
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
            Constraint::Min(20),
            Constraint::Length(20),
        ],
    )
    .header(header)
    .block(
        Block::bordered()
            .border_type(BorderType::Rounded)
            .title(" SIGINT FEED ")
            .title_style(Style::default().fg(MODERN_THEME.accent_b).bold())
            .border_style(Style::default().fg(MODERN_THEME.border))
            .style(Style::default().bg(MODERN_THEME.bg)),
    );

    let event_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Diagnostics
            Constraint::Length(8), // Leaderboard (Slightly larger)
            Constraint::Min(0),    // Table
        ])
        .split(right_column[1]);

    render_diagnostics_strip(frame, event_layout[0], snapshot, control);
    render_event_leaderboard(frame, event_layout[1], snapshot);
    frame.render_widget(table, event_layout[2]);

    map_area
}

fn filter_event(
    event: &crate::simulation::WorldEvent,
    filter: LogFilter,
    pinned: Option<crate::simulation::Nation>,
) -> bool {
    let passes = match filter {
        LogFilter::All => true,
        LogFilter::War => matches!(event.kind, WorldEventKind::Warfare { .. }),
        LogFilter::TradeSocial => matches!(
            event.kind,
            WorldEventKind::Trade { .. } | WorldEventKind::Social { .. }
        ),
        LogFilter::ScienceSpace => matches!(
            event.kind,
            WorldEventKind::ScienceProgress { .. }
                | WorldEventKind::ScienceVictory { .. }
                | WorldEventKind::InterstellarProgress { .. }
                | WorldEventKind::InterstellarVictory { .. }
        ),
        LogFilter::Diplomacy => matches!(
            event.kind,
            WorldEventKind::EraShift { .. }
                | WorldEventKind::MacroShock { .. }
                | WorldEventKind::Social { .. }
        ),
    };
    if !passes {
        return false;
    }
    if let Some(pin) = pinned {
        return event_involves(event, pin);
    }
    true
}

fn event_involves(
    event: &crate::simulation::WorldEvent,
    nation: crate::simulation::Nation,
) -> bool {
    match &event.kind {
        WorldEventKind::Trade { actor, .. } => actor.nation == nation,
        WorldEventKind::Social { convener, .. } => convener.nation == nation,
        WorldEventKind::MacroShock { .. } => false,
        WorldEventKind::Warfare { winner, loser, .. } => *winner == nation || *loser == nation,
        WorldEventKind::EraShift { nation: n, .. } => *n == nation,
        WorldEventKind::ScienceProgress { nation: n, .. } => *n == nation,
        WorldEventKind::ScienceVictory { winner, .. } => *winner == nation,
        WorldEventKind::InterstellarProgress { leader, .. } => *leader == nation,
        WorldEventKind::InterstellarVictory { winner, .. } => *winner == nation,
    }
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
            format!("LOG: {}", control.log_filter.label()),
            Style::default().fg(MODERN_THEME.accent_a).bold(),
        ),
        Span::raw(" | "),
        Span::styled(
            format!(
                "PIN: {}",
                control
                    .pinned_nation
                    .map(|n| n.name().to_string())
                    .unwrap_or_else(|| "None".to_string())
            ),
            Style::default().fg(MODERN_THEME.accent_b),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("War Δ {:+.2}", war_trend),
            Style::default().fg(if war_trend > 0.0 {
                MODERN_THEME.danger
            } else {
                MODERN_THEME.success
            }),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("CO2 Δ {:+.1}", carbon_trend),
            Style::default().fg(MODERN_THEME.warning),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("Pop Δ {}", format_number_commas(pop_trend.max(0.0) as u64)),
            Style::default().fg(MODERN_THEME.success),
        ),
    ])];

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(MODERN_THEME.border));

    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn narrative_ticker(snapshot: &ObserverSnapshot) -> String {
    let mut snippets = Vec::new();
    for event in snapshot.events.iter().rev().take(3) {
        let snippet = match &event.kind {
            WorldEventKind::Trade {
                actor, trade_focus, ..
            } => {
                format!("{} Trade {}", actor.nation.name(), trade_focus)
            }
            WorldEventKind::Social {
                convener,
                gathering_theme,
                ..
            } => {
                format!("{} {}", convener.nation.name(), gathering_theme)
            }
            WorldEventKind::MacroShock {
                stressor,
                projected_impact,
                ..
            } => {
                format!("Shock {} ({})", stressor, projected_impact)
            }
            WorldEventKind::Warfare {
                winner,
                loser,
                nuclear,
                ..
            } => {
                format!(
                    "{} vs {} {}",
                    winner.name(),
                    loser.name(),
                    if *nuclear { "[NUKE]" } else { "" }
                )
            }
            WorldEventKind::EraShift { nation, era, .. } => {
                format!("{} Era {}", nation.name(), era.label())
            }
            WorldEventKind::ScienceProgress { nation, progress } => {
                format!("{} Moon {:.0}%", nation.name(), progress)
            }
            WorldEventKind::ScienceVictory { winner, .. } => {
                format!("{} Science Win", winner.name())
            }
            WorldEventKind::InterstellarProgress { leader, progress } => {
                format!("{} Space {:.0}%", leader.name(), progress)
            }
            WorldEventKind::InterstellarVictory { winner, .. } => {
                format!("{} Galactic Civ", winner.name())
            }
        };
        snippets.push(snippet);
    }

    if snippets.is_empty() {
        "Systems Nominal".to_string()
    } else {
        snippets.join(" · ")
    }
}

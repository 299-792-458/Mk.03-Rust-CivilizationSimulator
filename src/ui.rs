use crate::simulation::events::WorldEventKind;
use crate::simulation::{AxialCoord, Nation, ObserverSnapshot, format_number_commas};
use ratatui::{
    prelude::*,
    style::Stylize,
    text::{Line, Span},
    widgets::{BarChart, Block, Borders, Cell, Paragraph, Row, Sparkline, Table, Wrap},
};
use std::collections::HashMap;
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

const WORLD_ATLAS: &str = r#"
............................................................................................................................
.............%%%%%%%%%%......................%%%%%%%%%%%%%%....................................................%%%%%%%%%%%%..
...........%%%%%%%%%%%%%%...................%%%%%%%%%%%%%%%%%.................................................%%%%%%%%%%%%%..
..........%%%%%%%%%%%%%%%...................%%%%%%%%%%%%%%%%%%................................................%%%%%%%%%%%%%..
..........%%%%%%%%%%%%%%%...................%%%%%%%%%%%%%%%%%%................................................%%%%%%%%%%%%%..
..........%%%%%%%%%%%%%%%...................%%%%%%%%%%%%%%%%%%................................................%%%%%%%%%%%%%..
..........%%%%%%%%%%%%%%%...................%%%%%%%%%%%%%%%%%%................................................%%%%%%%%%%%%%..
..........%%%%%%%%%%%%%%%...................%%%%%%%%%%%%%%%%%%................................................%%%%%%%%%%%%%..
..........%%%%%%%%%%%%%%%...................%%%%%%%%%%%%%%%%%%................................................%%%%%%%%%%%%%..
..........%%%%%%%%%%%%%%%...................%%%%%%%%%%%%%%%%%%................................................%%%%%%%%%%%%%..
..........%%%%%%%%%%%%%%%...................%%%%%%%%%%%%%%%%%%................................................%%%%%%%%%%%%%..
..........%%%%%%%%%%%%%%%...................%%%%%%%%%%%%%%%%%%................................................%%%%%%%%%%%%%..
..........%%%%%%%%%%%%%%%.........%%%%%.....%%%%%%%%%%%%%%%%%%................................................%%%%%%%%%%%%%..
..........%%%%%%%%%%%%%%%........%%%%%%%....%%%%%%%%%%%%%%%%%%..............%%%%%%............................%%%%%%%%%%%%%..
..........%%%%%%%%%%%%%%%.......%%%%%%%%...%%%%%%%%%%%%%%%%%%%............%%%%%%%%...........................%%%%%%%%%%%%%..
..........%%%%%%%%%%%%%%%......%%%%%%%%%...%%%%%%%%%%%%%%%%%%%...........%%%%%%%%%...........................%%%%%%%%%%%%%..
..........%%%%%%%%%%%%%%%.....%%%%%%%%%%...%%%%%%%%%%%%%%%%%%%..........%%%%%%%%%%...........................%%%%%%%%%%%%%..
..........%%%%%%%%%%%%%%%....%%%%%%%%%%%...%%%%%%%%%%%%%%%%%%%.........%%%%%%%%%%%...........................%%%%%%%%%%%%%..
..........%%%%%%%%%%%%%%%...%%%%%%%%%%%%...%%%%%%%%%%%%%%%%%%%........%%%%%%%%%%%%...........................%%%%%%%%%%%%%..
..........%%%%%%%%%%%%%%%...%%%%%%%%%%%%...%%%%%%%%%%%%%%%%%%%........%%%%%%%%%%%%...........................%%%%%%%%%%%%%..
............................................................................................................................
....%%%%%....................................................................................................................
...%%%%%%%%...........................................................................................%%%%%%%%%%%%............
..%%%%%%%%%%%........................................................................................%%%%%%%%%%%%%%%..........
..%%%%%%%%%%%%%.....................................................................................%%%%%%%%%%%%%%%%..........
..%%%%%%%%%%%%%%%...................................................................................%%%%%%%%%%%%%%%%..........
...%%%%%%%%%%%%%%%.....................................................%%%%%%.......................%%%%%%%%%%%%%%%%..........
....%%%%%%%%%%%%%%....................................................%%%%%%%%......................%%%%%%%%%%%%%%%%..........
.....%%%%%%%%%%%%%...................................................%%%%%%%%%......................%%%%%%%%%%%%%%%..........
......%%%%%%%%%%%....................................................%%%%%%%%%.......................%%%%%%%%%%%%%...........
.......%%%%%%%%%.....................................................%%%%%%%%........................%%%%%%%%%%%%............
........%%%%%%.......................................................%%%%%%%.........................%%%%%%%%%%..............
............................................................................................................................
"#;

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

fn render_control_deck(
    frame: &mut Frame,
    area: Rect,
    snapshot: &ObserverSnapshot,
    control: &ControlState,
) {
    let block = Block::default()
        .title("Control Deck — Civilization orchestration")
        .borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(2, 5),
            Constraint::Ratio(2, 5),
            Constraint::Ratio(1, 5),
        ])
        .split(inner);

    let active_preset = control
        .preset_status
        .iter()
        .find(|p| p.active)
        .map(|p| format!("{} [{}]", p.label, p.key))
        .unwrap_or_else(|| "Custom flow".to_string());

    let status_lines = vec![
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
            Span::raw(" · Tick "),
            Span::styled(
                format!("{} ms", control.tick_duration.as_millis()),
                Style::default().fg(Color::White),
            ),
            Span::raw(" · "),
            Span::styled(
                format!("{:.0} y/tick", control.years_per_tick),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw(" · Preset "),
            Span::styled(active_preset, Style::default().fg(Color::Magenta)),
        ]),
        Line::from(vec![
            Span::styled("Map ", Style::default().fg(Color::White)),
            Span::styled(
                control.map_overlay.label(),
                Style::default().fg(Color::LightCyan).bold(),
            ),
            Span::raw(" · Overlay [ ] cycle"),
            Span::raw(" · Climate "),
            Span::styled(
                format!(
                    "{:.0}ppm | Risk {:.1}% | Biodiversity {:.1}",
                    snapshot.science_victory.carbon_ppm,
                    snapshot.science_victory.climate_risk,
                    snapshot.science_victory.biodiversity
                ),
                Style::default().fg(Color::Gray),
            ),
        ]),
        Line::from(vec![
            Span::styled("Log ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("Filter {} ", control.log_filter.label())),
            Span::styled(
                if control.log_pin_selected {
                    "PIN ON"
                } else {
                    "PIN OFF"
                },
                Style::default().fg(if control.log_pin_selected {
                    Color::LightCyan
                } else {
                    Color::Gray
                }),
            ),
            Span::raw(" · Pin "),
            Span::styled(
                control
                    .pinned_nation
                    .map(|n| n.name().to_string())
                    .unwrap_or_else(|| "None".to_string()),
                Style::default().fg(Color::Magenta),
            ),
            Span::raw(" · Focus "),
            Span::styled(
                control
                    .selected_owner
                    .map(|n| n.name().to_string())
                    .unwrap_or_else(|| "None".to_string()),
                Style::default().fg(Color::LightGreen),
            ),
        ]),
        Line::from(format!(
            "Stage {} | Extinction {} | Hex {} | Entities {}",
            snapshot.geologic_stage,
            snapshot.extinction_events,
            snapshot.grid.hexes.len(),
            snapshot.entities.len()
        )),
        Line::from(vec![
            Span::styled("Hotkeys:", Style::default().fg(Color::Yellow)),
            Span::raw(" Space/P PAUSED/RESUME  "),
            Span::styled("+-", Style::default().fg(Color::Green)),
            Span::raw(" Tick speed  "),
            Span::styled("< >", Style::default().fg(Color::Cyan)),
            Span::raw(" Timescale  "),
            Span::styled("1~4", Style::default().fg(Color::LightMagenta)),
            Span::raw(" Preset  "),
            Span::styled("R", Style::default().fg(Color::LightYellow)),
            Span::raw(" Reset  "),
            Span::styled("Q", Style::default().fg(Color::Red)),
            Span::raw(" Quit  "),
            Span::styled("[ ]", Style::default().fg(Color::LightCyan)),
            Span::raw(" Map Preset  "),
            Span::styled("F", Style::default().fg(Color::Cyan)),
            Span::raw(" Log filter  "),
            Span::styled("G", Style::default().fg(Color::Magenta)),
            Span::raw(" Pin on/off  "),
            Span::styled("C", Style::default().fg(Color::LightCyan)),
            Span::raw(" Pin selection  "),
            Span::styled("V", Style::default().fg(Color::LightGreen)),
            Span::raw(" Focus toggle"),
        ]),
        Line::from("Mouse: top-left [-][+][R] buttons usable"),
    ];
    let status_paragraph = Paragraph::new(status_lines).wrap(Wrap { trim: true });
    frame.render_widget(status_paragraph, columns[0]);

    let preset_rows: Vec<Row> = control
        .preset_status
        .iter()
        .map(|preset| {
            let marker = if preset.active { "▶" } else { "·" };
            Row::new(vec![
                Cell::from(format!("{marker} {}", preset.label)),
                Cell::from(format!("{} | {}", preset.key, preset.intent)),
                Cell::from(format!("{} ms", preset.tick_ms)),
                Cell::from(format!("{:.0}y/t", preset.years_per_tick)),
            ])
            .style(if preset.active {
                Style::default().fg(Color::LightGreen).bold()
            } else {
                Style::default().fg(Color::White)
            })
        })
        .collect();

    let preset_table = Table::new(
        preset_rows,
        [
            Constraint::Length(14),
            Constraint::Min(18),
            Constraint::Length(9),
            Constraint::Length(12),
        ],
    )
    .header(
        Row::new(vec!["Preset", "Role", "Tick", "y/tick"])
            .style(Style::default().fg(Color::White).bold()),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Speed Presets"),
    );
    frame.render_widget(preset_table, columns[1]);

    let side = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(0)])
        .split(columns[2]);

    let pulse = Paragraph::new(world_pulse_lines(snapshot))
        .block(Block::default().borders(Borders::ALL).title("World Pulse"))
        .wrap(Wrap { trim: true });
    frame.render_widget(pulse, side[0]);

    let legend_lines = vec![
        Line::from(vec![
            Span::styled("Map", Style::default().fg(Color::White).bold()),
            Span::raw(" ◆ Leader | █ Territory | ✸ Front | ◎ Nuke "),
        ]),
        Line::from(vec![
            Span::raw("≈ Sea | ░ Ice | Mode "),
            Span::styled(
                control.map_overlay.label(),
                Style::default().fg(Color::Cyan).bold(),
            ),
            Span::raw("  [ [ ] ] toggle"),
        ]),
        Line::from(format!(
            "Sea {:.0}% | Ice {:.0}% | Fronts {}",
            snapshot.overlay.sea_level * 100.0,
            snapshot.overlay.ice_line * 100.0,
            snapshot.combat_hexes.len()
        )),
        Line::from(match control.selected_owner {
            Some(nation) => format!(
                "Selected hex: {} | Front {} | Nuke {}",
                nation.name(),
                if control
                    .selected_hex
                    .map(|c| snapshot.combat_hexes.contains(&c))
                    .unwrap_or(false)
                {
                    "Yes"
                } else {
                    "None"
                },
                if control
                    .selected_hex
                    .map(|c| snapshot.nuclear_hexes.contains(&c))
                    .unwrap_or(false)
                {
                    "Yes"
                } else {
                    "None"
                }
            ),
            None => "No hex selected".to_string(),
        }),
        Line::from(vec![
            Span::styled("Diplomacy ", Style::default().fg(Color::Magenta).bold()),
            Span::raw(format!(
                "Alliances {} · Sanctions {} · Trust {:.0}",
                snapshot.diplomacy.alliances.len(),
                snapshot.diplomacy.sanctions.len(),
                snapshot
                    .diplomacy
                    .trust
                    .iter()
                    .map(|(_, v)| *v)
                    .sum::<f32>()
                    / snapshot.diplomacy.trust.len().max(1) as f32
            )),
            Span::raw(if control.focus_mode {
                " · Focus mode"
            } else {
                ""
            }),
        ]),
    ];
    let legend = Paragraph::new(legend_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Legend / Overlay"),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(legend, side[1]);
}

fn world_pulse_lines(snapshot: &ObserverSnapshot) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::styled(
            format!(
                "Geology {} · Extinction {}",
                snapshot.geologic_stage, snapshot.extinction_events
            ),
            Style::default().fg(Color::LightBlue),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("Space {:.2}00M yrs", snapshot.cosmic_age_years / 100_000_000.0),
            Style::default().fg(Color::Cyan),
        ),
    ]));

    lines.push(Line::from(vec![
        Span::styled(
            format!("War Fatigue {:.1}", snapshot.overlay.war_fatigue),
            Style::default().fg(Color::LightRed),
        ),
        Span::raw(" | "),
        Span::styled(
            format!(
                "Richness {:.0}%",
                snapshot.overlay.resource_richness * 100.0
            ),
            Style::default().fg(Color::Green),
        ),
        Span::raw(" | "),
        Span::styled(
            format!(
                "Ice Line {:.0}% / Sea Level {:.0}%",
                snapshot.overlay.ice_line * 100.0,
                snapshot.overlay.sea_level * 100.0
            ),
            Style::default().fg(Color::LightCyan),
        ),
    ]));

    lines.push(Line::from(vec![
        Span::styled(
            format!("Science {:.1}%", snapshot.science_victory.leader_progress),
            Style::default().fg(Color::LightGreen),
        ),
        Span::raw(" | "),
        Span::styled(
            format!(
                "Space {:.1}%",
                snapshot.science_victory.interstellar_progress
            ),
            Style::default().fg(Color::Magenta),
        ),
        Span::raw(" | Events "),
        Span::styled(
            snapshot.events.len().to_string(),
            Style::default().fg(Color::Yellow),
        ),
    ]));

    lines.push(Line::from(vec![
        Span::styled("Chronicle ", Style::default().fg(Color::LightYellow)),
        Span::raw(narrative_ticker(snapshot)),
    ]));

    lines
}

fn render_world_state_panel(
    frame: &mut Frame,
    area: Rect,
    snapshot: &ObserverSnapshot,
    control: &ControlState,
) {
    let outer_block = Block::default().title("World State").borders(Borders::ALL);
    frame.render_widget(outer_block, area);

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
        } // Avoid panic if more than 3 nations

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
            // Diplomacy/ideology quick stats
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
            nation_lines.push(Line::from(Span::styled(
                format!(
                    "  Trust {:.0} | Fear {:.0} | Ideology {:.0} / Cohesion {:.0} / Volatility {:.0}",
                    trust, fear, leaning, cohesion, volatility
                ),
                Style::default().fg(Color::Gray),
            )));

            nation_lines.push(Line::from(Span::styled(
                format!(
                    "  Era: {} | Weapon: {}",
                    metrics.era.label(),
                    metrics.weapon_tier.label()
                ),
                Style::default().fg(Color::Cyan),
            )));
            nation_lines.push(Line::from(Span::styled(
                format!(
                    "  Population: {}",
                    format_number_commas(metrics.population)
                ),
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
                nation_lines.push(Line::from(Span::styled("  Economy", Style::default())));
                nation_lines.push(create_bar(metrics.economy, 100.0, 10, nation_color));
                nation_lines.push(Line::from(Span::styled(
                    "  Science (Science)",
                    Style::default(),
                )));
                nation_lines.push(create_bar(metrics.science, 100.0, 10, nation_color));
                nation_lines.push(Line::from(Span::styled("  Culture", Style::default())));
                nation_lines.push(create_bar(metrics.culture, 100.0, 10, nation_color));
                nation_lines.push(Line::from(Span::styled(
                    "  Diplomacy (Diplomacy)",
                    Style::default(),
                )));
                nation_lines.push(create_bar(metrics.diplomacy, 100.0, 10, nation_color));
                nation_lines.push(Line::from(Span::styled("  Religion", Style::default())));
                nation_lines.push(create_bar(metrics.religion, 100.0, 10, nation_color));
                nation_lines.push(Line::from(Span::styled("  Military", Style::default())));
                nation_lines.push(create_bar(metrics.military, 100.0, 10, nation_color));
                nation_lines.push(Line::from(Span::styled("  Territory", Style::default())));
                nation_lines.push(create_bar(metrics.territory, 100.0, 10, nation_color));
            }
            let nation_paragraph = Paragraph::new(nation_lines).scroll((0, 0));
            frame.render_widget(nation_paragraph, nations_layout[i]);
        }
    }

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
    frame.render_widget(speed_paragraph, panel_layout[6]);
}

fn render_science_progress_panel(frame: &mut Frame, area: Rect, snapshot: &ObserverSnapshot) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(area);

    let leader_name = snapshot
        .science_victory
        .leader
        .map(|n| n.name().to_string())
        .unwrap_or_else(|| "TBD".to_string());
    let leader_progress = snapshot
        .science_victory
        .leader_progress
        .min(snapshot.science_victory.goal);

    let text = Paragraph::new(vec![
        Line::from("Moonshot progress (1 tick = 1 gen)"),
        Line::from(format!(
            "Leader: {} | {:.1}% / {:.0}% | Cosmic {:.0}y/tick",
            leader_name,
            leader_progress,
            snapshot.science_victory.goal,
            snapshot.timescale_years_per_tick
        )),
    ]);
    frame.render_widget(text, layout[0]);

    let mut data: Vec<u64> = snapshot
        .science_victory
        .history
        .iter()
        .map(|v| v.min(snapshot.science_victory.goal).round() as u64)
        .collect();
    if data.is_empty() {
        data.push(0);
    }
    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Moonshot Progress (Leader)"),
        )
        .data(&data)
        .max(snapshot.science_victory.goal as u64)
        .style(Style::default().fg(if snapshot.science_victory.finished {
            Color::LightGreen
        } else {
            Color::Cyan
        }));
    frame.render_widget(sparkline, layout[1]);
}

fn render_evolutionary_charts(frame: &mut Frame, area: Rect, snapshot: &ObserverSnapshot) {
    let block = Block::default()
        .title("Evolutionary Markets — 5B-year portfolio")
        .borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lanes = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(3),
        ])
        .split(inner);

    let legend = Paragraph::new(vec![
        Line::from(Span::styled(
            "Parallel visualization of emergent timescales and macro shocks",
            Style::default().fg(Color::LightCyan),
        )),
        Line::from("Per-tick momentum, event density, mood vector"),
    ]);
    frame.render_widget(legend, lanes[0]);

    let mut moon_series: Vec<u64> = snapshot
        .science_victory
        .history
        .iter()
        .map(|v| v.min(snapshot.science_victory.goal).round() as u64)
        .collect();
    ensure_nonempty(&mut moon_series);
    let moon = Sparkline::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Moonshot Momentum — leader orbit"),
        )
        .data(&moon_series)
        .max(snapshot.science_victory.goal.max(1.0) as u64)
        .style(
            Style::default()
                .fg(Color::LightGreen)
                .bg(Color::Rgb(20, 20, 30)),
        );
    frame.render_widget(moon, lanes[1]);

    let climate_lane = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(lanes[2]);

    let carbon_series = series_from_history(&snapshot.overlay.carbon_history, 1.0);
    let biodiversity_series = series_from_history(&snapshot.overlay.biodiversity_history, 1.0);
    let risk_series = series_from_history(&snapshot.overlay.climate_risk_history, 1.0);
    let war_series = series_from_history(&snapshot.overlay.war_fatigue_history, 1.0);
    let richness_series = series_from_history(&snapshot.overlay.richness_history, 100.0);

    let planet_left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(2)])
        .split(climate_lane[0]);
    let planet_right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(2)])
        .split(climate_lane[1]);

    let carbon = Sparkline::default()
        .block(Block::default().borders(Borders::ALL).title("Carbon ppm"))
        .data(&carbon_series)
        .max(carbon_series.iter().cloned().max().unwrap_or(1))
        .style(Style::default().fg(Color::Red));
    frame.render_widget(carbon, planet_left[0]);

    let biodiversity = Sparkline::default()
        .block(Block::default().borders(Borders::ALL).title("Biodiversity"))
        .data(&biodiversity_series)
        .max(biodiversity_series.iter().cloned().max().unwrap_or(1))
        .style(Style::default().fg(Color::Green));
    frame.render_widget(biodiversity, planet_left[1]);

    let risk = Sparkline::default()
        .block(Block::default().borders(Borders::ALL).title("Climate Risk"))
        .data(&risk_series)
        .max(risk_series.iter().cloned().max().unwrap_or(1))
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(risk, planet_right[0]);

    let warfatigue = Sparkline::default()
        .block(Block::default().borders(Borders::ALL).title("War Fatigue"))
        .data(&war_series)
        .max(war_series.iter().cloned().max().unwrap_or(1))
        .style(Style::default().fg(Color::LightRed));
    frame.render_widget(warfatigue, planet_right[1]);

    let richness = Sparkline::default()
        .block(Block::default().borders(Borders::ALL).title("Richness %"))
        .data(&richness_series)
        .max(richness_series.iter().cloned().max().unwrap_or(1))
        .style(Style::default().fg(Color::LightBlue));
    frame.render_widget(richness, lanes[3]);

    let event_density = build_event_density_series(snapshot, (inner.width as usize).max(24));
    let density = Sparkline::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Event Density — War·Trade·Shock Count"),
        )
        .data(&event_density)
        .max(event_density.iter().cloned().max().unwrap_or(1))
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(density, lanes[4]);

    let sentiment_curve =
        build_sentiment_series(snapshot, (inner.width as usize / 2).max(16), snapshot.tick);
    let sentiment = Sparkline::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Pulse / Sentiment — Evolutionary Mood Drift"),
        )
        .data(&sentiment_curve)
        .max(sentiment_curve.iter().cloned().max().unwrap_or(1))
        .style(Style::default().fg(Color::Magenta));
    frame.render_widget(sentiment, lanes[5]);

    let pop_series: Vec<u64> = snapshot
        .science_victory
        .population_history
        .iter()
        .rev()
        .take(120)
        .cloned()
        .collect::<Vec<u64>>()
        .into_iter()
        .rev()
        .collect();
    let mut pop_series = pop_series;
    ensure_nonempty(&mut pop_series);
    let pop = Sparkline::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Civilization Pop (mega)"),
        )
        .data(&pop_series)
        .max(pop_series.iter().cloned().max().unwrap_or(1))
        .style(Style::default().fg(Color::White));
    frame.render_widget(pop, lanes[6]);
}

fn render_event_leaderboard(frame: &mut Frame, area: Rect, snapshot: &ObserverSnapshot) {
    let mut counts: HashMap<&'static str, u64> = HashMap::new();
    let mut sentiment_score: HashMap<&'static str, i64> = HashMap::new();

    for event in snapshot.events.iter().rev().take(120) {
        let cat = event.category();
        *counts.entry(cat).or_default() += 1;
        let delta = match event.sentiment() {
            crate::simulation::Sentiment::Positive => 1,
            crate::simulation::Sentiment::Negative => -2,
            crate::simulation::Sentiment::Neutral => 0,
        };
        *sentiment_score.entry(cat).or_default() += delta;
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

    let rows: Vec<Row> = categories
        .iter()
        .map(|cat| {
            let count = counts.get(cat).cloned().unwrap_or(0);
            let score = sentiment_score.get(cat).cloned().unwrap_or(0);
            let bar = heat_bar(count, max_count, 12);
            Row::new(vec![
                Cell::from(*cat),
                Cell::from(count.to_string()),
                Cell::from(score.to_string()),
                Cell::from(bar),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Min(10),
        ],
    )
    .header(
        Row::new(vec!["Type", "Count", "Sentiment", "Heat"])
            .style(Style::default().fg(Color::White).bold()),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Event Leaderboard"),
    );

    frame.render_widget(table, area);
}

fn render_glory_tiles(frame: &mut Frame, area: Rect, snapshot: &ObserverSnapshot) {
    let block = Block::default().borders(Borders::ALL).title("Hall of Fame");
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

fn render_war_theater_panel(frame: &mut Frame, area: Rect, snapshot: &ObserverSnapshot) {
    let block = Block::default().borders(Borders::ALL).title("War Theater");
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

    // Top armies
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

struct MapWidget<'a> {
    snapshot: &'a ObserverSnapshot,
    overlay: MapOverlay,
    selected_hex: Option<AxialCoord>,
    focus: Option<Nation>,
}

impl<'a> Widget for MapWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let atlas: Vec<&str> = WORLD_ATLAS.trim_matches('\n').lines().collect();
        if atlas.is_empty() {
            return;
        }
        let atlas_height = atlas.len() as f32;
        let atlas_width = atlas[0].len() as f32;
        let tick = self.snapshot.tick;
        let season = self.snapshot.season.as_str();
        let leader = self.snapshot.science_victory.leader;

        let season_tint = match season {
            "Peak Flame" => Color::Rgb(255, 120, 80),
            "Ember Fall" => Color::DarkGray,
            _ => Color::LightGreen,
        };

        for y in 0..area.height {
            for x in 0..area.width {
                let atlas_x = ((x as f32 / area.width as f32) * atlas_width)
                    .floor()
                    .min(atlas_width - 1.0) as usize;
                let atlas_y = ((y as f32 / area.height as f32) * atlas_height)
                    .floor()
                    .min(atlas_height - 1.0) as usize;
                let ch = atlas[atlas_y].as_bytes()[atlas_x] as char;
                let is_land = ch == '%' || ch == '#' || ch == '█';

                let mut base_char = if is_land { "▓" } else { "·" };
                let mut color = if is_land {
                    season_tint
                } else {
                    Color::Rgb(70, 110, 160)
                };

                // Sea/ice overlays respond to climate view too.
                let norm_y = y as f32 / area.height as f32;
                let sea_level = self.snapshot.overlay.sea_level;
                let ice_line = self.snapshot.overlay.ice_line;

                match self.overlay {
                    MapOverlay::Ownership => {
                        if !is_land && norm_y > sea_level {
                            base_char = "≈";
                            color = Color::Rgb(50, 90, 140);
                        }
                        if norm_y < ice_line {
                            base_char = "░";
                            color = Color::White;
                        }
                    }
                    MapOverlay::Climate => {
                        // Heatmap from carbon/climate risk
                        let risk =
                            (self.snapshot.science_victory.climate_risk / 140.0).clamp(0.0, 1.0);
                        let heat = (risk * 255.0).round().clamp(0.0, 255.0) as u8;
                        let green = (180.0 - risk * 120.0).max(20.0).min(255.0) as u8;
                        let cool = (150.0 - risk * 120.0).max(30.0).min(255.0) as u8;
                        color = if is_land {
                            Color::Rgb(heat, green, cool)
                        } else {
                            Color::Rgb(40, 100, 180)
                        };
                        if norm_y > sea_level {
                            base_char = "≈";
                            color = Color::Rgb(30, 80, 140);
                        }
                        if norm_y < ice_line {
                            base_char = "░";
                            color = Color::White;
                        }
                    }
                    MapOverlay::Conflict => {
                        // War fatigue tint + flashing fronts
                        let fatigue_norm =
                            (self.snapshot.overlay.war_fatigue / 100.0).clamp(0.0, 1.2);
                        let red = (120.0 + fatigue_norm * 100.0).min(255.0) as u8;
                        let green = (120.0 - fatigue_norm * 60.0).max(20.0) as u8;
                        color = Color::Rgb(red, green, 60);
                        if !is_land {
                            base_char = "·";
                            color = Color::Rgb(60, 90, 120);
                        }
                    }
                }

                // Dynamic accents
                if is_land && (tick + x as u64 + y as u64) % 13 == 0 {
                    color = Color::LightYellow;
                }
                if !is_land && tick % 5 == 0 {
                    color = Color::Rgb(90, 140, 200);
                }

                buf.set_string(
                    area.x + x,
                    area.y + y,
                    base_char,
                    Style::default().fg(color),
                );
            }
        }

        // Overlay hot zones
        let center_x = area.x + area.width / 2;
        let center_y = area.y + area.height / 2;
        let grid = &self.snapshot.grid;
        // Territories overlay
        for (&coord, hex) in &grid.hexes {
            let screen_x = center_x as i32 + coord.q * 2 + coord.r;
            let screen_y = center_y as i32 + coord.r;
            if screen_x < area.x as i32
                || screen_x >= (area.x + area.width) as i32
                || screen_y < area.y as i32
                || screen_y >= (area.y + area.height) as i32
            {
                continue;
            }
            let mut style = Style::default().fg(hex.owner.color());
            if Some(hex.owner) == leader {
                style = style.bold();
            }
            if Some(hex.owner) == self.focus {
                style = style.bg(Color::Rgb(30, 30, 60)).fg(Color::White).bold();
            }
            let glyph = if self.selected_hex == Some(coord) {
                "◎"
            } else if Some(hex.owner) == leader {
                "◆"
            } else {
                "█"
            };
            buf.set_string(screen_x as u16, screen_y as u16, glyph, style);
        }

        // Conflict overlays
        for (&coord, _) in &grid.hexes {
            let screen_x = center_x as i32 + coord.q * 2 + coord.r;
            let screen_y = center_y as i32 + coord.r;
            if screen_x < area.x as i32
                || screen_x >= (area.x + area.width) as i32
                || screen_y < area.y as i32
                || screen_y >= (area.y + area.height) as i32
            {
                continue;
            }
            if self.snapshot.nuclear_hexes.contains(&coord) {
                let glyph = if self.selected_hex == Some(coord) {
                    "◎"
                } else {
                    "◎"
                };
                buf.set_string(
                    screen_x as u16,
                    screen_y as u16,
                    glyph,
                    Style::default().fg(Color::Yellow).bg(Color::Red),
                );
            } else if self.snapshot.combat_hexes.contains(&coord) {
                let style = if tick % 2 == 0 {
                    Style::default().fg(Color::White)
                } else {
                    Style::default().fg(Color::Red)
                };
                let glyph = if self.selected_hex == Some(coord) {
                    "◎"
                } else {
                    "✸"
                };
                buf.set_string(screen_x as u16, screen_y as u16, glyph, style);
            }
        }
    }
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

fn build_event_density_series(snapshot: &ObserverSnapshot, buckets: usize) -> Vec<u64> {
    if buckets == 0 {
        return vec![0];
    }
    let bucket_size = ((snapshot.tick + 1) as f32 / buckets as f32)
        .ceil()
        .max(1.0) as u64;
    let mut series = vec![0u64; buckets];

    for event in &snapshot.events {
        let index = (event.tick / bucket_size).min((buckets - 1) as u64) as usize;
        series[index] += 1;
    }

    ensure_nonempty(&mut series);
    series
}

fn build_sentiment_series(snapshot: &ObserverSnapshot, buckets: usize, last_tick: u64) -> Vec<u64> {
    if buckets == 0 {
        return vec![0];
    }
    let bucket_size = ((last_tick + 1) as f32 / buckets as f32).ceil().max(1.0) as u64;
    let mut series = vec![0i64; buckets];

    for event in &snapshot.events {
        let index = (event.tick / bucket_size).min((buckets - 1) as u64) as usize;
        let delta = match event.kind {
            WorldEventKind::MacroShock { .. } | WorldEventKind::Warfare { .. } => -2,
            WorldEventKind::ScienceVictory { .. } | WorldEventKind::InterstellarVictory { .. } => 3,
            WorldEventKind::ScienceProgress { .. }
            | WorldEventKind::InterstellarProgress { .. }
            | WorldEventKind::EraShift { .. } => 2,
            WorldEventKind::Trade { .. } | WorldEventKind::Social { .. } => 1,
        };
        series[index] += delta;
    }

    let min_value = *series.iter().min().unwrap_or(&0);
    let offset = if min_value < 0 { -min_value as u64 } else { 0 };
    let mut shifted: Vec<u64> = series
        .into_iter()
        .map(|v| (v + offset as i64) as u64)
        .collect();
    ensure_nonempty(&mut shifted);
    shifted
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

fn ensure_nonempty(series: &mut Vec<u64>) {
    if series.is_empty() {
        series.push(0);
    }
    if series.iter().all(|v| *v == 0) {
        series[0] = 1;
    }
}

fn series_from_history(history: &[f32], scale: f32) -> Vec<u64> {
    let mut data: Vec<f32> = history
        .iter()
        .rev()
        .take(120)
        .cloned()
        .collect::<Vec<f32>>();
    data.reverse();
    let mut mapped: Vec<u64> = data
        .iter()
        .map(|v| (v * scale).abs().round() as u64)
        .collect();
    ensure_nonempty(&mut mapped);
    mapped
}

fn heat_bar(value: u64, max: u64, width: usize) -> String {
    let max = max.max(1);
    let filled = ((value as f32 / max as f32) * width as f32).round() as usize;
    let mut bar = "█".repeat(filled.min(width));
    bar.push_str(&"░".repeat(width.saturating_sub(filled)));
    bar
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

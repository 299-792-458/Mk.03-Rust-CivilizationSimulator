use ratatui::{
    prelude::*,
    style::Stylize,
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
};

use crate::simulation::ObserverSnapshot;
use super::ControlState;

/// Renders the control deck with status, presets, and legend/meta info.
pub fn render_control_deck(
    frame: &mut Frame,
    area: Rect,
    snapshot: &ObserverSnapshot,
    control: &ControlState,
) {
    let block = Block::default()
        .title("COMMAND BRIDGE — tempo / overlay / filters")
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

pub fn world_pulse_lines(snapshot: &ObserverSnapshot) -> Vec<Line<'static>> {
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
        Span::raw(crate::ui::narrative_ticker(snapshot)),
    ]));

    lines
}

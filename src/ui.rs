use crate::simulation::events::WorldEventKind;
use crate::simulation::{ObserverSnapshot, format_number_commas};
use ratatui::{
    prelude::*,
    style::Stylize,
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Sparkline, Table, Wrap},
};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ControlState {
    pub paused: bool,
    pub tick_duration: Duration,
    pub years_per_tick: f64,
    pub preset_status: Vec<PresetStatus>,
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

pub fn render(frame: &mut Frame, snapshot: &ObserverSnapshot, control: &ControlState) {
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
            Span::raw("대기 흐름: "),
            Span::styled(
                &snapshot.season_effect.label,
                Style::default().fg(Color::Yellow).bold(),
            ),
            Span::raw("  "),
            Span::styled(
                format!(
                    "온도 {:+.1}  사기 {:+.1}%  수확 {:+.1}%  위험 {:+.1}%",
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
            "우주 문명 달성 — 시뮬레이션 안정화",
            Style::default().fg(Color::LightGreen).bold(),
        )));
    } else if snapshot.science_victory.interstellar_mode {
        header_lines.push(Line::from(Span::styled(
            "성간 확장 단계 진행 중",
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
                "Cosmic {:.2}억년",
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
            format!("Scale {:.0}년/틱", snapshot.timescale_years_per_tick),
            Style::default().fg(Color::Gray),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("Sea {:.0}%", snapshot.overlay.sea_level * 100.0),
            Style::default().fg(Color::Blue),
        ),
    ]));

    let header_paragraph = Paragraph::new(header_lines).block(Block::new().borders(Borders::TOP));
    frame.render_widget(header_paragraph, main_layout[0]);
    render_control_deck(frame, main_layout[1], snapshot, control);

    // Create a vertical layout for the main content area
    let content_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main_layout[2]);

    // Top layout for world state and map
    let top_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(content_layout[0]);

    // World State Panel
    render_world_state_panel(frame, top_layout[0], snapshot, control);

    // Map Widget
    let map_widget = MapWidget { snapshot };
    frame.render_widget(map_widget, top_layout[1]);

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
                        .map(|c| format!("피해 {}명", format_number_commas(c)))
                        .unwrap_or_else(|| "피해 보고 없음".to_string());
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
                        "+{:.2} territory | 사상자 {}명{}",
                        territory_change,
                        format_number_commas(*casualties),
                        if *nuclear { " | 핵" } else { "" }
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
                    "달 탐사 단계".to_string(),
                    format!("{progress:.1}% 달성"),
                ),
                WorldEventKind::ScienceVictory { winner, progress } => (
                    winner.name().to_string(),
                    "과학 승리".to_string(),
                    format!("{progress:.1}% 완주"),
                ),
                WorldEventKind::InterstellarProgress { leader, progress } => (
                    leader.name().to_string(),
                    "성간 이주 단계".to_string(),
                    format!("{progress:.1}% 달성"),
                ),
                WorldEventKind::InterstellarVictory { winner, progress } => (
                    winner.name().to_string(),
                    "우주 문명 승리".to_string(),
                    format!("{progress:.1}% 완주"),
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

            Row::new(cells).height(1).style(style)
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
            .title("Event Log — 전쟁 피로/자원 풍부도는 좌측 패널 확인")
            .borders(Borders::ALL),
    );

    let event_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(0)])
        .split(content_layout[1]);

    render_event_leaderboard(frame, event_layout[0], snapshot);
    frame.render_widget(table, event_layout[1]);
}

fn render_control_deck(
    frame: &mut Frame,
    area: Rect,
    snapshot: &ObserverSnapshot,
    control: &ControlState,
) {
    let block = Block::default()
        .title("Control Deck — 문명 진화 오케스트레이션")
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
                format!("{:.0} 년/틱", control.years_per_tick),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw(" · Preset "),
            Span::styled(active_preset, Style::default().fg(Color::Magenta)),
        ]),
        Line::from(format!(
            "Stage {} | 멸종 {} | Hex {} | Entities {}",
            snapshot.geologic_stage,
            snapshot.extinction_events,
            snapshot.grid.hexes.len(),
            snapshot.entities.len()
        )),
        Line::from(vec![
            Span::styled("핫키:", Style::default().fg(Color::Yellow)),
            Span::raw(" Space/P 정지·재생  "),
            Span::styled("+-", Style::default().fg(Color::Green)),
            Span::raw(" 틱 속도  "),
            Span::styled("< >", Style::default().fg(Color::Cyan)),
            Span::raw(" 시간축  "),
            Span::styled("1~4", Style::default().fg(Color::LightMagenta)),
            Span::raw(" 프리셋  "),
            Span::styled("R", Style::default().fg(Color::LightYellow)),
            Span::raw(" 리셋  "),
            Span::styled("Q", Style::default().fg(Color::Red)),
            Span::raw(" 종료"),
        ]),
        Line::from("마우스: 좌측 상단 [-][+][R] 터미널 버튼도 사용 가능"),
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
        Row::new(vec!["모드", "역할", "틱", "연/틱"])
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
            Span::styled("지도", Style::default().fg(Color::White).bold()),
            Span::raw(" ◆ 선두국 | █ 영토 | ✸ 전선 | ◎ 핵 "),
        ]),
        Line::from("≈ 해수 | ░ 빙선 | 색: 계절 틴트"),
        Line::from(format!(
            "해수면 {:.0}% | 빙선 {:.0}% | 전선 {}",
            snapshot.overlay.sea_level * 100.0,
            snapshot.overlay.ice_line * 100.0,
            snapshot.combat_hexes.len()
        )),
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
                "지질 {} · 멸종 {}",
                snapshot.geologic_stage, snapshot.extinction_events
            ),
            Style::default().fg(Color::LightBlue),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("우주 {:.2}억년", snapshot.cosmic_age_years / 100_000_000.0),
            Style::default().fg(Color::Cyan),
        ),
    ]));

    lines.push(Line::from(vec![
        Span::styled(
            format!("전쟁 피로 {:.1}", snapshot.overlay.war_fatigue),
            Style::default().fg(Color::LightRed),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("풍부도 {:.0}%", snapshot.overlay.resource_richness * 100.0),
            Style::default().fg(Color::Green),
        ),
        Span::raw(" | "),
        Span::styled(
            format!(
                "빙선 {:.0}% / 해수 {:.0}%",
                snapshot.overlay.ice_line * 100.0,
                snapshot.overlay.sea_level * 100.0
            ),
            Style::default().fg(Color::LightCyan),
        ),
    ]));

    lines.push(Line::from(vec![
        Span::styled(
            format!("과학 {:.1}%", snapshot.science_victory.leader_progress),
            Style::default().fg(Color::LightGreen),
        ),
        Span::raw(" | "),
        Span::styled(
            format!(
                "우주 {:.1}%",
                snapshot.science_victory.interstellar_progress
            ),
            Style::default().fg(Color::Magenta),
        ),
        Span::raw(" | 이벤트 "),
        Span::styled(
            snapshot.events.len().to_string(),
            Style::default().fg(Color::Yellow),
        ),
    ]));

    lines.push(Line::from(vec![
        Span::styled("크로니클 ", Style::default().fg(Color::LightYellow)),
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
        .unwrap_or_else(|| "미정".to_string());
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
            "세대(Tick): {} | Entities: {} | 목표: 달 탐사 100%",
            tick, total_entities
        )),
        Line::from(vec![
            Span::styled(
                if control.paused { "정지" } else { "실행" },
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
                format!("{:.0}년/틱", control.years_per_tick),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw(" | "),
            Span::styled(
                format!("모드 {}", active_preset),
                Style::default().fg(Color::Magenta),
            ),
        ]),
        Line::from(format!(
            "Epoch: {} | Season: {}",
            snapshot.epoch, snapshot.season
        )),
        Line::from(format!(
            "대기: {} (ΔT {:+.1}, 사기 {:+.1}%, 수확 {:+.1}%, 위험 {:+.1}%)",
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
            "과학 승리: {} {:.1}% / 100% (격차 {:.1}p) | 성간 {:.1}% / {:.0}%",
            leader_name,
            leader_progress,
            gap,
            snapshot.science_victory.interstellar_progress,
            snapshot.science_victory.interstellar_goal
        )),
        Line::from(format!(
            "세계 포트폴리오: 인구 {} | GDP {:.1} | 이벤트 {}",
            format_number_commas(snapshot.science_victory.total_population),
            snapshot.science_victory.total_economy,
            snapshot.events.len()
        )),
    ];
    let info_paragraph = Paragraph::new(info_lines);
    frame.render_widget(info_paragraph, panel_layout[0]);

    render_science_progress_panel(frame, panel_layout[1], snapshot);
    render_evolutionary_charts(frame, panel_layout[2], snapshot);
    render_glory_tiles(frame, panel_layout[3], snapshot);
    render_war_theater_panel(frame, panel_layout[4], snapshot);

    let mut nations: Vec<_> = snapshot.all_metrics.0.keys().copied().collect();
    nations.sort_by_key(|a| a.name());

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
            let mut nation_lines = vec![];
            nation_lines.push(Line::from(Span::styled(
                nation.name(),
                Style::default().bold().underlined().fg(nation_color),
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
                format!("  인구: {} 명", format_number_commas(metrics.population)),
                Style::default().fg(Color::White),
            )));
            if let Some(civ_state) = snapshot.civ_state.0.get(&nation) {
                nation_lines.push(Line::from(Span::styled(
                    format!(
                        "  도시: {} | 행복도: {:.1} | 생산력: {:.1}",
                        civ_state.cities, civ_state.happiness, civ_state.production
                    ),
                    Style::default().fg(Color::Yellow),
                )));
            }
            let tech_list = if metrics.unlocked_techs.is_empty() {
                "기술 없음".to_string()
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
                nation_lines.push(Line::from(Span::styled(
                    "  경제 (Economy)",
                    Style::default(),
                )));
                nation_lines.push(create_bar(metrics.economy, 100.0, 10, nation_color));
                nation_lines.push(Line::from(Span::styled(
                    "  과학 (Science)",
                    Style::default(),
                )));
                nation_lines.push(create_bar(metrics.science, 100.0, 10, nation_color));
                nation_lines.push(Line::from(Span::styled(
                    "  문화 (Culture)",
                    Style::default(),
                )));
                nation_lines.push(create_bar(metrics.culture, 100.0, 10, nation_color));
                nation_lines.push(Line::from(Span::styled(
                    "  외교 (Diplomacy)",
                    Style::default(),
                )));
                nation_lines.push(create_bar(metrics.diplomacy, 100.0, 10, nation_color));
                nation_lines.push(Line::from(Span::styled(
                    "  종교 (Religion)",
                    Style::default(),
                )));
                nation_lines.push(create_bar(metrics.religion, 100.0, 10, nation_color));
                nation_lines.push(Line::from(Span::styled(
                    "  군사 (Military)",
                    Style::default(),
                )));
                nation_lines.push(create_bar(metrics.military, 100.0, 10, nation_color));
                nation_lines.push(Line::from(Span::styled(
                    "  영토 (Territory)",
                    Style::default(),
                )));
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
        "{} ms/tick | {:.0} 년/틱",
        control.tick_duration.as_millis(),
        control.years_per_tick
    )));
    speed_lines.push(Line::from(vec![
        Span::from("["),
        Span::styled("-", Style::default().fg(Color::Red).bold()),
        Span::from("] ["),
        Span::styled("+", Style::default().fg(Color::Green).bold()),
        Span::from("]  "),
        Span::styled("< >", Style::default().fg(Color::Cyan).bold()),
        Span::from("  "),
        Span::styled("Space/P", Style::default().fg(Color::Yellow).bold()),
        Span::from(" 정지/재생"),
    ]));
    speed_lines.push(Line::from(vec![
        Span::raw("1~4 프리셋  |  "),
        Span::styled("R", Style::default().fg(Color::LightYellow).bold()),
        Span::raw(" 초기화  |  Q 종료"),
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
        .unwrap_or_else(|| "미정".to_string());
    let leader_progress = snapshot
        .science_victory
        .leader_progress
        .min(snapshot.science_victory.goal);

    let text = Paragraph::new(vec![
        Line::from("달 탐사 진행도 (1틱 = 1세대)"),
        Line::from(format!(
            "주도국: {} | {:.1}% / {:.0}% | Cosmic {:.0}년/틱",
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
        .title("Evolutionary Markets — 50억년 포트폴리오 흐름")
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
            "창발적 시간축과 거시 충격을 티커처럼 병렬 시각화",
            Style::default().fg(Color::LightCyan),
        )),
        Line::from("세대별 모멘텀 · 사건 밀도 · 분위기 벡터"),
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
                .title("Moonshot Momentum — 주도국 궤도"),
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
                .title("Event Density — 전쟁·무역·충격 거래량"),
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
                .title("Pulse / Sentiment — 진화적 무드 드리프트"),
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

    let categories = ["전쟁", "무역", "사회", "거시충격", "과학", "우주", "시대"];
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
        Row::new(vec!["종류", "거래량", "감성", "히트"])
            .style(Style::default().fg(Color::White).bold()),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("사건 리더보드"),
    );

    frame.render_widget(table, area);
}

fn render_glory_tiles(frame: &mut Frame, area: Rect, snapshot: &ObserverSnapshot) {
    let block = Block::default().borders(Borders::ALL).title("명예의 전당");
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
            "인구 정점",
            top_pop
                .map(|(n, pop)| format!("{} | {}명", n.name(), format_number_commas(pop)))
                .unwrap_or_else(|| "데이터 없음".to_string()),
            Color::LightCyan,
        ),
        (
            "경제 패권",
            top_gdp
                .map(|(n, m)| format!("{} | 경제 {:.1}", n.name(), m.economy))
                .unwrap_or_else(|| "데이터 없음".to_string()),
            Color::LightGreen,
        ),
        (
            "과학 선두",
            science_leader
                .map(|(n, p)| format!("{} | {:.1}% 달", n.name(), p))
                .unwrap_or_else(|| "미정".to_string()),
            Color::Yellow,
        ),
        (
            "전쟁 승률",
            war_champ
                .map(|(n, w)| format!("{} | {}승", n, w))
                .unwrap_or_else(|| "평화".to_string()),
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
            format!("{} 군사 {:.1} / 영토 {:.1}", nation.name(), mil, terr),
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
        lines.push(Line::from("최근 전투 없음 — 달 착륙 집중"));
    } else {
        lines.push(Line::from(Span::styled(
            "최근 전투",
            Style::default().bold().fg(Color::White),
        )));
        for (win, lose, nuclear, casualties) in recent_battles.drain(..) {
            lines.push(Line::from(format!(
                "{} vs {}{} | 사상자 {}",
                win,
                lose,
                if nuclear { " (핵)" } else { "" },
                format_number_commas(casualties)
            )));
        }
    }

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, inner);
}

struct MapWidget<'a> {
    snapshot: &'a ObserverSnapshot,
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
            "불꽃 절정" => Color::Rgb(255, 120, 80),
            "잿불 내림" => Color::DarkGray,
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

                // Sea level overlay
                let norm_y = y as f32 / area.height as f32;
                if !is_land && norm_y > self.snapshot.overlay.sea_level {
                    base_char = "≈";
                    color = Color::Rgb(50, 90, 140);
                }
                // Ice line overlay
                if norm_y < self.snapshot.overlay.ice_line {
                    base_char = "░";
                    color = Color::White;
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
            let glyph = if Some(hex.owner) == leader {
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
                buf.set_string(
                    screen_x as u16,
                    screen_y as u16,
                    "◎",
                    Style::default().fg(Color::Yellow).bg(Color::Red),
                );
            } else if self.snapshot.combat_hexes.contains(&coord) {
                let style = if tick % 2 == 0 {
                    Style::default().fg(Color::White)
                } else {
                    Style::default().fg(Color::Red)
                };
                buf.set_string(screen_x as u16, screen_y as u16, "✸", style);
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
                format!("{} 무역 — {}", actor.nation.name(), trade_focus)
            }
            WorldEventKind::Social {
                convener,
                gathering_theme,
                ..
            } => {
                format!("{} 모임 — {}", convener.nation.name(), gathering_theme)
            }
            WorldEventKind::MacroShock {
                stressor,
                projected_impact,
                ..
            } => {
                format!("충격 {} → {}", stressor, projected_impact)
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
                    if *nuclear { "핵" } else { "전" },
                    loser.name(),
                    if *nuclear { "!" } else { "" }
                )
            }
            WorldEventKind::EraShift { nation, era, .. } => {
                format!("{} 시대 상승 → {}", nation.name(), era.label())
            }
            WorldEventKind::ScienceProgress { nation, progress } => {
                format!("{} 달 {:.0}%", nation.name(), progress)
            }
            WorldEventKind::ScienceVictory { winner, .. } => {
                format!("{} 과학 승리", winner.name())
            }
            WorldEventKind::InterstellarProgress { leader, progress } => {
                format!("{} 성간 {:.0}%", leader.name(), progress)
            }
            WorldEventKind::InterstellarVictory { winner, .. } => {
                format!("{} 우주 문명", winner.name())
            }
        };
        snippets.push(snippet);
    }

    if snippets.is_empty() {
        "고요한 순간 — 데이터 수집 중".to_string()
    } else {
        snippets.join(" · ")
    }
}

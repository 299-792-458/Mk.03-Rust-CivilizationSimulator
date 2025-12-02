use crate::simulation::events::WorldEventKind;
use crate::simulation::{ObserverSnapshot, format_number_commas};
use ratatui::{
    prelude::*,
    style::Stylize,
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Sparkline, Table},
};
use std::time::Duration;
use std::collections::HashMap;

pub fn render(frame: &mut Frame, snapshot: &ObserverSnapshot, tick_duration: Duration) {
    // Main layout
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(0)])
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

    let header_paragraph = Paragraph::new(header_lines).block(Block::new().borders(Borders::TOP));
    frame.render_widget(header_paragraph, main_layout[0]);

    // Create a vertical layout for the main content area
    let content_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main_layout[1]);

    // Top layout for world state and map
    let top_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(content_layout[0]);

    // World State Panel
    render_world_state_panel(frame, top_layout[0], snapshot, tick_duration);

    // Map Widget
    let map_widget = MapWidget { snapshot };
    frame.render_widget(map_widget, top_layout[1]);

    // Event Log Panel - Using a Table for alignment
    let header_cells = ["Nation", "Tick", "Category", "Actor/Source", "Details", "Impact/Level"]
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

fn render_world_state_panel(
    frame: &mut Frame,
    area: Rect,
    snapshot: &ObserverSnapshot,
    tick_duration: Duration,
) {
    let outer_block = Block::default().title("World State").borders(Borders::ALL);
    frame.render_widget(outer_block, area);

    let panel_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(7),
            Constraint::Length(12),
            Constraint::Length(5),
            Constraint::Min(0),
            Constraint::Length(3),
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

    let info_lines = vec![
        Line::from(format!(
            "세대(Tick): {} | Entities: {} | 목표: 달 탐사 100%",
            tick, total_entities
        )),
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
                format!("Richness {:>4.0}%", snapshot.overlay.resource_richness * 100.0),
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
    ];
    let info_paragraph = Paragraph::new(info_lines);
    frame.render_widget(info_paragraph, panel_layout[0]);

    render_science_progress_panel(frame, panel_layout[1], snapshot);
    render_evolutionary_charts(frame, panel_layout[2], snapshot);
    render_glory_tiles(frame, panel_layout[3], snapshot);

    let mut nations: Vec<_> = snapshot.all_metrics.0.keys().copied().collect();
    nations.sort_by_key(|a| a.name());

    let nations_len = nations.len().max(1) as u32;
    let constraints: Vec<Constraint> = (0..nations.len())
        .map(|_| Constraint::Ratio(1, nations_len))
        .collect();
    let nations_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(panel_layout[4]);

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
        "Tick Speed",
        Style::default().bold(),
    )));
    speed_lines.push(Line::from(format!("{} ms/tick", tick_duration.as_millis())));
    speed_lines.push(Line::from(vec![
        Span::from("["),
        Span::styled("-", Style::default().fg(Color::Red).bold()),
        Span::from("] ["),
        Span::styled("+", Style::default().fg(Color::Green).bold()),
        Span::from("] ["),
        Span::styled("R", Style::default().fg(Color::Yellow).bold()),
        Span::from("]"),
    ]));
    let speed_paragraph = Paragraph::new(speed_lines);
    frame.render_widget(speed_paragraph, panel_layout[5]);
}

fn render_science_progress_panel(
    frame: &mut Frame,
    area: Rect,
    snapshot: &ObserverSnapshot,
) {
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
            "주도국: {} | {:.1}% / {:.0}%",
            leader_name, leader_progress, snapshot.science_victory.goal
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
            Constraint::Length(4),
            Constraint::Length(5),
            Constraint::Length(3),
            Constraint::Length(3),
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
        .constraints([Constraint::Length(2), Constraint::Length(3)])
        .split(climate_lane[0]);
    let planet_right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(3)])
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
    frame.render_widget(sentiment, lanes[3]);
}

struct MapWidget<'a> {
    snapshot: &'a ObserverSnapshot,
}

impl<'a> Widget for MapWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let grid = &self.snapshot.grid;
        let center_x = area.x + area.width / 2;
        let center_y = area.y + area.height / 2;
        let tick = self.snapshot.tick;

        let (season_tint, glow_char) = match self.snapshot.season.as_str() {
            "불꽃 절정" => (Color::LightRed, "░"),
            "잿불 내림" => (Color::DarkGray, "▒"),
            _ => (Color::LightGreen, "·"),
        };

        for (&coord, hex) in &grid.hexes {
            // Convert axial to screen coordinates (flat-top hexes)
            // Use compact spacing but keep stagger for world-outline fidelity
            let hex_width = 1;
            let hex_height = 1;
            let screen_x = center_x as i32 + coord.q * 2 + coord.r;
            let screen_y = center_y as i32 + coord.r;

            let hex_char = if Some(hex.owner) == self.snapshot.science_victory.leader {
                "◆"
            } else {
                "█"
            };

            let mut color = if tick % 5 == 0 {
                season_tint
            } else {
                hex.owner.color()
            };

            // Twinkling effect for combat zones
            if self.snapshot.nuclear_hexes.contains(&coord) {
                color = Color::Yellow;
                let blast_char = "◎";
                if screen_x >= area.x as i32
                    && screen_x + 1 <= (area.x + area.width) as i32
                    && screen_y >= area.y as i32
                    && screen_y + 1 <= (area.y + area.height) as i32
                {
                    buf.set_string(
                        screen_x as u16,
                        screen_y as u16,
                        blast_char,
                        Style::default().fg(color),
                    );
                }
                continue;
            } else if self.snapshot.combat_hexes.contains(&coord) {
                if self.snapshot.tick % 2 == 0 {
                    color = Color::White; // Bright color for twinkling
                }
            }

            // Ambient shimmer based on seasonal mood
            if tick % 7 == 0 {
                let within = screen_x >= area.x as i32
                    && screen_x + hex_width <= (area.x + area.width) as i32
                    && screen_y + 1 >= area.y as i32
                    && screen_y + 1 < (area.y + area.height) as i32;
                if within {
                    buf.set_string(
                        screen_x as u16,
                        (screen_y + 1) as u16,
                        glow_char,
                        Style::default().fg(season_tint),
                    );
                }
            }

            // Climate risk shimmer
            if self.snapshot.science_victory.climate_risk > 60.0 && tick % 3 == 0 {
                if screen_x >= area.x as i32
                    && screen_x + hex_width <= (area.x + area.width) as i32
                    && screen_y >= area.y as i32
                    && screen_y + hex_height <= (area.y + area.height) as i32
                {
                    buf.set_string(
                        screen_x as u16,
                        screen_y as u16,
                        "∙",
                        Style::default().fg(Color::Red),
                    );
                }
            }

            // Draw the hex character
            if screen_x >= area.x as i32
                && screen_x + hex_width <= (area.x + area.width) as i32
                && screen_y >= area.y as i32
                && screen_y + hex_height <= (area.y + area.height) as i32
            {
                buf.set_string(
                    screen_x as u16,
                    screen_y as u16,
                    hex_char,
                    Style::default().fg(color),
                );
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
    let bucket_size = ((snapshot.tick + 1) as f32 / buckets as f32).ceil().max(1.0) as u64;
    let mut series = vec![0u64; buckets];

    for event in &snapshot.events {
        let index = (event.tick / bucket_size).min((buckets - 1) as u64) as usize;
        series[index] += 1;
    }

    ensure_nonempty(&mut series);
    series
}

fn build_sentiment_series(
    snapshot: &ObserverSnapshot,
    buckets: usize,
    last_tick: u64,
) -> Vec<u64> {
    if buckets == 0 {
        return vec![0];
    }
    let bucket_size = ((last_tick + 1) as f32 / buckets as f32).ceil().max(1.0) as u64;
    let mut series = vec![0i64; buckets];

    for event in &snapshot.events {
        let index = (event.tick / bucket_size).min((buckets - 1) as u64) as usize;
        let delta = match event.kind {
            WorldEventKind::MacroShock { .. } | WorldEventKind::Warfare { .. } => -2,
            WorldEventKind::ScienceVictory { .. } | WorldEventKind::InterstellarVictory { .. } => {
                3
            }
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

use ratatui::{
    prelude::*,
    style::Stylize,
    text::Line,
    widgets::{BarChart, Block, Paragraph, Sparkline},
};

use crate::simulation::events::WorldEventKind;
use crate::simulation::ObserverSnapshot;

/// Evolutionary, climate, and sentiment charts.
pub fn render_evolutionary_charts(
    frame: &mut Frame,
    area: Rect,
    snapshot: &ObserverSnapshot,
) {
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

/// Additional graph lane for dramatic indicator swings.
pub fn render_indicator_grid(
    frame: &mut Frame,
    area: Rect,
    snapshot: &ObserverSnapshot,
) {
    let block = Block::default()
        .title("Graph Deck — Civilization Pulseboard")
        .borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(inner);

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(rows[0]);

    let war_series = series_from_history(&snapshot.overlay.war_fatigue_history, 1.0);
    let war_line = Sparkline::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("War Fatigue — front heat"),
        )
        .data(&war_series)
        .max(war_series.iter().cloned().max().unwrap_or(1))
        .style(Style::default().fg(Color::LightRed));
    frame.render_widget(war_line, top[0]);

    let climate_series = series_from_history(&snapshot.overlay.carbon_history, 1.0);
    let climate_line = Sparkline::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Carbon ppm — climate whip"),
        )
        .data(&climate_series)
        .max(climate_series.iter().cloned().max().unwrap_or(1))
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(climate_line, top[1]);

    let mut pop_series: Vec<u64> = snapshot
        .science_victory
        .population_history
        .iter()
        .rev()
        .take(120)
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    ensure_nonempty(&mut pop_series);
    let population_line = Sparkline::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Population — boom/bust waves"),
        )
        .data(&pop_series)
        .max(pop_series.iter().cloned().max().unwrap_or(1))
        .style(Style::default().fg(Color::LightGreen));
    frame.render_widget(population_line, top[2]);

    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(rows[1]);

    let (eco_pairs, eco_max) = build_metric_bar_data(snapshot, |m| m.economy, 5);
    let eco_refs: Vec<(&str, u64)> = eco_pairs
        .iter()
        .map(|(name, value)| (name.as_str(), *value))
        .collect();
    let econ_bars = BarChart::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Economy — surplus race"),
        )
        .data(&eco_refs)
        .max(eco_max)
        .bar_width(6)
        .bar_gap(1)
        .bar_style(Style::default().fg(Color::LightGreen))
        .value_style(Style::default().fg(Color::White).bold());
    frame.render_widget(econ_bars, bottom[0]);

    let (mil_pairs, mil_max) = build_metric_bar_data(snapshot, |m| m.military, 5);
    let mil_refs: Vec<(&str, u64)> = mil_pairs
        .iter()
        .map(|(name, value)| (name.as_str(), *value))
        .collect();
    let war_bars = BarChart::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Military — mobilization"),
        )
        .data(&mil_refs)
        .max(mil_max)
        .bar_width(6)
        .bar_gap(1)
        .bar_style(Style::default().fg(Color::LightRed))
        .value_style(Style::default().fg(Color::White).bold());
    frame.render_widget(war_bars, bottom[1]);

    let (science_pairs, science_max) = build_metric_bar_data(snapshot, |m| m.science, 5);
    let science_refs: Vec<(&str, u64)> = science_pairs
        .iter()
        .map(|(name, value)| (name.as_str(), *value))
        .collect();
    let science_bars = BarChart::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Science — breakthrough tempo"),
        )
        .data(&science_refs)
        .max(science_max)
        .bar_width(6)
        .bar_gap(1)
        .bar_style(Style::default().fg(Color::Cyan))
        .value_style(Style::default().fg(Color::White).bold());
    frame.render_widget(science_bars, bottom[2]);
}

pub fn render_science_progress_panel(
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

pub fn build_event_density_series(snapshot: &ObserverSnapshot, buckets: usize) -> Vec<u64> {
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

pub fn build_sentiment_series(
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

pub fn ensure_nonempty(series: &mut Vec<u64>) {
    if series.is_empty() {
        series.push(0);
    }
    if series.iter().all(|v| *v == 0) {
        series[0] = 1;
    }
}

pub fn series_from_history(history: &[f32], scale: f32) -> Vec<u64> {
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

pub fn build_metric_bar_data<F>(
    snapshot: &ObserverSnapshot,
    selector: F,
    cap: usize,
) -> (Vec<(String, u64)>, u64)
where
    F: Fn(&crate::simulation::resources::NationMetrics) -> f32,
{
    let mut entries: Vec<(String, u64)> = snapshot
        .all_metrics
        .0
        .iter()
        .map(|(nation, metrics)| {
            (
                nation.name().to_string(),
                selector(metrics).max(0.0).round() as u64,
            )
        })
        .collect();
    entries.sort_by(|a, b| b.1.cmp(&a.1));
    entries.truncate(cap);
    let max_value = entries.iter().map(|(_, v)| *v).max().unwrap_or(1).max(1);
    (entries, max_value)
}

pub fn heat_bar(value: u64, max: u64, width: usize) -> String {
    let max = max.max(1);
    let filled = ((value as f32 / max as f32) * width as f32).round() as usize;
    let mut bar = "█".repeat(filled.min(width));
    bar.push_str(&"░".repeat(width.saturating_sub(filled)));
    bar
}

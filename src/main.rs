use std::io::{self, stdout};
use std::panic;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, prelude::*};
use tokio::sync::{Notify, watch};

mod simulation;
mod ui;

use simulation::{ObserverSnapshot, SimulationConfig, SimulationWorld};
use ui::{ControlState, PresetStatus};

#[derive(Clone, Copy)]
struct SpeedPreset {
    key: char,
    label: &'static str,
    intent: &'static str,
    tick_ms: u64,
    years_per_tick: f64,
}

impl SpeedPreset {
    fn duration(&self) -> Duration {
        Duration::from_millis(self.tick_ms)
    }
}

const SPEED_PRESETS: [SpeedPreset; 4] = [
    SpeedPreset {
        key: '1',
        label: "Chronicle",
        intent: "느긋하게 관측",
        tick_ms: 1_600,
        years_per_tick: 250_000.0,
    },
    SpeedPreset {
        key: '2',
        label: "Standard",
        intent: "균형 진행",
        tick_ms: 1_000,
        years_per_tick: 1_000_000.0,
    },
    SpeedPreset {
        key: '3',
        label: "Hyperdrive",
        intent: "급격 발전",
        tick_ms: 400,
        years_per_tick: 5_000_000.0,
    },
    SpeedPreset {
        key: '4',
        label: "Singularity",
        intent: "우주 질주",
        tick_ms: 120,
        years_per_tick: 20_000_000.0,
    },
];

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Simulation Setup
    let config = SimulationConfig {
        tick_duration: Duration::from_secs(1),
        grid_radius: 24,
        years_per_tick: 1_000_000.0,
        ..Default::default()
    };
    let initial_tick_duration = config.tick_duration;
    let initial_years_per_tick = config.years_per_tick;

    let (tick_duration_tx, mut tick_duration_rx) = watch::channel(initial_tick_duration);
    let (timescale_tx, mut timescale_rx) = watch::channel(initial_years_per_tick);
    let (pause_tx, mut pause_rx) = watch::channel(false);
    let mut active_preset: Option<char> = Some('2');

    let observer = Arc::new(RwLock::new(ObserverSnapshot::default()));
    let shutdown_notify = Arc::new(Notify::new());

    let mut simulation = SimulationWorld::with_observer(config, observer.clone());
    let notify_for_simulation = shutdown_notify.clone();
    let simulation_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(*tick_duration_rx.borrow());
        let mut paused = *pause_rx.borrow();
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if !paused {
                        simulation.tick();
                    }
                },
                result = tick_duration_rx.changed() => {
                    if result.is_ok() {
                        let new_duration = *tick_duration_rx.borrow();
                        interval = tokio::time::interval(new_duration);
                    } else {
                        // Channel closed, time to shut down
                        break;
                    }
                },
                result = timescale_rx.changed() => {
                    if result.is_ok() {
                        let new_scale = *timescale_rx.borrow();
                        simulation.set_timescale(new_scale);
                    } else {
                        break;
                    }
                },
                result = pause_rx.changed() => {
                    if result.is_ok() {
                        paused = *pause_rx.borrow();
                    } else {
                        break;
                    }
                },
                _ = notify_for_simulation.notified() => break,
            }
        }
    });
    let ctrlc_notify = shutdown_notify.clone();
    let ctrl_c_task = tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        ctrlc_notify.notify_waiters();
    });

    // TUI Setup
    let mut terminal = init_terminal()?;
    let mut term_guard = TerminalGuard::new();
    panic::set_hook(Box::new(|info| {
        let _ = restore_terminal();
        eprintln!("panic: {info}");
    }));
    let mut app_should_run = true;

    while app_should_run {
        let control_state = ControlState {
            paused: *pause_tx.borrow(),
            tick_duration: *tick_duration_tx.borrow(),
            years_per_tick: *timescale_tx.borrow(),
            preset_status: preset_status(active_preset),
        };

        terminal.draw(|frame| {
            let snapshot = observer.read().expect("Observer lock is poisoned").clone();
            ui::render(frame, &snapshot, &control_state);
        })?;

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') => app_should_run = false,
                    KeyCode::Char(' ') | KeyCode::Char('p') | KeyCode::Char('P') => {
                        let new_state = !*pause_tx.borrow();
                        pause_tx.send(new_state).ok();
                    }
                    KeyCode::Char('1')
                    | KeyCode::Char('2')
                    | KeyCode::Char('3')
                    | KeyCode::Char('4') => {
                        let key_char = if let KeyCode::Char(c) = key.code {
                            c
                        } else {
                            '1'
                        };
                        if let Some(selected) =
                            apply_preset(key_char, &tick_duration_tx, &timescale_tx)
                        {
                            active_preset = Some(selected);
                            pause_tx.send(false).ok();
                        }
                    }
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        let current_duration = *tick_duration_tx.borrow();
                        let new_duration = (current_duration / 2).max(Duration::from_millis(1));
                        active_preset = None;
                        tick_duration_tx.send(new_duration).ok();
                    }
                    KeyCode::Char('-') => {
                        let current_duration = *tick_duration_tx.borrow();
                        let new_duration = current_duration * 2;
                        active_preset = None;
                        tick_duration_tx.send(new_duration).ok();
                    }
                    KeyCode::Char('<') | KeyCode::Char(',') => {
                        let current = *timescale_tx.borrow();
                        let new_scale = (current / 2.0).max(1_000.0);
                        active_preset = None;
                        timescale_tx.send(new_scale).ok();
                    }
                    KeyCode::Char('>') | KeyCode::Char('.') => {
                        let current = *timescale_tx.borrow();
                        let new_scale = (current * 2.0).min(50_000_000_000.0);
                        active_preset = None;
                        timescale_tx.send(new_scale).ok();
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        active_preset = Some('2');
                        tick_duration_tx.send(initial_tick_duration).ok();
                        timescale_tx.send(initial_years_per_tick).ok();
                        pause_tx.send(false).ok();
                    }
                    _ => {}
                },
                Event::Mouse(mouse) => {
                    if mouse.kind == event::MouseEventKind::Down(event::MouseButton::Left) {
                        // These coordinates are hardcoded based on the UI layout
                        // A more robust solution would calculate them dynamically
                        let button_y = 15; // Approximate line number for the buttons
                        if mouse.row == button_y {
                            if (1..=3).contains(&mouse.column) {
                                // [-]
                                let current_duration = *tick_duration_tx.borrow();
                                let new_duration = current_duration * 2;
                                active_preset = None;
                                tick_duration_tx.send(new_duration).ok();
                            } else if (5..=7).contains(&mouse.column) {
                                // [+]
                                let current_duration = *tick_duration_tx.borrow();
                                let new_duration =
                                    (current_duration / 2).max(Duration::from_millis(1));
                                active_preset = None;
                                tick_duration_tx.send(new_duration).ok();
                            } else if (9..=11).contains(&mouse.column) {
                                // [R]
                                active_preset = Some('2');
                                tick_duration_tx.send(initial_tick_duration).ok();
                                timescale_tx.send(initial_years_per_tick).ok();
                                pause_tx.send(false).ok();
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        if ctrl_c_task.is_finished() {
            app_should_run = false;
        }
    }

    // Shutdown
    shutdown_notify.notify_waiters();
    simulation_task.await?;
    restore_terminal()?;
    term_guard.disarm();

    Ok(())
}

fn preset_status(active: Option<char>) -> Vec<PresetStatus> {
    SPEED_PRESETS
        .iter()
        .map(|preset| PresetStatus {
            key: preset.key,
            label: preset.label,
            intent: preset.intent,
            tick_ms: preset.tick_ms,
            years_per_tick: preset.years_per_tick,
            active: Some(preset.key) == active,
        })
        .collect()
}

fn apply_preset(
    key: char,
    tick_duration_tx: &watch::Sender<Duration>,
    timescale_tx: &watch::Sender<f64>,
) -> Option<char> {
    let preset = SPEED_PRESETS.iter().find(|p| p.key == key)?;
    tick_duration_tx.send(preset.duration()).ok();
    timescale_tx.send(preset.years_per_tick).ok();
    Some(key)
}

fn init_terminal() -> io::Result<Terminal<impl Backend>> {
    enable_raw_mode()?;
    stdout()
        .execute(EnterAlternateScreen)?
        .execute(event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout());
    Terminal::new(backend)
}

fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    stdout()
        .execute(LeaveAlternateScreen)?
        .execute(event::DisableMouseCapture)?;
    Ok(())
}

/// Ensures terminal is restored on panic/early-return.
struct TerminalGuard {
    armed: bool,
}

impl TerminalGuard {
    fn new() -> Self {
        Self { armed: true }
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        if self.armed {
            let _ = restore_terminal();
        }
    }
}

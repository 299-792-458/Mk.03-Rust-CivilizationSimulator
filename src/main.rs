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

    let observer = Arc::new(RwLock::new(ObserverSnapshot::default()));
    let shutdown_notify = Arc::new(Notify::new());

    let mut simulation = SimulationWorld::with_observer(config, observer.clone());
    let notify_for_simulation = shutdown_notify.clone();
    let simulation_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(*tick_duration_rx.borrow());
        loop {
            tokio::select! {
                _ = interval.tick() => simulation.tick(),
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
        terminal.draw(|frame| {
            let snapshot = observer.read().expect("Observer lock is poisoned").clone();
            let tick_duration = *tick_duration_tx.borrow();
            ui::render(frame, &snapshot, tick_duration);
        })?;

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') => app_should_run = false,
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        let current_duration = *tick_duration_tx.borrow();
                        let new_duration = (current_duration / 2).max(Duration::from_millis(1));
                        tick_duration_tx.send(new_duration).ok();
                    }
                    KeyCode::Char('-') => {
                        let current_duration = *tick_duration_tx.borrow();
                        let new_duration = current_duration * 2;
                        tick_duration_tx.send(new_duration).ok();
                    }
                    KeyCode::Char('<') | KeyCode::Char(',') => {
                        let current = *timescale_tx.borrow();
                        let new_scale = (current / 2.0).max(1_000.0);
                        timescale_tx.send(new_scale).ok();
                    }
                    KeyCode::Char('>') | KeyCode::Char('.') => {
                        let current = *timescale_tx.borrow();
                        let new_scale = (current * 2.0).min(50_000_000_000.0);
                        timescale_tx.send(new_scale).ok();
                    }
                    KeyCode::Char('r') => {
                        tick_duration_tx.send(initial_tick_duration).ok();
                        timescale_tx.send(initial_years_per_tick).ok();
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
                                tick_duration_tx.send(new_duration).ok();
                            } else if (5..=7).contains(&mouse.column) {
                                // [+]
                                let current_duration = *tick_duration_tx.borrow();
                                let new_duration =
                                    (current_duration / 2).max(Duration::from_millis(1));
                                tick_duration_tx.send(new_duration).ok();
                            } else if (9..=11).contains(&mouse.column) {
                                // [R]
                                tick_duration_tx.send(initial_tick_duration).ok();
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

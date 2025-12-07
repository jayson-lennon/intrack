use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

use crossterm::{
    cursor,
    event::{
        DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
        Event as CrosstermEvent, KeyEvent, KeyEventKind, MouseEvent,
    },
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use error_stack::{Report, ResultExt};
use futures::{FutureExt, StreamExt};
use ratatui::backend::CrosstermBackend as Backend;
use serde::{Deserialize, Serialize};
use tokio::{
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use wherror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    Init,
    Quit,
    Error,
    Closed,
    Tick,
    Render,
    FocusGained,
    FocusLost,
    Paste(String),
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}

#[derive(Debug)]
pub struct Tui {
    pub terminal: ratatui::Terminal<Backend<std::io::Stderr>>,
    pub task: JoinHandle<Result<(), Report<TuiError>>>,
    pub cancellation_token: CancellationToken,
    pub event_rx: UnboundedReceiver<Event>,
    pub event_tx: UnboundedSender<Event>,
    pub frame_rate: f64,
    pub tick_rate: f64,
    pub mouse: bool,
    pub paste: bool,
}

#[derive(Debug, Error)]
#[error(debug)]
pub struct TuiError;

impl Tui {
    pub fn new() -> Result<Self, Report<TuiError>> {
        let tick_rate = 4.0;
        let frame_rate = 60.0;
        let terminal =
            ratatui::Terminal::new(Backend::new(std::io::stderr())).change_context(TuiError)?;
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let cancellation_token = CancellationToken::new();
        let task = tokio::spawn(async { Ok(()) });
        let mouse = false;
        let paste = false;
        Ok(Self {
            terminal,
            task,
            cancellation_token,
            event_rx,
            event_tx,
            frame_rate,
            tick_rate,
            mouse,
            paste,
        })
    }

    #[must_use]
    pub fn with_tick_rate(mut self, tick_rate: f64) -> Self {
        self.tick_rate = tick_rate;
        self
    }

    #[must_use]
    pub fn with_frame_rate(mut self, frame_rate: f64) -> Self {
        self.frame_rate = frame_rate;
        self
    }

    #[must_use]
    pub fn enable_mouse(mut self, mouse: bool) -> Self {
        self.mouse = mouse;
        self
    }

    #[must_use]
    pub fn enable_paste(mut self, paste: bool) -> Self {
        self.paste = paste;
        self
    }

    pub fn start(&mut self) -> Result<(), Report<TuiError>> {
        let tick_delay = std::time::Duration::from_secs_f64(1.0 / self.tick_rate);
        let render_delay = std::time::Duration::from_secs_f64(1.0 / self.frame_rate);
        self.cancel();
        self.cancellation_token = CancellationToken::new();
        let cancellation_token = self.cancellation_token.clone();
        let event_tx = self.event_tx.clone();
        self.task = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut tick_interval = tokio::time::interval(tick_delay);
            let mut render_interval = tokio::time::interval(render_delay);
            event_tx
                .send(Event::Init)
                .change_context(TuiError)
                .attach("failed to send init event")?;
            loop {
                let tick_delay = tick_interval.tick();
                let render_delay = render_interval.tick();
                let crossterm_event = reader.next().fuse();
                tokio::select! {
                  () = cancellation_token.cancelled() => {
                    break;
                  }
                  maybe_event = crossterm_event => {
                    match maybe_event {
                      Some(Ok(evt)) => {
                        match evt {
                          CrosstermEvent::Key(key) => {
                            if key.kind == KeyEventKind::Press {
                              event_tx.send(Event::Key(key)).change_context(TuiError).attach("failed to forward Key event")?;
                            }
                          },
                          CrosstermEvent::Mouse(mouse) => {
                            event_tx.send(Event::Mouse(mouse)).change_context(TuiError).attach("failed to forward Mouse event")?;
                          },
                          CrosstermEvent::Resize(x, y) => {
                            event_tx.send(Event::Resize(x, y)).change_context(TuiError).attach("failed to forward Resize event")?;
                          },
                          CrosstermEvent::FocusLost => {
                            event_tx.send(Event::FocusLost).change_context(TuiError).attach("failed to forward FocusLost event")?;
                          },
                          CrosstermEvent::FocusGained => {
                            event_tx.send(Event::FocusGained).change_context(TuiError).attach("failed to forward FocusGained event")?;
                          },
                          CrosstermEvent::Paste(s) => {
                            event_tx.send(Event::Paste(s)).change_context(TuiError).attach("failed to forward Paste event")?;
                          },
                        }
                      }
                      Some(Err(_)) => {
                        event_tx.send(Event::Error).change_context(TuiError)?;
                      }
                      None => {},
                    }
                  },
                  _ = tick_delay => {
                      event_tx.send(Event::Tick).change_context(TuiError)?;
                  },
                  _ = render_delay => {
                      event_tx.send(Event::Render).change_context(TuiError)?;
                  },
                }
            }
            Ok(())
        });

        Ok(())
    }

    pub fn stop(&self) -> Result<(), Report<TuiError>> {
        self.cancel();
        let mut counter = 0;
        while !self.task.is_finished() {
            std::thread::sleep(Duration::from_millis(1));
            counter += 1;
            if counter > 50 {
                self.task.abort();
            }
            if counter > 100 {
                tracing::error!("Failed to abort task in 100 milliseconds for unknown reason");
                break;
            }
        }
        Ok(())
    }

    pub fn enter(&mut self) -> Result<(), Report<TuiError>> {
        crossterm::terminal::enable_raw_mode().change_context(TuiError)?;
        crossterm::execute!(std::io::stderr(), EnterAlternateScreen, cursor::Hide)
            .change_context(TuiError)?;
        if self.mouse {
            crossterm::execute!(std::io::stderr(), EnableMouseCapture).change_context(TuiError)?;
        }
        if self.paste {
            crossterm::execute!(std::io::stderr(), EnableBracketedPaste)
                .change_context(TuiError)?;
        }
        self.start()
    }

    pub fn exit(&mut self) -> Result<(), Report<TuiError>> {
        self.stop()?;
        if crossterm::terminal::is_raw_mode_enabled().change_context(TuiError)? {
            self.flush().change_context(TuiError)?;
            if self.paste {
                crossterm::execute!(std::io::stderr(), DisableBracketedPaste)
                    .change_context(TuiError)?;
            }
            if self.mouse {
                crossterm::execute!(std::io::stderr(), DisableMouseCapture)
                    .change_context(TuiError)?;
            }
            crossterm::execute!(std::io::stderr(), LeaveAlternateScreen, cursor::Show)
                .change_context(TuiError)?;
            crossterm::terminal::disable_raw_mode().change_context(TuiError)?;
        }
        Ok(())
    }

    pub fn cancel(&self) {
        self.cancellation_token.cancel();
    }

    pub fn suspend(&mut self) -> Result<(), Report<TuiError>> {
        self.exit().change_context(TuiError)?;
        // #[cfg(not(windows))]
        // signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP)
        //     .change_context(TuiError)?;
        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), Report<TuiError>> {
        self.enter().change_context(TuiError)?;
        Ok(())
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.event_rx.recv().await
    }
}

impl Deref for Tui {
    type Target = ratatui::Terminal<Backend<std::io::Stderr>>;

    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

impl DerefMut for Tui {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        self.exit().expect("failure while exiting TUI backend");
    }
}

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

/// Events that can occur in the terminal user interface.
///
/// This enum represents all possible events that the TUI can handle,
/// including user input (keyboard, mouse), terminal events (resize, focus),
/// and internal timing events (tick, render).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    /// Initial event sent when the TUI starts
    Init,
    /// Quit event, typically triggered by user request
    Quit,
    /// Error event, sent when an error occurs
    Error,
    /// Closed event, sent when the TUI is closed
    Closed,
    /// Tick event, sent periodically at the configured tick rate
    Tick,
    /// Render event, sent periodically at the configured frame rate
    Render,
    /// Focus gained event, sent when the terminal gains focus
    FocusGained,
    /// Focus lost event, sent when the terminal loses focus
    FocusLost,
    /// Paste event, sent when text is pasted into the terminal
    /// Contains the pasted text
    Paste(String),
    /// Key event, sent when a key is pressed
    /// Contains the key event details
    Key(KeyEvent),
    /// Mouse event, sent when mouse interaction occurs
    /// Contains the mouse event details
    Mouse(MouseEvent),
    /// Resize event, sent when the terminal is resized
    /// Contains the new width and height
    Resize(u16, u16),
}

/// Terminal User Interface wrapper for handling terminal events and rendering.
///
/// This struct manages the terminal state, event handling, and rendering loop.
/// It provides a high-level interface for building TUI applications using ratatui
/// and crossterm. The TUI runs in raw mode with an alternate screen buffer
/// and can handle keyboard, mouse, and terminal resize events.
#[derive(Debug)]
pub struct Tui {
    /// The underlying ratatui terminal instance
    pub terminal: ratatui::Terminal<Backend<std::io::Stderr>>,
    /// Background task handle for the event loop
    pub task: JoinHandle<Result<(), Report<TuiError>>>,
    /// Cancellation token for stopping the event loop
    pub cancellation_token: CancellationToken,
    /// Receiver channel for events
    pub event_rx: UnboundedReceiver<Event>,
    /// Sender channel for events
    pub event_tx: UnboundedSender<Event>,
    /// Target frame rate for render events (frames per second)
    pub frame_rate: f64,
    /// Target tick rate for tick events (ticks per second)
    pub tick_rate: f64,
    /// Whether mouse capture is enabled
    pub mouse: bool,
    /// Whether bracketed paste mode is enabled
    pub paste: bool,
}

/// Error type for TUI operations.
///
/// This error is returned when TUI operations fail, such as entering/exiting
/// raw mode, starting/stopping the event loop, or sending/receiving events.
#[derive(Debug, Error)]
#[error(debug)]
pub struct TuiError;

impl Tui {
    /// Creates a new TUI instance with default settings.
    ///
    /// Initializes a terminal with default tick rate of 4.0 Hz, frame rate of 60.0 Hz,
    /// and mouse capture and bracketed paste disabled.
    ///
    /// # Errors
    ///
    /// Returns an error if the terminal cannot be initialized.
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

    /// Sets the tick rate for periodic tick events.
    ///
    /// The tick rate determines how often `Event::Tick` events are generated
    /// (ticks per second). This is independent of the frame rate.
    #[must_use]
    pub fn with_tick_rate(mut self, tick_rate: f64) -> Self {
        self.tick_rate = tick_rate;
        self
    }

    /// Sets the frame rate for periodic render events.
    ///
    /// The frame rate determines how often `Event::Render` events are generated
    /// (frames per second). This is independent of the tick rate.
    #[must_use]
    pub fn with_frame_rate(mut self, frame_rate: f64) -> Self {
        self.frame_rate = frame_rate;
        self
    }

    /// Enables or disables mouse capture.
    ///
    /// When enabled, the TUI will capture mouse events and generate `Event::Mouse`
    /// for mouse interactions. Requires entering raw mode to take effect.
    #[must_use]
    pub fn enable_mouse(mut self, mouse: bool) -> Self {
        self.mouse = mouse;
        self
    }

    /// Enables or disables bracketed paste mode.
    ///
    /// When enabled, pasted text will be bracketed with paste control sequences,
    /// allowing the TUI to distinguish between typed and pasted text. Generates
    /// `Event::Paste` events for pasted content. Requires entering raw mode to take effect.
    #[must_use]
    pub fn enable_paste(mut self, paste: bool) -> Self {
        self.paste = paste;
        self
    }

    /// Starts the TUI event loop and background tasks.
    ///
    /// Spawns a background task that listens for terminal events (keyboard, mouse,
    /// resize, focus, paste) and periodically generates tick and render events.
    /// Sends an `Event::Init` event to signal initialization.
    ///
    /// The event loop runs asynchronously and can be stopped by calling `stop()`
    /// or `cancel()`. Events can be received using the `next()` method.
    ///
    /// # Errors
    ///
    /// Returns an error if the event loop cannot be started or if sending
    /// the initial event fails.
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

    /// Stops the TUI event loop and waits for tasks to complete.
    ///
    /// Cancels the event loop and waits up to 100ms for the background task to finish.
    /// If the task doesn't complete within 50ms, it will be aborted. After 100ms,
    /// the function returns regardless of task state.
    ///
    /// # Errors
    ///
    /// This function always returns `Ok` as errors are handled internally.
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
                break;
            }
        }
        Ok(())
    }

    /// Enters raw mode and alternate screen buffer.
    ///
    /// Enables raw mode for the terminal, enters the alternate screen buffer,
    /// and hides the cursor. Optionally enables mouse capture and bracketed paste
    /// mode based on the configuration set by `enable_mouse()` and `enable_paste()`.
    ///
    /// # Errors
    ///
    /// Returns an error if raw mode cannot be enabled or if terminal commands fail.
    pub fn enter_raw_mode(&mut self) -> Result<(), Report<TuiError>> {
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
        Ok(())
    }

    /// Exits raw mode and returns to the normal screen buffer.
    ///
    /// Exits raw mode, returns to the normal screen buffer, shows the cursor,
    /// and disables mouse capture and bracketed paste mode if they were enabled.
    /// Only performs cleanup if raw mode is currently enabled.
    ///
    /// # Errors
    ///
    /// Returns an error if raw mode detection fails or if terminal commands fail.
    pub fn exit_raw_mode(&mut self) -> Result<(), Report<TuiError>> {
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

    /// Cancels the running event loop.
    ///
    /// Signals the background event loop task to stop by cancelling the
    /// cancellation token. The task will exit on the next iteration.
    pub fn cancel(&self) {
        self.cancellation_token.cancel();
    }

    /// Suspends the TUI by exiting raw mode.
    ///
    /// Exits raw mode and returns the terminal to a normal state. This allows
    /// the terminal to be used for other purposes (e.g., shell commands).
    /// Can be resumed by calling `resume()`.
    ///
    /// # Errors
    ///
    /// Returns an error if exiting raw mode fails.
    ///
    /// # Notes
    ///
    /// The commented-out signal handling code would allow sending SIGTSTP
    /// on Unix systems to suspend the process, but this is currently disabled.
    pub fn suspend(&mut self) -> Result<(), Report<TuiError>> {
        self.exit_raw_mode().change_context(TuiError)?;
        // #[cfg(not(windows))]
        // signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP)
        //     .change_context(TuiError)?;
        Ok(())
    }

    /// Resumes the TUI by entering raw mode.
    ///
    /// Re-enters raw mode and restores the alternate screen buffer after
    /// the TUI has been suspended. Should be called after `suspend()`.
    ///
    /// # Errors
    ///
    /// Returns an error if entering raw mode fails.
    pub fn resume(&mut self) -> Result<(), Report<TuiError>> {
        self.enter_raw_mode().change_context(TuiError)?;
        Ok(())
    }

    /// Retrieves the next event from the event queue.
    ///
    /// Asynchronously waits for and returns the next event. Returns `None`
    /// if the event sender has been dropped (typically when the TUI is stopped).
    ///
    /// # Examples
    ///
    /// ```
    /// # use intrack::feat::tui::Tui;
    /// # tokio_test::block_on(async {
    /// let mut tui = Tui::new().unwrap();
    /// tui.start().unwrap();
    /// if let Some(event) = tui.next().await {
    ///     // Handle the event
    /// }
    /// # })
    /// ```
    pub async fn next(&mut self) -> Option<Event> {
        self.event_rx.recv().await
    }
}

/// Allows immutable access to the underlying terminal through dereferencing.
///
/// This implementation enables direct use of terminal methods on the `Tui` struct
/// by automatically dereferencing to the internal terminal instance.
impl Deref for Tui {
    type Target = ratatui::Terminal<Backend<std::io::Stderr>>;

    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

/// Allows mutable access to the underlying terminal through dereferencing.
///
/// This implementation enables direct use of mutable terminal methods on the `Tui`
/// struct by automatically dereferencing to the internal terminal instance.
impl DerefMut for Tui {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}

/// Automatic cleanup when the Tui instance is dropped.
///
/// Ensures that raw mode is properly exited when the Tui instance is dropped,
/// returning the terminal to a normal state. This prevents the terminal from
/// remaining in raw mode if the Tui is dropped unexpectedly.
///
/// # Panics
///
/// Panics if exiting raw mode fails during drop. This is a critical error
/// that should not occur in normal circumstances.
impl Drop for Tui {
    fn drop(&mut self) {
        self.exit_raw_mode()
            .expect("failure while exiting TUI backend");
    }
}

use crate::errors::EmulationError;
use crate::errors::EmulationError::Quit;
use crossterm::event::Event::Key;
use crossterm::event::{EventStream, KeyCode, KeyEventKind};
use std::time::{Duration, Instant};

use futures::{future::FutureExt, select, StreamExt};
use futures_timer::Delay;

///
/// Read keypad state, but only block read until timeout is reached. if timeout is reached,
/// it means that no keys have been input.
///
/// On Chip8, keypad looks like this :
/// ```
/// | 1 | 2 | 3 | C |
/// | 4 | 5 | 6 | D |
/// | 7 | 8 | 9 | E |
/// | A | 0 | B | F |
/// ```
/// It's mapped on the left side of the keyboard from keys 1 to 4 (left to right),
/// through 1 to z (top to bottom)
///
pub(crate) async fn async_read_keypad_state(
    event_stream: &mut EventStream,
    duration: Duration,
) -> Result<[bool; 16], EmulationError> {
    let last_tick = Instant::now();
    let mut keypad_state = [false; 16];
    let mut remaining_duration = duration;
    loop {
        let mut delay = Delay::new(remaining_duration).fuse();

        let mut event = event_stream.next().fuse();

        select! {
            _ = delay => return Ok(keypad_state),
            maybe_event = event => {
                match maybe_event {
                    Some(Ok(Key(key))) => {
                        if key.kind == KeyEventKind::Press {

                            match key.code {
                                KeyCode::Char('p') => return Err(Quit),

                                KeyCode::Char('1') => { keypad_state.fill(false); keypad_state[0x1] = true },
                                KeyCode::Char('2') => { keypad_state.fill(false); keypad_state[0x2] = true },
                                KeyCode::Char('3') => { keypad_state.fill(false); keypad_state[0x3] = true },
                                KeyCode::Char('4') => { keypad_state.fill(false); keypad_state[0xC] = true },

                                KeyCode::Char('q') => { keypad_state.fill(false); keypad_state[0x4] = true },
                                KeyCode::Char('w') => { keypad_state.fill(false); keypad_state[0x5] = true },
                                KeyCode::Char('e') => { keypad_state.fill(false); keypad_state[0x6] = true },
                                KeyCode::Char('r') => { keypad_state.fill(false); keypad_state[0xD] = true },

                                KeyCode::Char('a') => { keypad_state.fill(false); keypad_state[0x7] = true },
                                KeyCode::Char('s') => { keypad_state.fill(false); keypad_state[0x8] = true },
                                KeyCode::Char('d') => { keypad_state.fill(false); keypad_state[0x9] = true },
                                KeyCode::Char('f') => { keypad_state.fill(false); keypad_state[0xE] = true },

                                KeyCode::Char('z') => { keypad_state.fill(false); keypad_state[0xA] = true },
                                KeyCode::Char('x') => { keypad_state.fill(false); keypad_state[0x0] = true },
                                KeyCode::Char('c') => { keypad_state.fill(false); keypad_state[0xB] = true },
                                KeyCode::Char('v') => { keypad_state.fill(false); keypad_state[0xF] = true },
                                _ => {}
                            }
                        }
                    }
                    Some(Err(e)) => return Err(EmulationError::UnknownInput),
                    _ => return Ok([false; 16]),
                }
            }
        }
        remaining_duration = remaining_duration.saturating_sub(last_tick.elapsed());
        if remaining_duration.as_millis() == 0 {
            return Ok(keypad_state);
        }
    }
}

pub(crate) fn read_keypad_state(
    event_stream: &mut EventStream,
) -> Result<[bool; 16], EmulationError> {
    async_std::task::block_on(async_read_keypad_state(
        event_stream,
        Duration::from_millis(1),
    ))
}

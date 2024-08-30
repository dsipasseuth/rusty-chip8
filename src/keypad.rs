use crate::keypad::KeypadEvent::{Clear, Quit};
use async_std::channel::Sender;
use async_std::task::JoinHandle;
use crossterm::event::Event::Key;
use crossterm::event::{EventStream, KeyCode, KeyEventKind};
use futures::{future::FutureExt, select, StreamExt};
use futures_timer::Delay;
use std::time::Duration;

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
pub(crate) async fn async_listen_keypad_state(keypad_listener: Sender<KeypadEvent>) {
    let mut event_stream = EventStream::new();
    loop {
        let mut delay = Delay::new(Duration::from_millis(500)).fuse();

        let mut event = event_stream.next().fuse();

        select! {
            _ = delay => { keypad_listener.send(Clear).await.unwrap(); },
            maybe_event = event => {
                match maybe_event {
                    Some(Ok(Key(key))) => {
                        if key.kind == KeyEventKind::Press {
                            match key.code {
                                KeyCode::Char('p') => { keypad_listener.send(Quit).await.unwrap(); },

                                KeyCode::Char('1') => { keypad_listener.send(KeypadEvent::Keypad(0x1u8)).await.unwrap(); },
                                KeyCode::Char('2') => { keypad_listener.send(KeypadEvent::Keypad(0x2u8)).await.unwrap(); },
                                KeyCode::Char('3') => { keypad_listener.send(KeypadEvent::Keypad(0x3u8)).await.unwrap(); },
                                KeyCode::Char('4') => { keypad_listener.send(KeypadEvent::Keypad(0xCu8)).await.unwrap(); },

                                KeyCode::Char('q') => { keypad_listener.send(KeypadEvent::Keypad(0x4u8)).await.unwrap(); },
                                KeyCode::Char('w') => { keypad_listener.send(KeypadEvent::Keypad(0x5u8)).await.unwrap(); },
                                KeyCode::Char('e') => { keypad_listener.send(KeypadEvent::Keypad(0x6u8)).await.unwrap(); },
                                KeyCode::Char('r') => { keypad_listener.send(KeypadEvent::Keypad(0xDu8)).await.unwrap(); },

                                KeyCode::Char('a') => { keypad_listener.send(KeypadEvent::Keypad(0x7u8)).await.unwrap(); },
                                KeyCode::Char('s') => { keypad_listener.send(KeypadEvent::Keypad(0x8u8)).await.unwrap(); },
                                KeyCode::Char('d') => { keypad_listener.send(KeypadEvent::Keypad(0x9u8)).await.unwrap(); },
                                KeyCode::Char('f') => { keypad_listener.send(KeypadEvent::Keypad(0xEu8)).await.unwrap(); },

                                KeyCode::Char('z') => { keypad_listener.send(KeypadEvent::Keypad(0xAu8)).await.unwrap(); },
                                KeyCode::Char('x') => { keypad_listener.send(KeypadEvent::Keypad(0x0u8)).await.unwrap(); },
                                KeyCode::Char('c') => { keypad_listener.send(KeypadEvent::Keypad(0xBu8)).await.unwrap(); },
                                KeyCode::Char('v') => { keypad_listener.send(KeypadEvent::Keypad(0xFu8)).await.unwrap(); },
                                _ => {},
                            }
                        }
                    }
                    _ => {},
                }
            }
        }
    }
}

pub(crate) fn spawn_keypad_handler(keypad_listener: Sender<KeypadEvent>) -> JoinHandle<()> {
    async_std::task::spawn(async_listen_keypad_state(keypad_listener))
}

pub(crate) enum KeypadEvent {
    Clear,
    Keypad(u8),
    Quit,
}

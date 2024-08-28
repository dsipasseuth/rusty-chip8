use crate::errors::EmulationError;
use crate::errors::EmulationError::Quit;
use crossterm::event;
use crossterm::event::Event::Key;
use crossterm::event::{KeyCode, KeyEventKind};
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
pub fn read_keypad_state(timeout: Duration) -> Result<[bool; 16], EmulationError> {
    if event::poll(timeout).map_err(|err| EmulationError::UnknownInput)? {
        if let Key(key) = event::read().map_err(|err| EmulationError::UnknownInput)? {
            let mut keypad_state = [false; 16];
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('p') => return Err(Quit),

                    KeyCode::Char('0') => keypad_state[0] = true,
                    KeyCode::Char('1') => keypad_state[1] = true,
                    KeyCode::Char('2') => keypad_state[2] = true,
                    KeyCode::Char('4') => keypad_state[12] = true,

                    KeyCode::Char('q') => keypad_state[3] = true,
                    KeyCode::Char('w') => keypad_state[4] = true,
                    KeyCode::Char('e') => keypad_state[5] = true,
                    KeyCode::Char('r') => keypad_state[13] = true,

                    KeyCode::Char('a') => keypad_state[6] = true,
                    KeyCode::Char('s') => keypad_state[7] = true,
                    KeyCode::Char('d') => keypad_state[8] = true,
                    KeyCode::Char('f') => keypad_state[14] = true,

                    KeyCode::Char('z') => keypad_state[10] = true,
                    KeyCode::Char('x') => keypad_state[9] = true,
                    KeyCode::Char('c') => keypad_state[11] = true,
                    KeyCode::Char('v') => keypad_state[15] = true,
                    _ => {}
                }
                return Ok(keypad_state);
            }
        }
    }
    Ok([false; 16])
}

mod chip8;
mod errors;
mod keypad;

use std::env;
use std::fs;

use crate::chip8::Chip8;
use ratatui::symbols::Marker;
use ratatui::{
    crossterm::{
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    prelude::*,
    widgets::{canvas::*, *},
};

use crate::errors::EmulationError;
use std::{
    io::{self, stdout, Stdout},
    time::{Duration, Instant},
};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    println!("Loading {:?}", args);
    let rom_path = &args[1];
    let contents = fs::read(rom_path).expect("Cannot read file");

    let mut terminal = init_terminal()?;

    let mut keypad_state;

    // pooling time.
    let mut last_tick = Instant::now();

    // 60hz
    let tick_rate = Duration::from_millis(16);
    //let tick_rate = Duration::from_millis(160);

    let mut vm = Chip8::default();

    vm.load(contents);
    loop {
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        // perform one cycle
        if last_tick.elapsed() >= tick_rate {
            let _ = terminal.draw(|frame| {
                let [top, bottom] =
                    Layout::vertical([Constraint::Percentage(70), Constraint::Fill(1)]).areas(frame.area());
                let [top_left, top_right] =
                    Layout::horizontal([Constraint::Percentage(35), Constraint::Fill(1)]).areas(top);
                frame.render_widget(as_canvas(&vm), top_left);
                frame.render_widget(as_debug(&vm), top_right);
                frame.render_widget(as_instruction(), bottom);
            });
            match keypad::read_keypad_state(timeout) {
                Err(err) => match err {
                    EmulationError::UnknownOpcode(opcode) => {
                        panic!("error happened with opcode {:?}", opcode)
                    }
                    EmulationError::UnknownInput => panic!("error happened unknown input"),
                    EmulationError::Quit => break,
                },
                Ok(new_state) => keypad_state = new_state,
            }
            if let Err(error) = vm.cycle(keypad_state) {
                match error {
                    EmulationError::UnknownOpcode(opcode) => panic!("something wrong happened, {:?}", opcode),
                    EmulationError::UnknownInput => panic!("wrong input"),
                    EmulationError::Quit => break
                }
            }
            last_tick = Instant::now();
        }
    }
    restore_terminal()
}

///
/// Returns points in the canvas screen referential.
/// Chip8 have a top left coordinates being (0,0),
/// while ratatui works with the bottom left coordinates being (0,0)
///
fn as_points(vm: &Chip8) -> Vec<(f64, f64)> {
    let mut y_axis = 32;
    let mut x_axis = 0;
    let mut coords = vec![];
    for pixel in vm.gfx {
        if pixel {
            coords.push((x_axis as f64, y_axis as f64))
        }
        x_axis += 1;
        if x_axis % 64 == 0 {
            y_axis -= 1;
            x_axis = 0;
        }
    }
    coords
}

fn as_canvas(vm: &Chip8) -> impl Widget {
    let coords = as_points(vm);
    Canvas::default()
        .block(Block::bordered().title("Screen"))
        .marker(Marker::Block)
        .x_bounds([0.0, 64.0])
        .y_bounds([0.0, 32.0])
        .paint(move |ctx| {
            ctx.draw(&Points {
                coords: &coords,
                color: Color::default(),
            });
        })
}

fn as_debug(vm: &Chip8) -> impl Widget {
    let mut content = String::new();
    vm.debug_log.iter().for_each(|line| {
        content.push_str(line);
        content.push('\n');
    });
    Paragraph::new(content)
        .block(Block::bordered().title("Debug Logs"))
}

fn as_instruction() -> impl Widget {
    Paragraph::new("Press 'p' to quit.")
        .block(Block::bordered().title("Instructions"))
}

fn init_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

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

use std::{
    io::{self, stdout, Stdout},
    time::{Duration, Instant},
};
use ratatui::layout::Flex;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    println!("Loading {:?}", args);
    let rom_path = &args[1];
    let contents = fs::read(rom_path).expect("Cannot read file");

    let mut terminal = init_terminal()?;

    let mut keypad_state ;

    // pooling time.
    let mut last_tick = Instant::now();

    // 60hz
    let tick_rate = Duration::from_millis(16);

    let mut vm = Chip8::default();

    vm.load(contents);
    loop {
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        let _ = terminal.draw(|frame| {
            if vm.draw_flag {
                let vertical_layout =
                    Layout::vertical([Constraint::Percentage(80), Constraint::Percentage(20)]);
                let [top, bottom] = vertical_layout.areas(frame.area());
                let centered_top = center(top, Constraint::Length(64), Constraint::Length(32));
                frame.render_widget(as_canvas(&vm), centered_top);
                frame.render_widget(as_instruction(&vm), bottom);
            }
        });

        // perform one cycle
        if last_tick.elapsed() >= tick_rate {
            match keypad::read_keypad_state(timeout) {
                Err(_) => break,
                Ok(new_state) => keypad_state = new_state,
            }
            if let Err(error) = vm.cycle(keypad_state) {
                panic!("something wrong happened, {:?}", error)
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

fn as_instruction(vm: &Chip8) -> impl Widget {
    let coords = as_points(&vm);
    Paragraph::new(format!("Press 'p' to quit."))
        .block(Block::bordered().title("Instructions"))
        .black()
        .on_white()
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
fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

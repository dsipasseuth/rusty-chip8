mod chip8;
use std::env;
use std::fs;

use crate::chip8::Chip8;
use ratatui::crossterm::event;
use ratatui::crossterm::event::{Event, KeyCode};
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

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    println!("Loading {:?}", args);
    let rom_path = &args[1];
    let contents = fs::read(rom_path).expect("Cannot read file");

    let mut terminal = init_terminal()?;

    // pooling time.
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(16);

    let mut vm = chip8::init();
    vm.load(contents);
    loop {
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        let _ = terminal.draw(|frame| {
            if vm.draw_flag() {
                let vertical_layout =
                    Layout::vertical([Constraint::Percentage(90), Constraint::Percentage(10)]);
                let [top, bottom] = vertical_layout.areas(frame.area());
                frame.render_widget(as_canvas(&vm), top);
                frame.render_widget(as_instruction(), bottom);
            }
        });

        // perform one cycle
        if last_tick.elapsed() >= tick_rate {
            vm.cycle();
            last_tick = Instant::now();
        }
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    _ => {}
                }
            }
        }
        vm.set_keys();
    }
    restore_terminal()
}

fn as_points(vm: &Chip8) -> Vec<(f64, f64)> {
    let mut y_axis = 0f64;
    let mut x_axis = 0f64;
    let mut breakline = 0;
    let mut coords = Vec::new();
    for pixel in vm.get_gfx() {
        if pixel {
            coords.push((x_axis, y_axis))
        }
        breakline += 1;
        if breakline % 64 == 0 {
            y_axis += 1.0;
            x_axis = 0f64;
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
        .y_bounds([0.0, 64.0])
        .paint(move |ctx| {
            ctx.draw(&Points {
                coords: &coords,
                color: Color::default(),
            });
        })
}

fn as_instruction() -> impl Widget {
    Paragraph::new("Press 'q' to quit").white().on_blue()
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
mod chip8;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("Loading {:?}", args);
    let rom_path = &args[1];
    let contents = fs::read(rom_path)
        .expect("Cannot read file");
    
    let mut vm = chip8::init();
    vm.load(contents);

    loop {
        vm.cycle();
        if vm.draw_flag() {
            let mut breakline = 0;
            println!("==================================================================");
            for pixel in vm.get_gfx().iter() {
                if breakline == 0 {
                    print!("|")
                }
                if *pixel {
                    print!("X");
                } else {
                    print!(" ")
                }
                breakline += 1;
                if breakline % 64 == 0 {
                    breakline = 0;
                    println!("|");
                }
            }
            println!("==================================================================");
        }

        vm.set_keys();
    }
}
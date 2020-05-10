use rand::Rng;

pub struct Chip8 {
    op_code: u16,
    // also named PC
    // This is where to read the op code in memory
    program_counter: u16,
    memory: [u8; 4096],
    // also named V
    register: [u8; 16],
    memory_index: u16, // also named I
    // 64x32 pixel
    gfx: [bool; 2048],
    delay_timer: u8,
    sound_timer: u8,
    stack: Vec<u16>,
    keys: [bool; 16],
    draw_flag: bool,
    rng:rand::prelude::ThreadRng,
}

impl Chip8 {
    
    pub fn load(&mut self, bytes: Vec<u8>) {
        let mut i = 512;
        for v in &bytes {
            self.memory[i] = *v;
            i = i + 1;
        }
        println!("Rom Loaded into memory");
    }

    pub fn set_keys(&mut self) {
        
    }

    pub fn get_gfx(&self) -> [bool; 2048] {
        self.gfx
    }

    fn read_op_code(&self) -> u16 {
        (self.memory[self.program_counter as usize] as u16) << 8 | self.memory[(self.program_counter + 1) as usize] as u16
    }

    // Register X is always located at the same position in opcode.
    fn read_vx(&self) -> u8 {
        self.register[((self.op_code & 0x0F00) >> 8) as usize]
    }

    // Write at vx location.
    fn write_vx(&mut self, value: u8) {
        self.register[((self.op_code & 0x0F00) >> 8) as usize] = value;
    }
    
    // Register Y if available is always located at the same position in opcode.
    fn read_vy(&self) -> u8 {
        self.register[((self.op_code & 0x00F0) >> 4) as usize]
    }

    fn set_program_counter(&mut self, index: u16) {
        self.program_counter = index;
    }

    fn increase_program_counter_if(&mut self, condition: bool) {
        if condition { self.increase_program_counter() }
    }

    fn increase_program_counter(&mut self) {
        self.program_counter += 2;
    }

    fn call_at(&mut self, address: u16) {
        self.stack.push(self.program_counter);
        self.program_counter = address;
    }

    fn draw(&mut self, x:u8, y:u8, height:u8) {
        for y_row in 0..height {
            self.register[0xF] = 0;
            let sprite = self.memory[(self.memory_index + y_row as u16) as usize];
            for x_col in 0..8 {
                if (sprite & (0x80 >> x_col)) > 0 {
                    let gfx_loc:usize = (x as usize + x_col as usize + (y as usize + y_row as usize) * 64) % 2048;
                    if self.gfx[gfx_loc] == true {
                        self.register[0xF] = 1
                    } else {
                        self.gfx[gfx_loc] ^= true
                    }
                }
            }
        }
        self.draw_flag = true;
    }

    fn register_dump(&mut self, reg_max: u8) {
        for reg_index in 0..reg_max {
            self.memory[self.memory_index as usize + reg_index as usize] = self.register[reg_index as usize];
        }
        self.memory[self.memory_index as usize + reg_max as usize] = self.register[reg_max as usize];
    }

    fn register_load(&mut self, reg_max: u8) {
        for reg_index in 0..reg_max as usize {
            self.register[reg_index] = self.memory[self.memory_index as usize + reg_index];
        }
        self.register[reg_max as usize] = self.memory[self.memory_index as usize + reg_max as usize];
    }

    pub fn cycle(&mut self) {
        // Fetch Opcode
        self.op_code = self.read_op_code();
        // Decode Opcode
        // Op code list : https://en.wikipedia.org/wiki/CHIP-8#Opcode_table
        match self.op_code & 0xF000 {
            0x0000 => {
                match self.op_code {                
                    0x00E0 => {
                        self.gfx.iter_mut().for_each(|x| *x = false);
                        self.increase_program_counter();
                    },
                    0x00EE => {
                        self.program_counter = self.stack.pop().unwrap();
                    },
                    _ => println!("Call RCA 1082"),
                }
            }
            0x1000 => self.set_program_counter(self.op_code & 0x0FFF),
            0x2000 => self.call_at(self.op_code & 0x0FFF),
            0x3000 => {
                self.increase_program_counter_if(self.read_vx() == (self.op_code & 0x00FF) as u8);
                self.increase_program_counter();
            },
            0x4000 => {
                self.increase_program_counter_if(self.read_vx() != (self.op_code & 0x00FF) as u8);
                self.increase_program_counter();
            },
            0x5000 => {
                self.increase_program_counter_if(self.read_vx() != self.read_vy());
                self.increase_program_counter();
            },
            0x6000 => {
                self.write_vx((self.op_code & 0x00FF) as u8);
                self.increase_program_counter();
            },
            0x7000 => {
                let (result, _) = self.read_vx().overflowing_add((self.op_code & 0x00FF) as u8);
                self.write_vx(result);
                self.increase_program_counter();
            },
            0x8000 => {
                match self.op_code & 0x000F {
                    0x0000 => self.write_vx(self.read_vy()),
                    0x0001 => self.write_vx(self.read_vx() | self.read_vy()),
                    0x0002 => self.write_vx(self.read_vx() & self.read_vy()),
                    0x0003 => self.write_vx(self.read_vx() ^ self.read_vy()),
                    0x0004 => { 
                        let (result, carry) = self.read_vx().overflowing_add(self.read_vy());
                        self.write_vx(result);
                        self.register[0xF] = if carry { 1 } else { 0 };
                    },
                    0x0005 => {
                        let (result, carry) = self.read_vx().overflowing_sub(self.read_vy());
                        self.write_vx(result);
                        self.register[0xF] = if carry { 1 } else { 0 };
                    },
                    0x0006 => self.register[0xF] = self.read_vx() >> 1,
                    0x0007 => {
                        let (result, _) =  self.read_vy().overflowing_sub(self.read_vx());
                        self.write_vx(result);
                    },
                    0x000E => self.register[0xF] = self.read_vx() << 1,
                    _ => println!("Something wrong")
                };
                self.increase_program_counter();
            },
            0x9000 => {
                self.increase_program_counter_if(self.read_vy() != self.read_vy());
                self.increase_program_counter();
            },
            0xA000 => {
                self.memory_index = self.op_code & 0x0FFF;
                self.increase_program_counter();
            },
            0xB000 => {
                let v0:u16 = self.register[0] as u16;
                self.set_program_counter((self.op_code & 0x0FFF) + v0);
            },
            0xC000 => {
                let random_number:u8 = self.rng.gen();
                self.write_vx(random_number & (self.op_code & 0x00FF) as u8);
                self.increase_program_counter();
            },
            0xD000 => { 
                self.draw(self.read_vx(), self.read_vy(), (self.op_code & 0x000F) as u8);
                self.increase_program_counter();
            },
            0xE000 => {
                match self.op_code & 0x00FF {
                    0x009E => {
                        let key = self.read_vx() as usize;
                        self.increase_program_counter_if(self.keys[key]);
                        self.increase_program_counter();
                    },
                    0x00A1 => {
                        let key = self.read_vx() as usize;
                        self.increase_program_counter_if(self.keys[key]);
                        self.increase_program_counter();
                    },
                    _ => println!("Unkonwn op code")
                }
            },
            0xF000 => {
                match self.op_code & 0x00FF {
                    0x0007 => {
                        self.write_vx(self.delay_timer);
                    },
                    0x000A => {
                        // Lock until key press
                    },
                    0x0015 => {
                        self.delay_timer = self.read_vx();
                    },
                    0x0018 => {
                        self.sound_timer = self.read_vx();
                    },
                    0x001E => {
                        let (result, carry) = self.memory_index.overflowing_add(self.read_vx() as u16);
                        self.memory_index = result;
                        self.register[0xF] = if carry { 1 } else { 0 };
                    },
                    0x0029 => {
                    },
                    0x0033 => {
                    },
                    0x0055 => self.register_dump(self.read_vx()),
                    0x0065 => self.register_load(self.read_vx()),
                    _ => {},
                };
                self.increase_program_counter();
            },
            _ => println!("Haha!")
        }
        // Execute Opcode
        
        // Update timers
    }

    pub fn draw_flag(&mut self) -> bool {
        self.draw_flag
    }
}

pub fn init() -> Chip8 {
    println!("Rusty Chip8 initialized!");
    let mut init_memory = [0; 4096];
    let fontset =  [
        0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
        0x20, 0x60, 0x20, 0x20, 0x70, // 1
        0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
        0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
        0x90, 0x90, 0xF0, 0x10, 0x10, // 4
        0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
        0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
        0xF0, 0x10, 0x20, 0x40, 0x40, // 7
        0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
        0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
        0xF0, 0x90, 0xF0, 0x90, 0x90, // A
        0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
        0xF0, 0x80, 0x80, 0x80, 0xF0, // C
        0xE0, 0x90, 0x90, 0x90, 0xE0, // D
        0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
        0xF0, 0x80, 0xF0, 0x80, 0x80  // F
    ];
    init_memory[..80].clone_from_slice(&fontset);
    return Chip8 {
        op_code : 0,
        memory : init_memory,
        register : [0; 16],
        memory_index : 0,
        program_counter : 0x200,
        gfx: [false; 2048],
        delay_timer: 0,
        sound_timer: 0,
        stack: Vec::new(),
        keys: [false; 16],
        draw_flag: false,
        rng: rand::thread_rng(),
    };
}

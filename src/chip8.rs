use crate::errors::EmulationError;
use crate::errors::EmulationError::UnknownOpcode;
use rand::Rng;
use std::collections::VecDeque;
use std::convert::TryFrom;

///
/// Initial Fonts provided by the Chip8
///
const FONTS_SET: [u8; 80] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub(crate) struct Chip8 {
    pub op_code: u16,
    // also named PC
    // This is where to read the op code in memory
    pub program_counter: u16,
    pub memory: [u8; 4096],
    // also named V
    pub register: [u8; 16],
    pub memory_index: u16, // also named I
    // 64x32 pixel
    pub gfx: [bool; 2048],
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub stack: Vec<u16>,
    pub rng: rand::prelude::ThreadRng,
    pub debug_enabled: bool,
    pub debug_log: VecDeque<String>,
}

impl Default for Chip8 {
    fn default() -> Self {
        println!("Rusty Chip8 initialized!");
        let mut init_memory = [0; 4096];
        init_memory[..80].clone_from_slice(&FONTS_SET);
        Self {
            op_code: 0,
            memory: init_memory,
            register: [0; 16],
            memory_index: 0,
            program_counter: 0x200,
            gfx: [false; 2048],
            delay_timer: 0,
            sound_timer: 0,
            stack: Vec::new(),
            rng: rand::thread_rng(),
            debug_enabled: false,
            debug_log: VecDeque::new(),
        }
    }
}

const DEBUG_LOG_BUFFER_SIZE: usize = 50;

impl Chip8 {
    fn log(&mut self, log: String) {
        if self.debug_enabled {
            if self.debug_log.len() > DEBUG_LOG_BUFFER_SIZE {
                self.debug_log.pop_front();
            }
            self.debug_log.push_back(log)
        }
    }
    fn log_str(&mut self, log: &str) {
        if self.debug_enabled {
            if self.debug_log.len() > DEBUG_LOG_BUFFER_SIZE {
                self.debug_log.pop_front();
            }
            self.debug_log.push_back(log.to_string())
        }
    }

    pub fn load(&mut self, bytes: Vec<u8>) {
        let mut i = 512;
        for v in &bytes {
            self.memory[i] = *v;
            i = i + 1;
        }
        self.log_str("Rom Loaded into memory");
    }

    fn read_op_code(&self) -> u16 {
        (self.memory[self.program_counter as usize] as u16) << 8
            | self.memory[(self.program_counter + 1) as usize] as u16
    }

    // Register X is always located at the same position in opcode.
    fn read_vx(&self) -> u8 {
        self.register[((self.op_code & 0x0F00) >> 8) as usize]
    }

    // Write at vx location.
    fn write_vx(&mut self, value: u8) {
        let register_index = ((self.op_code & 0x0F00) >> 8) as usize;
        self.register[register_index] = value;
    }

    // Register Y if available is always located at the same position in opcode.
    fn read_vy(&self) -> u8 {
        self.register[((self.op_code & 0x00F0) >> 4) as usize]
    }

    fn write_vf(&mut self, value: u8) {
        self.register[0x0F] = value
    }

    fn set_program_counter(&mut self, index: u16) {
        self.program_counter = index;
    }

    fn increase_program_counter_if(&mut self, condition: bool) {
        if condition {
            self.increase_program_counter()
        }
    }

    fn increase_program_counter(&mut self) {
        self.program_counter += 2;
    }

    fn call_at(&mut self, address: u16) {
        self.stack.push(self.program_counter);
        self.program_counter = address;
    }

    fn draw(&mut self, x: u8, y: u8, height: u8) {
        self.write_vf(0);
        for y_row in 0..height {
            let sprite = self.memory[(self.memory_index + y_row as u16) as usize];
            for x_col in 0..8 {
                if (sprite & (0x80 >> x_col)) > 0 {
                    let gfx_loc: usize =
                        (x as usize + x_col as usize + (y as usize + y_row as usize) * 64) % 2048;
                    if self.gfx[gfx_loc] == true {
                        self.write_vf(1)
                    }
                    self.gfx[gfx_loc] ^= true
                }
            }
        }
    }

    fn register_dump(&mut self, reg_max: u8) {
        for reg_index in 0..reg_max {
            self.memory[self.memory_index as usize + reg_index as usize] =
                self.register[reg_index as usize];
        }
        self.memory[self.memory_index as usize + reg_max as usize] =
            self.register[reg_max as usize];
    }

    fn register_load(&mut self, reg_max: u8) {
        for reg_index in 0..reg_max as usize {
            self.register[reg_index] = self.memory[self.memory_index as usize + reg_index];
        }
        self.register[reg_max as usize] =
            self.memory[self.memory_index as usize + reg_max as usize];
    }

    pub fn cycle(&mut self, keypad: [bool; 16]) -> Result<u16, EmulationError> {
        // Fetch Opcode
        self.op_code = self.read_op_code();
        // Decode Opcode
        // Op code list : https://en.wikipedia.org/wiki/CHIP-8#Opcode_table
        let mut log = format!("Reading opcode [{:#06X}]", self.op_code);
        match self.op_code & 0xF000 {
            0x0000 => match self.op_code {
                0x00E0 => {
                    self.gfx.fill(false);
                    self.increase_program_counter();
                    log.push_str("Clear screen")
                }
                0x00EE => {
                    self.program_counter = self.stack.pop().unwrap();
                    self.increase_program_counter();
                    log.push_str(&format!(
                        "set program counter from stack back to {:#06X}",
                        self.program_counter
                    ))
                }
                _ => return Err(UnknownOpcode(self.op_code)),
            },
            0x1000 => {
                self.set_program_counter(self.op_code & 0x0FFF);
                log.push_str(&format!(
                    "set program counter to {:#06X}",
                    self.program_counter
                ))
            }
            0x2000 => {
                self.call_at(self.op_code & 0x0FFF);
                log.push_str("call instruction")
            }
            0x3000 => {
                self.increase_program_counter_if(self.read_vx() == (self.op_code & 0x00FF) as u8);
                self.increase_program_counter();
                log.push_str("increase pc (match vx)")
            }
            0x4000 => {
                self.increase_program_counter_if(self.read_vx() != (self.op_code & 0x00FF) as u8);
                self.increase_program_counter();
                log.push_str("increase pc (not match vx)")
            }
            0x5000 => {
                self.increase_program_counter_if(self.read_vx() == self.read_vy());
                self.increase_program_counter();
                log.push_str("increase pc (vx == vy)")
            }
            0x6000 => {
                self.write_vx((self.op_code & 0x00FF) as u8);
                self.increase_program_counter();
                log.push_str("write vx")
            }
            0x7000 => {
                let (result, _) = self
                    .read_vx()
                    .overflowing_add((self.op_code & 0x00FF) as u8);
                self.write_vx(result);
                self.increase_program_counter();
                log.push_str("add into write vx (no carry flag)")
            }
            0x8000 => {
                match self.op_code & 0x000F {
                    0x0000 => {
                        self.write_vx(self.read_vy());
                        log.push_str("move vy into vx")
                    }
                    0x0001 => {
                        self.write_vx(self.read_vx() | self.read_vy());
                        log.push_str("vx = vx or vy")
                    }
                    0x0002 => {
                        self.write_vx(self.read_vx() & self.read_vy());
                        log.push_str("vx = vx and vy")
                    }
                    0x0003 => {
                        self.write_vx(self.read_vx() ^ self.read_vy());
                        log.push_str("vx = vx xor vy")
                    }
                    0x0004 => {
                        let (result, carry) = self.read_vx().overflowing_add(self.read_vy());
                        self.write_vx(result);
                        self.write_vf(if carry { 1 } else { 0 });
                        log.push_str("vx = vx + vy (with carry flag)")
                    }
                    0x0005 => {
                        let (result, carry) = self.read_vx().overflowing_sub(self.read_vy());
                        self.write_vx(result);
                        self.write_vf(if carry { 1 } else { 0 });
                        log.push_str("vx = vx - vy (with carry)")
                    }
                    // TODO(switch implementation for original chip8, see https://www.reddit.com/r/EmuDev/comments/72dunw/chip8_8xy6_help/)
                    0x0006 => {
                        self.write_vf(self.read_vx() & 0x01);
                        self.write_vx(self.read_vx() >> 1);
                        log.push_str("vx = vx >> 1")
                    }
                    0x0007 => {
                        let (result, carry) = self.read_vy().overflowing_sub(self.read_vx());
                        self.write_vx(result);
                        self.write_vf(if carry { 1 } else { 0 });
                        log.push_str("vx = vy - vx (with carry)")
                    }
                    // TODO(switch implementation for original chip8, see https://www.reddit.com/r/EmuDev/comments/72dunw/chip8_8xy6_help/)
                    0x000E => {
                        self.write_vf(self.read_vx() & 0x80);
                        self.write_vx(self.read_vx() << 1);
                        log.push_str("vx = vx << 1")
                    }
                    _ => return Err(UnknownOpcode(self.op_code)),
                };
                self.increase_program_counter();
            }
            0x9000 => {
                self.increase_program_counter_if(self.read_vx() != self.read_vy());
                self.increase_program_counter();
                log.push_str("increase pc vx != vy")
            }
            0xA000 => {
                self.memory_index = self.op_code & 0x0FFF;
                self.increase_program_counter();
                log.push_str("write memory")
            }
            0xB000 => {
                let v0: u16 = self.register[0] as u16;
                self.set_program_counter((self.op_code & 0x0FFF) + v0);
                log.push_str(&format!("jump by {}", v0))
            }
            0xC000 => {
                let random_number: u8 = self.rng.gen();
                self.write_vx(random_number & (self.op_code & 0x00FF) as u8);
                self.increase_program_counter();
                log.push_str("randomize vx")
            }
            0xD000 => {
                self.draw(
                    self.read_vx(),
                    self.read_vy(),
                    (self.op_code & 0x000F) as u8,
                );
                self.increase_program_counter();
                log.push_str("draw")
            }
            0xE000 => match self.op_code & 0x00FF {
                0x009E => {
                    let key = self.read_vx() as usize;
                    self.increase_program_counter_if(keypad[key]);
                    self.increase_program_counter();
                    log.push_str("skip if key pressed in vx")
                }
                0x00A1 => {
                    let key = self.read_vx() as usize;
                    self.increase_program_counter_if(!keypad[key]);
                    self.increase_program_counter();
                    log.push_str("skip if key pressed in not vx")
                }
                _ => return Err(UnknownOpcode(self.op_code)),
            },
            0xF000 => {
                match self.op_code & 0x00FF {
                    0x0007 => {
                        self.write_vx(self.delay_timer);
                        log.push_str("vx = delay timer");
                    }
                    0x000A => {
                        // Increase counter only if key press
                        if keypad.iter().any(|&key| key) {
                            self.increase_program_counter();
                            log.push_str("key pressed read, continuing")
                        }
                        log.push_str("wait for key press");
                    }
                    0x0015 => {
                        self.delay_timer = self.read_vx();
                        log.push_str("set delay timer");
                    }
                    0x0018 => {
                        self.sound_timer = self.read_vx();
                        log.push_str("set sound timer");
                    }
                    0x001E => {
                        let (result, _) = self.memory_index.overflowing_add(self.read_vx() as u16);
                        self.memory_index = result & 0x0FFF; // u12, not u16
                                                             // TODO handle overflowing.
                        log.push_str("i = i + vx")
                    }
                    0x0029 => {
                        self.memory_index = match self.read_vx() {
                            0x0 => 0,
                            0x1 => 5,
                            0x2 => 10,
                            0x3 => 15,
                            0x4 => 20,
                            0x5 => 25,
                            0x6 => 30,
                            0x7 => 35,
                            0x8 => 40,
                            _ => return Err(UnknownOpcode(self.op_code)),
                        };
                        log.push_str(&format!("i = sprite_addr[{:#06X}]", self.read_vx()))
                    }
                    0x0033 => {
                        let mut value = self.read_vx();
                        let hundreds = value / 100;
                        value %= 100;
                        let tens = value / 10;
                        let unit = value % 10;
                        self.memory[self.memory_index as usize] = hundreds;
                        self.memory[self.memory_index as usize + 1] = tens;
                        self.memory[self.memory_index as usize + 2] = unit;
                        log.push_str(&format!(
                            "vx {:#06X} as decimal number [{} {} {}]",
                            self.read_vx(),
                            hundreds,
                            tens,
                            unit
                        ))
                    }
                    0x0055 => {
                        let register_index = u8::try_from((self.op_code & 0x0F00) >> 8).unwrap();
                        self.register_dump(register_index);
                        log.push_str("dump vy")
                    }
                    0x0065 => {
                        let register_index = u8::try_from((self.op_code & 0x0F00) >> 8).unwrap();
                        self.register_load(register_index);
                        log.push_str("load vx")
                    }
                    _ => return Err(UnknownOpcode(self.op_code)),
                };
                self.increase_program_counter();
            }
            _ => return Err(UnknownOpcode(self.op_code)),
        };

        self.log(log);

        // Update timers
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }

        Ok(self.op_code)
    }
}

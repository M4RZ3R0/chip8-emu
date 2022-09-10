use rand::random;
use std::collections::VecDeque;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const START_ADDR: u16 = 0x200;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;

const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
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

pub struct Machine {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    stack: VecDeque<u16>,
    keys: [bool; NUM_KEYS],
    dt: u8,
    st: u8,
}

impl Default for Machine {
    fn default() -> Self {
        Self::new()
    }
}

impl Machine {
    pub fn new() -> Self {
        let mut new_machine = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            stack: VecDeque::with_capacity(STACK_SIZE),
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };

        new_machine.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_machine
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = START_ADDR as usize + data.len();
        self.ram[start..end].copy_from_slice(data);
    }

    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.stack = VecDeque::with_capacity(STACK_SIZE);
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
    }

    pub fn push(&mut self, val: u16) {
        self.stack.push_back(val);
    }

    pub fn pop(&mut self) -> u16 {
        if let Some(i) = self.stack.pop_back() {
            i
        } else {
            println!("Error: stack is empty");
            std::process::exit(5);
        }
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                // BEEP
            }
            self.st -= 1;
        }
    }

    pub fn keypress(&mut self, index: usize, pressed: bool) {
        self.keys[index] = pressed;
    }

    pub fn tick(&mut self) {
        // Fetch
        let op = self.fetch();
        // Decode & execute
        self.execute(op);
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        self.pc += 2;

        (higher_byte << 8) | lower_byte
    }

    fn execute(&mut self, op: u16) {
        let byte1 = (op & 0xF000) >> 12;
        let byte2 = (op & 0x0F00) >> 8;
        let byte3 = (op & 0x00F0) >> 4;
        let byte4 = op & 0x000F;

        match (byte1, byte2, byte3, byte4) {
            (0, 0, 0xE, 0) => self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            (0, 0, 0xE, 0xE) => self.pc = self.pop(),
            (1, _, _, _) => self.pc = op & 0xFFF,
            (2, _, _, _) => {
                self.push(self.pc);
                self.pc = op & 0xFFF
            }
            (3, _, _, _) => {
                if self.v_reg[byte2 as usize] == (op & 0xFF) as u8 {
                    self.pc += 2
                }
            }
            (4, _, _, _) => {
                if self.v_reg[byte2 as usize] != (op & 0xFF) as u8 {
                    self.pc += 2
                }
            }
            (5, _, _, 0) => {
                if self.v_reg[byte2 as usize] == self.v_reg[byte3 as usize] {
                    self.pc += 2
                }
            }
            (6, _, _, _) => self.v_reg[byte2 as usize] = (op & 0xFF) as u8,
            (7, _, _, _) => {
                self.v_reg[byte2 as usize] =
                    self.v_reg[byte2 as usize].wrapping_add((op & 0xFF) as u8)
            }
            (8, _, _, 0) => self.v_reg[byte2 as usize] = self.v_reg[byte3 as usize],
            (8, _, _, 1) => self.v_reg[byte2 as usize] |= self.v_reg[byte3 as usize],
            (8, _, _, 2) => self.v_reg[byte2 as usize] &= self.v_reg[byte3 as usize],
            (8, _, _, 3) => self.v_reg[byte2 as usize] ^= self.v_reg[byte3 as usize],
            (8, _, _, 4) => {
                let carry;
                (self.v_reg[byte2 as usize], carry) =
                    self.v_reg[byte2 as usize].overflowing_add(self.v_reg[byte3 as usize]);

                self.v_reg[0xF] = if carry { 1 } else { 0 };
            }
            (8, _, _, 5) => {
                let borrow;
                (self.v_reg[byte2 as usize], borrow) =
                    self.v_reg[byte2 as usize].overflowing_sub(self.v_reg[byte3 as usize]);

                self.v_reg[0xF] = if borrow { 0 } else { 1 };
            }
            (8, _, _, 6) => {
                self.v_reg[0xF] = self.v_reg[byte2 as usize] & 1;
                self.v_reg[byte2 as usize] >>= 1;
            }
            (8, _, _, 7) => {
                let borrow;
                (self.v_reg[byte2 as usize], borrow) =
                    self.v_reg[byte3 as usize].overflowing_sub(self.v_reg[byte2 as usize]);

                self.v_reg[0xF] = if borrow { 1 } else { 0 };
            }
            (8, _, _, 0xE) => {
                self.v_reg[0xF] = (self.v_reg[byte2 as usize] >> 7) & 1;
                self.v_reg[byte2 as usize] <<= 1;
            }
            (9, _, _, 0) => {
                if self.v_reg[byte2 as usize] != self.v_reg[byte3 as usize] {
                    self.pc += 2
                }
            }
            (0xA, _, _, _) => self.i_reg = op & 0xFFF,
            (0xB, _, _, _) => self.pc = (self.v_reg[0] as u16) + (op & 0xFFF),
            (0xC, _, _, _) => self.v_reg[byte2 as usize] = random::<u8>() & (op & 0xFF) as u8,
            (0xD, _, _, _) => {
                let x_start = self.v_reg[byte2 as usize] as u16;
                let y_start = self.v_reg[byte3 as usize] as u16;

                let mut flipped = false;

                for j in 0..byte4 {
                    let addr = self.i_reg + j as u16;
                    let pixels = self.ram[addr as usize];

                    for i in 0..8 {
                        if (pixels & (0b1000_0000 >> i)) != 0 {
                            let x = (x_start + i) as usize % SCREEN_WIDTH;
                            let y = (y_start + j) as usize % SCREEN_HEIGHT;

                            let index = y * SCREEN_WIDTH + x;
                            flipped |= self.screen[index];
                            self.screen[index] ^= true;
                        }
                    }
                }

                self.v_reg[0xF] = if flipped { 1 } else { 0 };
            }
            (0xE, _, 9, 0xE) => {
                if self.keys[self.v_reg[byte2 as usize] as usize] {
                    self.pc += 2
                }
            }
            (0xE, _, 0xA, 1) => {
                if !self.keys[self.v_reg[byte2 as usize] as usize] {
                    self.pc += 2
                }
            }
            (0xF, _, 0, 7) => self.v_reg[byte2 as usize] = self.dt,
            (0xF, _, 0, 0xA) => {
                let mut pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_reg[byte2 as usize] = i as u8;
                        pressed = true;
                        break;
                    }
                }

                if !pressed {
                    self.pc -= 2;
                }
            }
            (0xF, _, 1, 5) => self.dt = self.v_reg[byte2 as usize],
            (0xF, _, 1, 8) => self.st = self.v_reg[byte2 as usize],
            (0xF, _, 1, 0xE) => {
                self.i_reg = self.i_reg.wrapping_add(self.v_reg[byte2 as usize] as u16)
            }
            (0xF, _, 2, 9) => self.i_reg = (self.v_reg[byte2 as usize] * 5) as u16,
            (0xF, _, 3, 3) => {
                let mut vx = self.v_reg[byte2 as usize];
                for i in 0..3 {
                    let tmp = vx % 10;
                    self.ram[(self.i_reg + (2 - i)) as usize] = tmp;
                    vx /= 10;
                }
            }
            (0xF, _, 5, 5) => {
                for index in 0..=byte2 {
                    self.ram[(self.i_reg + index) as usize] = self.v_reg[index as usize];
                }
            }
            (0xF, _, 6, 5) => {
                for index in 0..=byte2 {
                    self.v_reg[index as usize] = self.ram[(self.i_reg + index) as usize];
                }
            }
            (_, _, _, _) => (),
        };
    }
}

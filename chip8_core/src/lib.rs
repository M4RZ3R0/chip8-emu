use std::collections::VecDeque;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const START_ADDR: u16 = 0x200;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;

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

impl Machine {
    pub fn new() -> Self {
        Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            stack: VecDeque::with_capacity(STACK_SIZE),
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        }
    }
}

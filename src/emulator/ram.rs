const N_INSTRUCTIONS: usize = 2048 * 2; // Some games want more than original
const N_BYTES: usize = 2 * N_INSTRUCTIONS;

pub const FONT_POS: usize = 0x50;
pub const PROG_MEM_START: usize = 0x200;

pub struct Ram {
    mem: [u8; N_BYTES], // Handling 16 bit uints ensures alignment for most cases
    iterator: usize,
}

impl Ram {
    pub fn fetch(&mut self) -> u16 {
        return ((self.mem[self.iterator] as u16) << 8) | self.mem[self.iterator + 1] as u16;
    }
    pub fn set_short(&mut self, val: u16) -> () {
        self.mem[self.iterator] = (val >> 8) as u8;
        self.mem[self.iterator + 1] = val as u8;
    }
    pub fn set_byte(&mut self, val: u8) -> () {
        self.mem[self.iterator] = val;
    }
    pub fn increment(&mut self) -> () {
        self.iterator += 2;
    }
    pub fn decrement(&mut self) -> () {
        self.iterator -= 2;
    }
    pub fn jump_to(&mut self, address: usize) -> () {
        self.iterator = address;
    }
    pub fn address(&self) -> usize {
        return self.iterator;
    }
    pub fn get(&mut self, address: usize) -> u8 {
        return self.mem[address] as u8;
    }
    pub fn get_pgrm_mem(&mut self) -> &mut [u8] {
        return &mut self.mem[PROG_MEM_START..N_INSTRUCTIONS];
    }
}
const FONT: [u8; 80] = [
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

pub fn create_ram() -> Ram {
    let mut ram = Ram {
        mem: [0; N_BYTES],
        iterator: FONT_POS,
    };
    for i in 0..FONT.len() {
        ram.jump_to(FONT_POS + i);
        ram.set_byte(FONT[i]);
    }
    ram.jump_to(PROG_MEM_START);
    return ram;
}

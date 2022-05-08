mod ascii;
mod instruction;
use atomic_counter::AtomicCounter;
use instruction::Instruction;
use log::{debug, error, trace};
mod keyboard;
mod ram;
mod stack;
mod timer;
use rand;

const CLOCK_INTERVAL_US: u64 = 1000;

pub struct Emulator {
    display: ascii::Display,
    stack: stack::Stack,
    ram: ram::Ram,
    sound_timer: timer::ThreadedCounter,
    delay_timer: timer::ThreadedCounter,
    keyboard: keyboard::Keyboard,
}

impl Emulator {
    fn fetch(&mut self) -> u16 {
        let instr = self.ram.fetch();
        self.ram.increment();
        return instr;
    }

    fn clear_cmd(&mut self, inst: u16) -> bool {
        trace!("clear {:#0x}", inst);
        if inst.jump_addr() == 0x0E0 {
            self.display.clear();
        }
        if inst.jump_addr() == 0x0EE {
            return false;
        }
        return true;
    }

    fn display_cmd(&mut self, inst: u16) {
        trace!("Display cmd! {:#0x}", inst);
        let vx = (self.stack.v[inst.x_register_of() as usize] % (ascii::WIDTH as u8)) as usize;
        let vy = (self.stack.v[inst.y_register_of() as usize] % (ascii::HEIGHT as u8)) as usize;
        let n = std::cmp::min(
            inst.fourth_nibble_of() as usize,
            ascii::HEIGHT as usize - vy,
        );
        let mut zeroed = false;

        let mut i_pos = self.stack.i as usize;

        for i in 0..n {
            for j in 0..std::cmp::min(8, ascii::WIDTH - vx) {
                let i_val = self.ram.get(i_pos);
                let toggle_val = ((i_val >> (7 - j)) & 1) != 0;
                zeroed |= self.display.xor(toggle_val, vy + i, vx + j);
            }
            i_pos += 1;
        }
        self.stack.v[0xF] = zeroed as u8;
        self.display.draw();
    }

    fn jump_cmd(&mut self, inst: u16) {
        trace!("Jump cmd {:#0x}", inst);
        self.ram.jump_to(inst.jump_addr() as usize);
    }

    fn register_set(&mut self, inst: u16) {
        trace!("register_set {:#0x}", inst);
        self.stack.v[inst.x_register_of() as usize] = inst.second_byte_of();
    }

    fn register_add(&mut self, inst: u16) {
        trace!("register_add {:#0x}", inst);
        let reg = inst.x_register_of() as usize;
        self.stack.v[reg] =
            (((self.stack.v[reg] as u16) + (inst.second_byte_of() as u16)) & 0xFF) as u8;
    }

    fn index_set(&mut self, inst: u16) {
        trace!("index_set {:#0x}", inst);
        self.stack.i = inst.jump_addr();
    }

    fn subroutine(&mut self, inst: u16) {
        trace!("subroutine {:#0x}", inst);
        let pos = self.ram.address();
        self.ram.jump_to(inst.jump_addr() as usize);
        self.start_loop();
        self.ram.jump_to(pos);
    }

    fn logic_cmd(&mut self, inst: u16) {
        match inst.fourth_nibble_of() {
            0 => {
                self.stack.v[inst.x_register_of() as usize] =
                    self.stack.v[inst.y_register_of() as usize];
            }
            1 => {
                self.stack.v[inst.x_register_of() as usize] |=
                    self.stack.v[inst.y_register_of() as usize];
            }
            2 => {
                self.stack.v[inst.x_register_of() as usize] &=
                    self.stack.v[inst.y_register_of() as usize];
            }
            3 => {
                self.stack.v[inst.x_register_of() as usize] ^=
                    self.stack.v[inst.y_register_of() as usize];
            }
            4 => {
                let sum = self.stack.v[inst.x_register_of() as usize] as u16
                    + self.stack.v[inst.y_register_of() as usize] as u16;
                self.stack.v[0xF] = if sum > 255 { 1 } else { 0 };
                self.stack.v[inst.x_register_of() as usize] = (sum & 0xFF) as u8;
            }
            5 => {
                let diff = self.stack.v[inst.x_register_of() as usize] as i16
                    - self.stack.v[inst.y_register_of() as usize] as i16;
                self.stack.v[0xF] = if diff > 0 { 1 } else { 0 };
                self.stack.v[inst.x_register_of() as usize] = (diff & 0xFF) as u8;
            }
            7 => {
                let diff = self.stack.v[inst.y_register_of() as usize] as i16
                    - self.stack.v[inst.x_register_of() as usize] as i16;
                self.stack.v[0xF] = if diff > 0 { 1 } else { 0 };
                self.stack.v[inst.x_register_of() as usize] = (diff & 0xFF) as u8;
            }
            6 => {
                // Old
                //self.stack.v[inst.x_register_of() as usize] =
                //    self.stack.v[inst.y_register_of() as usize] >> 1;
                //self.stack.v[0xF] = self.stack.v[inst.y_register_of() as usize] & 1;
                // New
                self.stack.v[0xF] = self.stack.v[inst.x_register_of() as usize] & 1;
                self.stack.v[inst.x_register_of() as usize] =
                    self.stack.v[inst.x_register_of() as usize] >> 1;
                debug!("Warning: Ambiguous instruction {:#0x}", inst);
            }
            0xE => {
                // Old
                // self.stack.v[inst.x_register_of() as usize] =
                //     self.stack.v[inst.y_register_of() as usize] << 1;
                // self.stack.v[0xF] = self.stack.v[inst.y_register_of() as usize] >> 7;
                // New
                self.stack.v[0xF] = self.stack.v[inst.x_register_of() as usize] >> 7;
                self.stack.v[inst.x_register_of() as usize] =
                    self.stack.v[inst.x_register_of() as usize] << 1;
                debug!("Warning: Ambiguous instruction {:#0x}", inst);
            }
            _ => {
                error!("Bad logic_cmd {:#0x}", inst);
            }
        }
    }

    fn jump_offset_cmd(&mut self, inst: u16) {
        debug!("jump_offset_cmd has ambiguous definitions {:#0x}", inst);
        self.ram
            .jump_to(inst.jump_addr() as usize + self.stack.v[0] as usize);
    }

    fn random(&mut self, inst: u16) {
        self.stack.v[inst.x_register_of() as usize] = rand::random::<u8>() & inst.second_byte_of();
    }

    fn skip_cmd(&mut self, inst: u16) {
        match inst.instruction_of() {
            0x3 => {
                if (self.stack.v[inst.x_register_of() as usize]) == inst.second_byte_of() {
                    self.ram.increment();
                }
            }
            0x4 => {
                if (self.stack.v[inst.x_register_of() as usize]) != inst.second_byte_of() {
                    self.ram.increment();
                }
            }
            0x5 => {
                if (self.stack.v[inst.x_register_of() as usize])
                    == self.stack.v[inst.y_register_of() as usize]
                {
                    self.ram.increment();
                }
            }
            0x9 => {
                if (self.stack.v[inst.x_register_of() as usize])
                    != self.stack.v[inst.y_register_of() as usize]
                {
                    self.ram.increment();
                }
            }
            _ => {
                error!("Invalid skip");
                std::process::exit(1);
            }
        }
    }

    fn skip_if_cmd(&mut self, inst: u16) {
        match inst.second_byte_of() {
            0x9E => {
                if self
                    .keyboard
                    .is_pressed(self.stack.v[inst.x_register_of() as usize] as i32)
                {
                    self.ram.increment();
                }
            }
            0xA1 => {
                if !self
                    .keyboard
                    .is_pressed(self.stack.v[inst.x_register_of() as usize] as i32)
                {
                    self.ram.increment();
                }
            }
            _ => {
                error!("Invalid skip_if");
                std::process::exit(1);
            }
        }
    }

    fn wait_keypress(&mut self, inst: u16) {
        let val = self.keyboard.get_press();
        if val == -1 {
            self.ram.decrement();
            return;
        }
        self.stack.v[inst.x_register_of() as usize] = val as u8;
    }

    fn bloated_cmd(&mut self, inst: u16) {
        match inst.second_byte_of() {
            0x07 => {
                let val: u8 = 255 - (*self.delay_timer.counter).get() as u8;
                self.stack.v[inst.x_register_of() as usize] = val;
            }
            0x15 => {
                let val = 255 - self.stack.v[inst.x_register_of() as usize] as usize;
                (*self.delay_timer.counter).reset();
                // We are simply hoping the timer will not have time to increment :P
                // Locks would be required for better thread handling
                (*self.delay_timer.counter).add(val);
            }
            0x18 => {
                let val = 255 - self.stack.v[inst.x_register_of() as usize] as usize;
                (*self.sound_timer.counter).reset();
                // We are simply hoping the timer will not have time to increment :P
                // Locks would be required for better thread handling
                (*self.sound_timer.counter).add(val);
            }
            0x1E => {
                let sum = self.stack.i + self.stack.v[inst.x_register_of() as usize] as u16;
                // Instructions unclear; should the bit only be set on overflow? Or never at all?
                self.stack.v[0xF] = (sum > 0xFFF) as u8;
                self.stack.i = sum & 0xFFF;
            }
            0x0A => {
                self.wait_keypress(inst);
            }
            0x29 => {
                // Whats the encoding here? Is 0 the first letter?
                self.stack.i =
                    (ram::FONT_POS as u16) + self.stack.v[inst.x_register_of() as usize] as u16;
            }
            0x33 => {
                let mut val = self.stack.v[inst.x_register_of() as usize];
                let addr = self.ram.address();
                for i in 0..3 {
                    self.ram.jump_to(self.stack.i as usize + (2 - i) as usize);
                    self.ram.set_byte(val % 10);
                    val /= 10;
                }
                self.ram.jump_to(addr);
            }
            0x55 => {
                let addr = self.ram.address();
                for i in 0..(inst.x_register_of() + 1) {
                    self.ram.jump_to(self.stack.i as usize + i as usize);
                    self.ram.set_byte(self.stack.v[i as usize]);
                }
                self.ram.jump_to(addr); // New behaviour: jump back;
            }
            0x65 => {
                for i in 0..(inst.x_register_of() + 1) {
                    self.stack.v[i as usize] = self.ram.get(self.stack.i as usize + i as usize);
                }
            }
            _ => {
                error!("Invalid f cmd");
                std::process::exit(1);
            }
        }
    }

    fn execute(&mut self, inst: u16) -> bool {
        match inst.instruction_of() {
            0x0 => return self.clear_cmd(inst),
            0x1 => self.jump_cmd(inst),
            0x2 => self.subroutine(inst),
            0x3 | 0x4 | 0x5 | 0x9 => self.skip_cmd(inst),
            0x6 => self.register_set(inst),
            0x7 => self.register_add(inst),
            0x8 => self.logic_cmd(inst),
            0xA => self.index_set(inst),
            0xB => self.jump_offset_cmd(inst),
            0xC => self.random(inst),
            0xD => self.display_cmd(inst),
            0xE => self.skip_if_cmd(inst),
            0xF => self.bloated_cmd(inst),
            _ => {
                error!("Invalid instruction {:#0x}", inst);
                std::process::exit(1);
            }
        }
        return true;
    }

    fn loop_run(&mut self) -> bool {
        let val = self.fetch();
        return self.execute(val);
    }

    fn start_loop(&mut self) {
        let mut timer = timer::Timer {
            interval: std::time::Duration::from_micros(CLOCK_INTERVAL_US),
            action: || self.loop_run(),
        };
        timer.run();
    }

    pub fn get_pgrm_mem(&mut self) -> &mut [u8] {
        return self.ram.get_pgrm_mem();
    }

    pub fn run(&mut self) {
        self.display.draw();
        self.start_loop();
        // This will stop the threads
        (*self.sound_timer.counter).add(256);
        (*self.delay_timer.counter).add(256);
    }
}

pub fn create_emulator() -> Emulator {
    return Emulator {
        display: ascii::create_display(),
        stack: stack::Stack { i: 0, v: [0; 16] },
        ram: ram::create_ram(),
        sound_timer: timer::create_threaded_counter(std::time::Duration::from_micros(16667)), // 60 hz
        delay_timer: timer::create_threaded_counter(std::time::Duration::from_micros(16667)),
        keyboard: keyboard::create(),
    };
}

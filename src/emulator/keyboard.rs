use log::trace;
use device_query::{DeviceQuery, DeviceState, Keycode};

pub struct Keyboard {
    keymap: [i32; 100],
    state: DeviceState,
}

pub fn create() -> Keyboard {
    let mut kb = Keyboard {
        keymap: [-1; 100],
        state: DeviceState::new(),
    };
    kb.keymap[Keycode::Key1 as usize] = 1;
    kb.keymap[Keycode::Key2 as usize] = 2;
    kb.keymap[Keycode::Key3 as usize] = 3;
    kb.keymap[Keycode::Key4 as usize] = 0xC;
    kb.keymap[Keycode::Q as usize] = 0x4;
    kb.keymap[Keycode::W as usize] = 0x5;
    kb.keymap[Keycode::E as usize] = 0x6;
    kb.keymap[Keycode::R as usize] = 0xD;
    kb.keymap[Keycode::A as usize] = 0x7;
    kb.keymap[Keycode::S as usize] = 0x8;
    kb.keymap[Keycode::D as usize] = 0x9;
    kb.keymap[Keycode::F as usize] = 0xE;
    kb.keymap[Keycode::Z as usize] = 0xA;
    kb.keymap[Keycode::X as usize] = 0x0;
    kb.keymap[Keycode::C as usize] = 0xB;
    kb.keymap[Keycode::V as usize] = 0xF;
    return kb;
}


impl Keyboard {
    // Returns -1 if nothing is pressed
    pub fn get_press(&mut self) -> i32 {
        let keys = self.state.get_keys();
        trace!("Keys:");
        for key in keys.iter() {
            trace!("{}", key.clone() as usize);
            if self.keymap[key.clone() as usize] != -1 {
                return self.keymap[key.clone() as usize];
            }
        }
        return -1;
    }
    pub fn is_pressed(&mut self, but: i32) -> bool {
        trace!("IsPressed {}", but);
        for key in self.state.get_keys().iter() {
            if self.keymap[key.clone() as usize] == but {

                return true;
            }
        }
        return false;
    }
}

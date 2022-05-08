use log;

pub const HEIGHT: usize = 32;
pub const WIDTH: usize = 64;

pub struct Display {
    pub buffer: [[bool; WIDTH]; HEIGHT],
}

fn clear() {
    if log::log_enabled!(log::Level::Trace){
        return;
    }
    print!("\x1B[2J");
}

const OFF: &str = "⬛";
const ON: &str = "⬜";

pub fn create_display() -> Display {
    return Display {
        buffer: [[false; WIDTH]; HEIGHT],
    };
}

impl Display {
    // Returns true if pixel was set to 0
    pub fn xor(&mut self, val: bool, h: usize, w: usize) -> bool {
        if val {
            let prev_val = self.buffer[h][w];
            self.buffer[h][w] = !prev_val;
            return prev_val;
        }
        return false;
    }
    pub fn clear(&mut self) {
        for ele in self.buffer.iter_mut() {
            (*ele).fill(false);
        }
    }
    pub fn draw(&self) {
        clear();
        for i in 0..HEIGHT {
            for j in 0..WIDTH {
                print!("{}", if self.buffer[i][j] { ON } else { OFF });
            }
            println!();
        }
    }
}

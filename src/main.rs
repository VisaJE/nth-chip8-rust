mod emulator;
mod from_file;
use env_logger;

fn main() {
    env_logger::init();
    log::info!("Logging on");
    let mut emul = emulator::create_emulator();
    from_file::read("prog.ch8", emul.get_pgrm_mem());
    emul.run();
}

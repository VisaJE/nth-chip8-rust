pub trait Instruction {
    fn instruction_of(&self) -> u16;
    fn x_register_of(&self) -> u16;
    fn y_register_of(&self) -> u16;
    fn fourth_nibble_of(&self) -> u16;
    fn second_byte_of(&self) -> u8;
    fn jump_addr(&self) -> u16;
}
impl Instruction for u16 {
    fn instruction_of(&self) -> u16 {
        return self >> 12;
    }
    fn x_register_of(&self) -> u16 {
        return (self >> 8) & 0x0F;
    }

    fn y_register_of(&self) -> u16 {
        return self >> 4 & 0x0F;
    }

    fn fourth_nibble_of(&self) -> u16 {
        return self & 0x0F;
    }

    fn second_byte_of(&self) -> u8 {
        return (self & 0xFF) as u8;
    }
    fn jump_addr(&self) -> u16 {
        return self & 0xFFF;
    }
}

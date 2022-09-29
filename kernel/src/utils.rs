pub fn calculate_instruction_length(low: u8) -> usize {
    if low & 0b11 != 0b11 {
        16
    } else if low & 0b11100 != 0b11100 {
        32
    } else if low & 0b11_1111 == 0b01_1111 {
        48
    } else if low & 0b111_1111 == 0b011_1111 {
        64
    } else {
        panic!("not <=64 instruction");
    }
}

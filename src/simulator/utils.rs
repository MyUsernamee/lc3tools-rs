pub fn get_bit(value: u16, bit: u16) -> bool {
    (value >> bit) == 1
}

pub fn sign_extend(bits: u16, value: u16) -> u16 {
    let value = value & ((1 << bits) - 1);
    let sign_bit = value & (1 << bits - 1);
    if sign_bit != 0 {
        return (!(0b0u16) << bits) | value;
    }
    value
}


pub struct Catridge {
    program_memory: Box<u8>,
    character_memory: Box<u8>,

    program_banks: u8,
    character_banks: u8,
}

impl Catridge {
    pub fn new(filepath: &str) {}
}

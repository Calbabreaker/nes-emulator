use crate::mappers::*;

pub struct Catridge {
    mapper: Box<dyn Mapper>,
    prg_memory: Box<u8>,
    chr_memory: Box<u8>,
}

impl Catridge {
    pub fn new(data: &[u8]) {
        let mapper_id = data[7] & 0xf0 | data[6] >> 4;

        let mapper = match mapper_id {
            0 => Mapper0::new(),
        }

        // Catridge {
        //     header
        // }
    }
}

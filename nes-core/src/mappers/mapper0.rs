use super::{Mapper, MapperInfo};

pub struct Mapper0 {
    info: MapperInfo,
}

impl Mapper0 {
    pub fn new(info: MapperInfo) -> Self {
        Mapper0 { info }
    }
}

impl Mapper for Mapper0 {
    fn map_prg_read(&self, address: u16, address_out: &mut u16) -> bool {
        let prg_banks = self.info.prg_banks;
        match address {
            0x8000..=0xffff => {
                *address_out = address & if prg_banks == 1 { 0x3fff } else { 0x7fff };
                true
            }
            _ => false,
        }
    }

    fn map_prg_write(&self, address: u16, address_out: &mut u16) -> bool {
        self.map_prg_read(address, address_out)
    }

    fn map_chr_read(&self, address: u16, address_out: &mut u16) -> bool {
        match address {
            0x0000..=0x1fff => {
                *address_out = address;
                true
            }
            _ => false,
        }
    }

    fn map_chr_write(&self, address: u16, address_out: &mut u16) -> bool {
        false
    }
}

pub struct MapperInfo {
    pub prg_banks: u8,
    pub chr_banks: u8,
}

pub trait Mapper {
    fn map_prg_read(&self, address: u16, address_out: &mut u16) -> bool;
    fn map_prg_write(&self, address: u16, address_out: &mut u16) -> bool;
    fn map_chr_read(&self, address: u16, address_out: &mut u16) -> bool;
    fn map_chr_write(&self, address: u16, address_out: &mut u16) -> bool;
}

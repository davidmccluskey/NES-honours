
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Mirroring {
    Vertical,
    Horizontal,
    Hardware,
}
pub trait Mapper{
    fn cpu_mapper_read(&mut self, address: u16, mapped_address: &mut u32, data: &mut u8) -> bool;
    fn cpu_mapper_write(&mut self, address: u16, mapped_address: &mut u32, data: &mut u8) -> bool;
    fn ppu_mapper_read(&mut self, address: u16, mapped_address: &mut u32) -> bool;
    fn ppu_mapper_write(&mut self, address: u16, mapped_address: &mut u32) -> bool;
    fn reset(&mut self);
    fn mirror(&mut self) -> Mirroring;
}

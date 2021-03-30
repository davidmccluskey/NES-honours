
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Mirroring {
    Vertical,
    Horizontal,
    OnescreenLow,
    OnescreenHigh,
    Hardware,
}
pub trait Mapper{
    fn cpu_mapper_read(&mut self, addr: u16, mapped_addr: &mut u32) -> bool;
    fn cpu_mapper_write(&mut self, addr: u16, mapped_addr: &mut u32) -> bool;
    fn ppu_mapper_read(&mut self, addr: u16, mapped_addr: &mut u32) -> bool;
    fn ppu_mapper_write(&mut self, addr: u16, mapped_addr: &mut u32) -> bool;
    fn mirror(&mut self) -> Mirroring;
    fn reset(&mut self);
}
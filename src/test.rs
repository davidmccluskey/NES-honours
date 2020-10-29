use super::*;
use rand::Rng;
use CPU6502::Flags;

//https://github.com/leandromoreira/python_chip16/blob/master/tests/test_cpu.py#L132

#[test]
fn test_multiply(){
    let mut nes = CPU6502::CPU6502::new();
    let mut nOffset = 0x8000;
    let mut v: Vec<&str> = "A2 0A 8E 00 00 A2 03 8E 01 00 AC 00 00 A9 00 18 6D 01 00 88 D0 FA 8D 02 00 EA EA EA".rsplit(' ').collect();
    v.reverse();
    for c in v.iter() {
        if c.to_string() != " " {
            let z = u8::from_str_radix(c, 16).unwrap();
            nes.bus.ram[nOffset] = z;
        }
        nOffset = nOffset + 1;
    }

    nes.bus.ram[0xFFFC] = 0x00;
    nes.bus.ram[0xFFFD] = 0x80;
    nes.reset();

    for x in 0..40{
        while {
            nes.clock(); 
            !nes.complete()}{}
    }

    assert_eq!(nes.a, 30);
    assert_eq!(nes.bus.ram[2], 30);
    
}

#[test]
fn test_add(){
    let mut nes = CPU6502::CPU6502::new();
    let mut nOffset = 0x8000;
    let mut v: Vec<&str> = "A2 64 8E 00 00 A2 20 8E 01 00 A9 00 18 6D 00 00 6D 01 00 8D 02 00 EA EA EA".rsplit(' ').collect();
    v.reverse();
    for c in v.iter() {
        if c.to_string() != " " {
            let z = u8::from_str_radix(c, 16).unwrap();
            nes.bus.ram[nOffset] = z;
        }
        nOffset = nOffset + 1;
    }

    nes.bus.ram[0xFFFC] = 0x00;
    nes.bus.ram[0xFFFD] = 0x80;
    nes.reset();

    for x in 0..40{
        while {
            nes.clock(); 
            !nes.complete()}{}
    }

    assert_eq!(nes.a, 132);
    assert_eq!(nes.bus.ram[2], 132);
}

#[test]
fn test_subtract(){
    let mut nes = CPU6502::CPU6502::new();
    let mut nOffset = 0x8000;
    let mut v: Vec<&str> = "A9 64 A2 20 8E 01 00 18 ED 01 00 8D 02 00 EA EA EA".rsplit(' ').collect();
    v.reverse();
    for c in v.iter() {
        if c.to_string() != " " {
            let z = u8::from_str_radix(c, 16).unwrap();
            nes.bus.ram[nOffset] = z;
        }
        nOffset = nOffset + 1;
    }

    nes.bus.ram[0xFFFC] = 0x00;
    nes.bus.ram[0xFFFD] = 0x80;
    nes.reset();

    for x in 0..40{
        while {
            nes.clock(); 
            !nes.complete()}{}
    }

    assert_eq!(nes.a, 67);
    assert_eq!(nes.bus.ram[2], 67);
    
}


#[test]
fn test_getFlag_setFlag(){
    let mut nes = CPU6502::CPU6502::new();

    let mut a = nes.GetFlag(Flags::C);
    assert_eq!(a, 0);

    nes.SetFlag(Flags::C, true);
    a = nes.GetFlag(Flags::C);
    assert_eq!(a, 1);

    nes.SetFlag(Flags::C, false);
    a = nes.GetFlag(Flags::C);
    assert_eq!(a, 0);
}

#[test]
fn test_read_write(){
    let mut rng = rand::thread_rng();

    let n1: u8 = rng.gen();
    let n2: u8 = rng.gen();
    let n3: u8 = rng.gen();

    let a1: u16 = rng.gen();
    let a2: u16 = rng.gen();
    let a3: u16 = rng.gen();

    let mut nes = CPU6502::CPU6502::new();

    nes.write(a1, n1);
    nes.write(a2, n2);
    nes.write(a3, n3);

    let a = nes.read(a1);
    let b = nes.read(a2);
    let c = nes.read(a3);

    assert_eq!(a, n1);
    assert_eq!(b, n2);
    assert_eq!(c, n3);

}

#[test]
fn test_reset(){
    let mut nes = CPU6502::CPU6502::new();
    nes.reset();

    assert_eq!(nes.a, 0);
    assert_eq!(nes.x, 0);
    assert_eq!(nes.y, 0);
    assert_eq!(nes.sptr, 0xFD);

    let low = nes.read(nes.addr_absolute) as u16;
    let high = nes.read(nes.addr_absolute + 1) as u16;
    let a = (high << 8) | low;

    assert_eq!(nes.pc, a);

    assert_eq!(nes.addr_absolute, 0x0000);
    assert_eq!(nes.addr_relative, 0x0000);
    assert_eq!(nes.fetched, 0x00);
    assert_eq!(nes.cycles, 8);
}

#[test]
fn test_fetch(){

}

#[test]
fn test_shiftleft(){

}

#[test]
fn test_shiftright(){

}

#[test]
fn test_compareA(){

}

#[test]
fn test_compareX(){

}

#[test]
fn test_compareY(){

}

#[test]
fn test_DEY(){

}

#[test]
fn test_DEC(){

}

#[test]
fn test_DEX(){

}

#[test]
fn test_INC(){

}

#[test]
fn test_INX(){

}

#[test]
fn test_INY(){

}
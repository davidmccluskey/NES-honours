use super::*;

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

fn test_brk(){
    
}
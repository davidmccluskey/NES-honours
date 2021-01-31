use crate::bus::Bus;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::num::Wrapping;

//All operations acquired from http://www.6502.org/tutorials/6502opcodes.html

pub struct CPU6502 {
    pub bus: Bus,
    pub a: u8,		     // Accumulator Register
    pub x: u8,      	 // X Register
    pub y: u8,           // Y Register
    pub sptr: u8,	     // Stack Pointer
    pub pc: u16,	     // Program Counter
    pub sr: u8,		     // Status Register

    fetched: u8,         //Fetched data
    addr_absolute: u16,  //Absolute address
    addr_relative: u16,  //Relative address
    opcode: u8,          //Opcode for current Instruction
    cycles: u8,          //Number of cycles

    lookup: Vec<Instruction>,   //lookup table
}

bitflags! {
    pub struct Flags: u8 {
       const C = 1 << 0;	// Carry Bit
       const Z = 1 << 1;	// Zero
       const I = 1 << 2;	// Disable Interrupts
       const D = 1 << 3;	// Decimal Mode (unused in the NES)
       const B = 1 << 4;	// Break
       const U = 1 << 5;	// Unused
       const V = 1 << 6;	// Overflow
       const N = 1 << 7;	// Negative
    }
}

struct Instruction{
    name: String,
    addr_name: String,
    cycles: u8,
    addrmode: fn(&mut CPU6502) -> u8,
    operation: fn(&mut CPU6502) -> u8,
}

impl Instruction{
    pub fn new(n: String, an: String, c: u8, oc: fn(&mut CPU6502) -> u8, am: fn(&mut CPU6502) -> u8) -> Instruction {
        Instruction{
            name: n,
            addr_name: an,
            cycles: c,
            addrmode: am,
            operation: oc,
        }
    }
}

impl CPU6502{
    pub fn new() -> CPU6502 {
        let bus = Bus::new();
        let lookup = set_lookup();
        CPU6502{
            bus,
            a: 0x00,
            x: 0x00,
            y: 0x00,
            sptr: 0x00,
            pc: 0x0000,
            sr: 0x00,

            fetched: 0x00,
            addr_absolute: 0x0000,
            addr_relative: 0x0000,
            opcode: 0x00,
            cycles: 0,
            lookup,
        }
    }

    pub fn subtract_stack(&mut self){
        if self.sptr != 0{
            self.sptr = self.sptr - 1;
        }else{
            println!("Error");
        }
    }

    pub fn get_flag(&mut self, flag: Flags) -> u8{
        if self.sr & (flag.bits) > 0 {
            return 1;
        }else{
            return 0;
        }
    }

    pub fn set_flag(&mut self, flag: Flags, v: bool){
        if v == true 
        {
             self.sr = self.sr | flag.bits;
        }else
        {
            self.sr = self.sr & !flag.bits
        }
    }

    pub fn read(&mut self, addr: u16 ) -> u8{
        return self.bus.cpu_read(addr, false);
    }

    pub fn write(&mut self, addr: u16, data: &mut u8){
        self.bus.cpu_write(addr, data);
    }

    // Reset Interrupt
    pub fn reset(&mut self){
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sptr = 0xFD;
        self.sr = 0x00 | Flags::U.bits;

        self.addr_absolute = 0xFFFC;

        let low = self.read(self.addr_absolute) as u16;
        let high = self.read(self.addr_absolute + 1) as u16;

        self.pc = (high << 8) | low;
        

        self.addr_absolute = 0x0000;
        self.addr_relative = 0x0000;

        self.fetched = 0x00;

        self.cycles = 8;
        self.bus.reset();
    }

    // Interrupt Request
    pub fn irq(&mut self){
        if self.get_flag(Flags::I) == 1{
            let val = (self.pc >> 8) & 0x00FF;
            self.write(0x0100 + self.sptr as u16, &mut val.to_be_bytes()[1]);
            self.subtract_stack();

            self.set_flag(Flags::B, false);
            self.set_flag(Flags::U, true);
            self.set_flag(Flags::I, true);

            let mut sr = self.sr;
            self.write(0x0100 + self.sptr as u16, &mut sr);
            self.subtract_stack();

            self.addr_absolute = 0xFFFE;
            let low = self.read(self.addr_absolute) as u16;
            let high = self.read(self.addr_absolute + 1) as u16;

            self.pc = (high << 8) | low;

            self.cycles = 7;
        }
    }

    // Non-Maskable Interrupt Request
    pub fn nmi(&mut self){
        let val = (self.pc >> 8) & 0x00FF;
        self.write(0x0100 + self.sptr as u16, &mut val.to_be_bytes()[1]);
        self.subtract_stack();

        self.set_flag(Flags::B, false);
        self.set_flag(Flags::U, true);
        self.set_flag(Flags::I, true);

        let mut sr = self.sr;
        self.write(0x0100 + self.sptr as u16, &mut sr);
        self.subtract_stack();

        self.addr_absolute = 0xFFFE;
        let low = self.read(self.addr_absolute) as u16;
        let high = self.read(self.addr_absolute + 1) as u16;

        self.pc = (high << 8) | low;

        self.cycles = 7;
    }

    //Perform one clock cycle's worth of update
    pub fn clock(&mut self){
        if self.cycles == 0 {
            self.opcode = self.read(self.pc);

            self.pc = self.pc + 1;
            self.cycles = self.lookup[self.opcode as usize].cycles;

            let add_cycle1 = (self.lookup[self.opcode as usize].addrmode)(self);
            let add_cycle2 = (self.lookup[self.opcode as usize].operation)(self);

            self.cycles = self.cycles + (add_cycle1 + add_cycle2);
        }
        self.cycles = self.cycles - 1;
        self.bus.clock();
    }

    pub fn complete(&mut self) -> bool {
        return self.cycles == 0;
    }

    pub fn fetch(&mut self) -> u8{
        if !(self.lookup[self.opcode as usize].addr_name == "IMP".to_string()) {
            self.fetched = self.read(self.addr_absolute);
        }
        return self.fetched;
    }

    #[allow(unused_assignments)]
    pub fn disassemble(&mut self, start: u16, stop: u16) -> HashMap<u32, String>{
        let mut addr: u32 = start as u32;
        let mut value:u8 = 0x00;
        let mut lo:u8 = 0x00;
        let mut hi:u8 = 0x00;
        let mut line_addr: u32 = 0;
        let mut lines = HashMap::new();

        while addr <= stop as u32{
            line_addr = addr;
            let mut format = format!("{:X}", addr);
            let mut d = "$".to_owned();
            d.push_str(&format);
            d.push_str(": ");
            //println!("{}", d);
            let mut val = u16::try_from(addr).unwrap();
            let opcode:u8 = self.bus.cpu_read(val, true); 
            addr = addr + 1;

            let v = &self.lookup[opcode as usize].name;
            d.push_str(v);
            d.push_str(" ");

            if self.lookup[opcode as usize].addr_name == "IMP"{
                d.push_str(" {IMP}");
            }else if self.lookup[opcode as usize].addr_name == "IMM"{
                addr = addr + 1;
                d.push_str(" {IMM}");
            }else if self.lookup[opcode as usize].addr_name == "ZP0"{
                lo = self.bus.cpu_read(val, true); 
                addr = addr + 1;
                d.push_str("$");		
                format = format!("{:X}", lo);			
                d.push_str(&format);
                d.push_str(" {ZP0}");		
            }else if self.lookup[opcode as usize].addr_name == "ZPX"{
                lo = self.bus.cpu_read(val, true); 
                addr = addr + 1;
                d.push_str("$");		
                format = format!("{:X}", lo);			
                d.push_str(&format);
                d.push_str(" X {ZPX}");	
		}else if self.lookup[opcode as usize].addr_name == "ZPY"{
                lo = self.bus.cpu_read(val, true); 
                addr = addr + 1;
                d.push_str("$");		
                format = format!("{:X}", lo);			
                d.push_str(&format);
                d.push_str(" Y {ZPY}");
            }else if self.lookup[opcode as usize].addr_name == "IZX"{
                lo = self.bus.cpu_read(val, true); 
                addr = addr + 1;
                d.push_str("$");		
                format = format!("{:X}", lo);			
                d.push_str(&format);
                d.push_str(" X {IZX}");
            }else if self.lookup[opcode as usize].addr_name == "IZY"{
                lo = self.bus.cpu_read(val, true); 
                addr = addr + 1;
                d.push_str("$");		
                format = format!("{:X}", lo);			
                d.push_str(&format);
                d.push_str(" X {IZY}");
		}else if self.lookup[opcode as usize].addr_name == "ABS"{
                lo = self.bus.cpu_read(val, true); 
                val = val + 1;
                addr = addr + 1;
                hi = self.bus.cpu_read(val, true); 
                val = val + 1;
                addr = addr + 1;
                d.push_str("$");		
                let hex_val = ((hi as u16) << 8) | lo as u16;
                format = format!("{:X}", hex_val);			
                d.push_str(&format);
                d.push_str(" {ABS}");
		}else if self.lookup[opcode as usize].addr_name == "ABX"{
                lo = self.bus.cpu_read(val, true); 
                val = val + 1;
                addr = addr + 1;
                hi = self.bus.cpu_read(val, true); 
                val = val + 1;
                addr = addr + 1;
                d.push_str("$");		
                let hex_val = ((hi as u16) << 8) | lo as u16;
                format = format!("{:X}", hex_val);			
                d.push_str(&format);
                d.push_str(" X {ABX}");
		}else if self.lookup[opcode as usize].addr_name == "ABY"{
                lo = self.bus.cpu_read(val, true); 
                val = val + 1;
                addr = addr + 1;
                hi = self.bus.cpu_read(val, true); 
                val = val + 1;
                addr = addr + 1;
                d.push_str("$");		
                let hex_val = ((hi as u16) << 8) | lo as u16;
                format = format!("{:X}", hex_val);			
                d.push_str(&format);
                d.push_str(" Y {ABY}");
		}
		else if self.lookup[opcode as usize].addr_name == "IND"{
                lo = self.bus.cpu_read(val, true); 
                val = val + 1;
                addr = addr + 1;
                hi = self.bus.cpu_read(val, true); 
                val = val + 1;
                addr = addr + 1;
                d.push_str("($");		
                let hex_val = ((hi as u16) << 8) | lo as u16;
                format = format!("{:X}", hex_val);			
                d.push_str(&format);
                d.push_str(" {IND})");
		}else if self.lookup[opcode as usize].addr_name == "REL"
		{
                value = self.bus.cpu_read(val, true); 
                addr = addr + 1;
                d.push_str("$");
                format = format!("{:X}", value);			
                d.push_str(&format);
                d.push_str(" [&");

                format = format!("{:X}", (addr + value as u32));	
                d.push_str(&format);	
                d.push_str("] {REL}");
            }
                lines.insert(line_addr, d);
        }
        println!("");
        return lines;
    }

}


fn set_lookup() -> Vec<Instruction> {
    let mut lookup = Vec::new();

    lookup.push(Instruction::new("BRK".to_string(),"IMM".to_string(), 7, CPU6502::BRK, CPU6502::IMM));       //1
    lookup.push(Instruction::new("ORA".to_string(),"IZX".to_string(), 6, CPU6502::ORA, CPU6502::IZX));       //2
    lookup.push(Instruction::new("???".to_string(),"IMP".to_string(), 2, CPU6502::ILL, CPU6502::IMP));       //3
    lookup.push(Instruction::new("???".to_string(),"IMP".to_string(), 8, CPU6502::ILL, CPU6502::IMP));       //4
    lookup.push(Instruction::new("???".to_string(),"IMP".to_string(), 3, CPU6502::NOP, CPU6502::IMP));       //5
    lookup.push(Instruction::new("ORA".to_string(),"ZP0".to_string(), 3, CPU6502::ORA, CPU6502::ZP0));       //6
    lookup.push(Instruction::new("ASL".to_string(),"ZP0".to_string(), 5, CPU6502::ASL, CPU6502::ZP0));       //7
    lookup.push(Instruction::new("???".to_string(),"IMP".to_string(), 5, CPU6502::ILL, CPU6502::IMP));       //8
    lookup.push(Instruction::new("PHP".to_string(),"IMP".to_string(), 3, CPU6502::PHP, CPU6502::IMP));       //9
    lookup.push(Instruction::new("ORA".to_string(),"IMM".to_string(), 2, CPU6502::ORA, CPU6502::IMM));       //10
    lookup.push(Instruction::new("ASL".to_string(),"IMP".to_string(), 2, CPU6502::ASL, CPU6502::IMP));       //11
    lookup.push(Instruction::new("???".to_string(),"IMP".to_string(), 2, CPU6502::ILL, CPU6502::IMP));       //12
    lookup.push(Instruction::new("???".to_string(),"IMP".to_string(), 4, CPU6502::NOP, CPU6502::IMP));       //13
    lookup.push(Instruction::new("ORA".to_string(),"ABS".to_string(), 4, CPU6502::ORA, CPU6502::ABS));       //14
    lookup.push(Instruction::new("ASL".to_string(),"ABS".to_string(), 6, CPU6502::ASL, CPU6502::ABS));       //15
    lookup.push(Instruction::new("???".to_string(),"IMP".to_string(), 6, CPU6502::ILL, CPU6502::IMP));       //16


    lookup.push(Instruction::new("BPL".to_string(), "REL".to_string(),2, CPU6502::BPL, CPU6502::REL));        //1
    lookup.push(Instruction::new("ORA".to_string(), "IZY".to_string(),5, CPU6502::ORA, CPU6502::IZY));        //2
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),2, CPU6502::ILL, CPU6502::IMP));        //3
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),8, CPU6502::ILL, CPU6502::IMP));        //4
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),4, CPU6502::NOP, CPU6502::IMP));        //5
    lookup.push(Instruction::new("ORA".to_string(), "ZPX".to_string(),4, CPU6502::ORA, CPU6502::ZPX));        //6
    lookup.push(Instruction::new("ASL".to_string(), "ZPX".to_string(),6, CPU6502::ASL, CPU6502::ZPX));        //7
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),6, CPU6502::ILL, CPU6502::IMP));        //8
    lookup.push(Instruction::new("CLC".to_string(), "IMP".to_string(),2, CPU6502::CLC, CPU6502::IMP));        //9
    lookup.push(Instruction::new("ORA".to_string(), "ABY".to_string(),4, CPU6502::ORA, CPU6502::ABY));        //10
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),2, CPU6502::NOP, CPU6502::IMP));        //11
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),7, CPU6502::ILL, CPU6502::IMP));        //12
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),4, CPU6502::NOP, CPU6502::IMP));        //13
    lookup.push(Instruction::new("ORA".to_string(), "ABX".to_string(),4, CPU6502::ORA, CPU6502::ABX));        //14
    lookup.push(Instruction::new("ASL".to_string(), "ABX".to_string(),7, CPU6502::ASL, CPU6502::ABX));        //15
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),7, CPU6502::ILL, CPU6502::IMP));        //16


    lookup.push(Instruction::new("JSR".to_string(), "ABS".to_string(),6, CPU6502::JSR, CPU6502::ABS));        //1
    lookup.push(Instruction::new("AND".to_string(), "IZX".to_string(),6, CPU6502::AND, CPU6502::IZX));        //2
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),2, CPU6502::ILL, CPU6502::IMP));        //3
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),8, CPU6502::ILL, CPU6502::IMP));        //4
    lookup.push(Instruction::new("BIT".to_string(), "ZP0".to_string(),3, CPU6502::BIT, CPU6502::ZP0));        //5
    lookup.push(Instruction::new("AND".to_string(), "ZP0".to_string(),3, CPU6502::AND, CPU6502::ZP0));        //6
    lookup.push(Instruction::new("ROL".to_string(), "ZP0".to_string(),5, CPU6502::ROL, CPU6502::ZP0));        //7
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),5, CPU6502::ILL, CPU6502::IMP));        //8
    lookup.push(Instruction::new("PLP".to_string(), "IMP".to_string(),4, CPU6502::PLP, CPU6502::IMP));        //9
    lookup.push(Instruction::new("AND".to_string(), "IMM".to_string(),2, CPU6502::AND, CPU6502::IMM));        //10
    lookup.push(Instruction::new("ROL".to_string(), "IMP".to_string(),2, CPU6502::ROL, CPU6502::IMP));        //11
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),2, CPU6502::ILL, CPU6502::IMP));        //12
    lookup.push(Instruction::new("BIT".to_string(), "ABS".to_string(),4, CPU6502::BIT, CPU6502::ABS));        //13
    lookup.push(Instruction::new("AND".to_string(), "ABS".to_string(),4, CPU6502::AND, CPU6502::ABS));        //14
    lookup.push(Instruction::new("ROL".to_string(), "ABS".to_string(),6, CPU6502::ROL, CPU6502::ABS));        //15
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),6, CPU6502::ILL, CPU6502::IMP));        //16


    lookup.push(Instruction::new("BMI".to_string(), "REL".to_string(),2, CPU6502::BMI, CPU6502::REL));        //1
    lookup.push(Instruction::new("AND".to_string(), "IZY".to_string(),5, CPU6502::AND, CPU6502::IZY));        //2
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),2, CPU6502::ILL, CPU6502::IMP));        //3
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),8, CPU6502::ILL, CPU6502::IMP));        //4
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),4, CPU6502::NOP, CPU6502::IMP));        //5
    lookup.push(Instruction::new("AND".to_string(), "ZPX".to_string(),4, CPU6502::AND, CPU6502::ZPX));        //6
    lookup.push(Instruction::new("ROL".to_string(), "ZPX".to_string(),6, CPU6502::ROL, CPU6502::ZPX));        //7
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),6, CPU6502::ILL, CPU6502::IMP));        //8
    lookup.push(Instruction::new("SEC".to_string(), "IMP".to_string(),2, CPU6502::SEC, CPU6502::IMP));        //9
    lookup.push(Instruction::new("AND".to_string(), "ABY".to_string(),4, CPU6502::AND, CPU6502::ABY));        //10
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),2, CPU6502::NOP, CPU6502::IMP));        //11
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),7, CPU6502::ILL, CPU6502::IMP));        //12
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),4, CPU6502::NOP, CPU6502::IMP));        //13
    lookup.push(Instruction::new("AND".to_string(), "ABX".to_string(),4, CPU6502::AND, CPU6502::ABX));        //14
    lookup.push(Instruction::new("ROL".to_string(), "ABX".to_string(),7, CPU6502::ROL, CPU6502::ABX));        //15
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),7, CPU6502::ILL, CPU6502::IMP));        //16


    lookup.push(Instruction::new("RTI".to_string(), "IMP".to_string(),6, CPU6502::RTI, CPU6502::IMP));        //1
    lookup.push(Instruction::new("EOR".to_string(), "IZX".to_string(),6, CPU6502::EOR, CPU6502::IZX));        //2
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),2, CPU6502::ILL, CPU6502::IMP));        //3
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),8, CPU6502::ILL, CPU6502::IMP));        //4
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),3, CPU6502::NOP, CPU6502::IMP));        //5
    lookup.push(Instruction::new("EOR".to_string(), "ZP0".to_string(),3, CPU6502::EOR, CPU6502::ZP0));        //6
    lookup.push(Instruction::new("LSR".to_string(), "ZP0".to_string(),5, CPU6502::LSR, CPU6502::ZP0));        //7
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),5, CPU6502::ILL, CPU6502::IMP));        //8
    lookup.push(Instruction::new("PHA".to_string(), "IMP".to_string(),3, CPU6502::PHA, CPU6502::IMP));        //9
    lookup.push(Instruction::new("EOR".to_string(), "IMM".to_string(),2, CPU6502::EOR, CPU6502::IMM));        //10
    lookup.push(Instruction::new("LSR".to_string(), "IMP".to_string(),2, CPU6502::LSR, CPU6502::IMP));        //11
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),2, CPU6502::ILL, CPU6502::IMP));        //12
    lookup.push(Instruction::new("JMP".to_string(), "ABS".to_string(),3, CPU6502::JMP, CPU6502::ABS));        //13
    lookup.push(Instruction::new("EOR".to_string(), "ABS".to_string(),4, CPU6502::EOR, CPU6502::ABS));        //14
    lookup.push(Instruction::new("LSR".to_string(), "ABS".to_string(),6, CPU6502::LSR, CPU6502::ABS));        //15
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),6, CPU6502::ILL, CPU6502::IMP));        //16


    lookup.push(Instruction::new("BVC".to_string(), "REL".to_string(),2, CPU6502::BVC, CPU6502::REL));        //1
    lookup.push(Instruction::new("EOR".to_string(), "IZY".to_string(),5, CPU6502::EOR, CPU6502::IZY));        //2
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),2, CPU6502::ILL, CPU6502::IMP));        //3
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),8, CPU6502::ILL, CPU6502::IMP));        //4
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),4, CPU6502::NOP, CPU6502::IMP));        //5
    lookup.push(Instruction::new("EOR".to_string(), "ZPX".to_string(),4, CPU6502::EOR, CPU6502::ZPX));        //6
    lookup.push(Instruction::new("LSR".to_string(), "ZPX".to_string(),6, CPU6502::LSR, CPU6502::ZPX));        //7
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),6, CPU6502::ILL, CPU6502::IMP));        //8
    lookup.push(Instruction::new("CLI".to_string(), "IMP".to_string(),2, CPU6502::CLI, CPU6502::IMP));        //9
    lookup.push(Instruction::new("EOR".to_string(), "ABY".to_string(),4, CPU6502::EOR, CPU6502::ABY));        //10
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),2, CPU6502::NOP, CPU6502::IMP));        //11
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),7, CPU6502::ILL, CPU6502::IMP));        //12
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),4, CPU6502::NOP, CPU6502::IMP));        //13
    lookup.push(Instruction::new("EOR".to_string(), "ABX".to_string(),4, CPU6502::EOR, CPU6502::ABX));        //14
    lookup.push(Instruction::new("LSR".to_string(), "ABX".to_string(),7, CPU6502::LSR, CPU6502::ABX));        //15
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),7, CPU6502::ILL, CPU6502::IMP));        //16


    lookup.push(Instruction::new("RTS".to_string(), "IMP".to_string(),6, CPU6502::RTS, CPU6502::IMP));        //1
    lookup.push(Instruction::new("ADC".to_string(), "IZX".to_string(),6, CPU6502::ADC, CPU6502::IZX));        //2
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),2, CPU6502::ILL, CPU6502::IMP));        //3
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),8, CPU6502::ILL, CPU6502::IMP));        //4
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),3, CPU6502::NOP, CPU6502::IMP));        //5
    lookup.push(Instruction::new("ADC".to_string(), "ZP0".to_string(),3, CPU6502::ADC, CPU6502::ZP0));        //6
    lookup.push(Instruction::new("ROR".to_string(), "ZP0".to_string(),5, CPU6502::ROR, CPU6502::ZP0));        //7
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),5, CPU6502::ILL, CPU6502::IMP));        //8
    lookup.push(Instruction::new("PLA".to_string(), "IMP".to_string(),4, CPU6502::PLA, CPU6502::IMP));        //9
    lookup.push(Instruction::new("ADC".to_string(), "IMM".to_string(),2, CPU6502::ADC, CPU6502::IMM));        //10
    lookup.push(Instruction::new("ROR".to_string(), "IMP".to_string(),2, CPU6502::ROR, CPU6502::IMP));        //11
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),2, CPU6502::ILL, CPU6502::IMP));        //12
    lookup.push(Instruction::new("JMP".to_string(), "IND".to_string(),5, CPU6502::JMP, CPU6502::IND));        //13
    lookup.push(Instruction::new("ADC".to_string(), "ABS".to_string(),4, CPU6502::ADC, CPU6502::ABS));        //14
    lookup.push(Instruction::new("ROR".to_string(), "ABS".to_string(),6, CPU6502::ROR, CPU6502::ABS));        //15
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),6, CPU6502::ILL, CPU6502::IMP));        //16


    lookup.push(Instruction::new("BVS".to_string(), "REL".to_string(),2, CPU6502::BVS, CPU6502::REL));        //1
    lookup.push(Instruction::new("ADC".to_string(), "IZY".to_string(),5, CPU6502::ADC, CPU6502::IZY));        //2
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),2, CPU6502::ILL, CPU6502::IMP));        //3
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),8, CPU6502::ILL, CPU6502::IMP));        //4
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),4, CPU6502::NOP, CPU6502::IMP));        //5
    lookup.push(Instruction::new("ADC".to_string(), "ZPX".to_string(),4, CPU6502::ADC, CPU6502::ZPX));        //6
    lookup.push(Instruction::new("ROR".to_string(), "ZPX".to_string(),6, CPU6502::ROR, CPU6502::ZPX));        //7
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),6, CPU6502::ILL, CPU6502::IMP));        //8
    lookup.push(Instruction::new("SEI".to_string(), "IMP".to_string(),2, CPU6502::SEI, CPU6502::IMP));        //9
    lookup.push(Instruction::new("ADC".to_string(), "ABY".to_string(),4, CPU6502::ADC, CPU6502::ABY));        //10
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),2, CPU6502::NOP, CPU6502::IMP));        //11
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),7, CPU6502::ILL, CPU6502::IMP));        //12
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),4, CPU6502::NOP, CPU6502::IMP));        //13
    lookup.push(Instruction::new("ADC".to_string(), "ABX".to_string(),4, CPU6502::ADC, CPU6502::ABX));        //14
    lookup.push(Instruction::new("ROR".to_string(), "ABX".to_string(),7, CPU6502::ROR, CPU6502::ABX));        //15
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),7, CPU6502::ILL, CPU6502::IMP));        //16


    lookup.push(Instruction::new("???".to_string(),"IMP".to_string(), 2, CPU6502::NOP, CPU6502::IMP));        //1
    lookup.push(Instruction::new("STA".to_string(),"IZX".to_string(), 6, CPU6502::STA, CPU6502::IZX));        //2
    lookup.push(Instruction::new("???".to_string(),"IMP".to_string(), 2, CPU6502::NOP, CPU6502::IMP));        //3
    lookup.push(Instruction::new("???".to_string(),"IMP".to_string(), 6, CPU6502::ILL, CPU6502::IMP));        //4
    lookup.push(Instruction::new("STY".to_string(),"ZP0".to_string(), 3, CPU6502::STY, CPU6502::ZP0));        //5
    lookup.push(Instruction::new("STA".to_string(),"ZP0".to_string(), 3, CPU6502::STA, CPU6502::ZP0));        //6
    lookup.push(Instruction::new("STX".to_string(),"ZP0".to_string(), 3, CPU6502::STX, CPU6502::ZP0));        //7
    lookup.push(Instruction::new("???".to_string(),"IMP".to_string(), 3, CPU6502::ILL, CPU6502::IMP));        //8
    lookup.push(Instruction::new("DEY".to_string(),"IMP".to_string(), 2, CPU6502::DEY, CPU6502::IMP));        //9
    lookup.push(Instruction::new("???".to_string(),"IMP".to_string(), 2, CPU6502::NOP, CPU6502::IMP));        //10
    lookup.push(Instruction::new("TXA".to_string(),"IMP".to_string(), 2, CPU6502::TXA, CPU6502::IMP));        //11
    lookup.push(Instruction::new("???".to_string(),"IMP".to_string(), 2, CPU6502::ILL, CPU6502::IMP));        //12
    lookup.push(Instruction::new("STY".to_string(),"ABS".to_string(), 4, CPU6502::STY, CPU6502::ABS));        //13
    lookup.push(Instruction::new("STA".to_string(),"ABS".to_string(), 4, CPU6502::STA, CPU6502::ABS));        //14
    lookup.push(Instruction::new("STX".to_string(),"ABS".to_string(), 4, CPU6502::STX, CPU6502::ABS));        //15
    lookup.push(Instruction::new("???".to_string(),"IMP".to_string(), 4, CPU6502::ILL, CPU6502::IMP));        //16


    lookup.push(Instruction::new("BCC".to_string(), "REL".to_string(),2, CPU6502::BCC, CPU6502::REL));        //1
    lookup.push(Instruction::new("STA".to_string(), "IZY".to_string(),6, CPU6502::STA, CPU6502::IZY));        //2
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),2, CPU6502::ILL, CPU6502::IMP));        //3
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),6, CPU6502::ILL, CPU6502::IMP));        //4
    lookup.push(Instruction::new("STY".to_string(), "ZPX".to_string(),4, CPU6502::STY, CPU6502::ZPX));        //5
    lookup.push(Instruction::new("STA".to_string(), "ZPX".to_string(),4, CPU6502::STA, CPU6502::ZPX));        //6
    lookup.push(Instruction::new("STX".to_string(), "ZPY".to_string(),4, CPU6502::STX, CPU6502::ZPY));        //7
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),4, CPU6502::ILL, CPU6502::IMP));        //8
    lookup.push(Instruction::new("TYA".to_string(), "IMP".to_string(),2, CPU6502::TYA, CPU6502::IMP));        //9
    lookup.push(Instruction::new("STA".to_string(), "ABY".to_string(),5, CPU6502::STA, CPU6502::ABY));        //10
    lookup.push(Instruction::new("TXS".to_string(), "IMP".to_string(),2, CPU6502::TXS, CPU6502::IMP));        //11
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),5, CPU6502::ILL, CPU6502::IMP));        //12
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),5, CPU6502::NOP, CPU6502::IMP));        //13
    lookup.push(Instruction::new("STA".to_string(), "ABX".to_string(),5, CPU6502::STA, CPU6502::ABX));        //14
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),5, CPU6502::ILL, CPU6502::IMP));        //15
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),5, CPU6502::ILL, CPU6502::IMP));        //16


    lookup.push(Instruction::new("LDY".to_string(), "IMM".to_string(),2, CPU6502::LDY, CPU6502::IMM));        //1
    lookup.push(Instruction::new("LDA".to_string(), "IZX".to_string(),6, CPU6502::LDA, CPU6502::IZX));        //2
    lookup.push(Instruction::new("LDX".to_string(), "IMM".to_string(),2, CPU6502::LDX, CPU6502::IMM));        //3
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),6, CPU6502::ILL, CPU6502::IMP));        //4
    lookup.push(Instruction::new("LDY".to_string(), "ZP0".to_string(),3, CPU6502::LDY, CPU6502::ZP0));        //5
    lookup.push(Instruction::new("LDA".to_string(), "ZP0".to_string(),3, CPU6502::LDA, CPU6502::ZP0));        //6
    lookup.push(Instruction::new("LDX".to_string(), "ZP0".to_string(),3, CPU6502::LDX, CPU6502::ZP0));        //7
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),3, CPU6502::ILL, CPU6502::IMP));        //8
    lookup.push(Instruction::new("TAY".to_string(), "IMP".to_string(),2, CPU6502::TAY, CPU6502::IMP));        //9
    lookup.push(Instruction::new("LDA".to_string(), "IMM".to_string(),2, CPU6502::LDA, CPU6502::IMM));        //10
    lookup.push(Instruction::new("TAX".to_string(), "IMP".to_string(),2, CPU6502::TAX, CPU6502::IMP));        //11
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),2, CPU6502::ILL, CPU6502::IMP));        //12
    lookup.push(Instruction::new("LDY".to_string(), "ABS".to_string(),4, CPU6502::LDY, CPU6502::ABS));        //13
    lookup.push(Instruction::new("LDA".to_string(), "ABS".to_string(),4, CPU6502::LDA, CPU6502::ABS));        //14
    lookup.push(Instruction::new("LDX".to_string(), "ABS".to_string(),4, CPU6502::LDX, CPU6502::ABS));        //15
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),4, CPU6502::ILL, CPU6502::IMP));        //16


    lookup.push(Instruction::new("BCS".to_string(), "REL".to_string(),2, CPU6502::BCS, CPU6502::REL));        //1
    lookup.push(Instruction::new("LDA".to_string(), "IZY".to_string(),5, CPU6502::LDA, CPU6502::IZY));        //2
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),2, CPU6502::ILL, CPU6502::IMP));        //3
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),5, CPU6502::ILL, CPU6502::IMP));        //4
    lookup.push(Instruction::new("LDY".to_string(), "ZPX".to_string(),4, CPU6502::LDY, CPU6502::ZPX));        //5
    lookup.push(Instruction::new("LDA".to_string(), "ZPX".to_string(),4, CPU6502::LDA, CPU6502::ZPX));        //6
    lookup.push(Instruction::new("LDX".to_string(), "ZPY".to_string(),4, CPU6502::LDX, CPU6502::ZPY));        //7
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),4, CPU6502::ILL, CPU6502::IMP));        //8
    lookup.push(Instruction::new("CLV".to_string(), "IMP".to_string(),2, CPU6502::CLV, CPU6502::IMP));        //9
    lookup.push(Instruction::new("LDA".to_string(), "ABY".to_string(),4, CPU6502::LDA, CPU6502::ABY));        //10
    lookup.push(Instruction::new("TSX".to_string(), "IMP".to_string(),2, CPU6502::TSX, CPU6502::IMP));        //11
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),4, CPU6502::ILL, CPU6502::IMP));        //12
    lookup.push(Instruction::new("LDY".to_string(), "ABX".to_string(),4, CPU6502::LDY, CPU6502::ABX));        //13
    lookup.push(Instruction::new("LDA".to_string(), "ABX".to_string(),4, CPU6502::LDA, CPU6502::ABX));        //14
    lookup.push(Instruction::new("LDX".to_string(), "ABY".to_string(),4, CPU6502::LDX, CPU6502::ABY));        //15
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),4, CPU6502::ILL, CPU6502::IMP));        //16


    lookup.push(Instruction::new("CPY".to_string(), "IMM".to_string(),2, CPU6502::CPY, CPU6502::IMM));        //1
    lookup.push(Instruction::new("CMP".to_string(), "IZX".to_string(),6, CPU6502::CMP, CPU6502::IZX));        //2
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),2, CPU6502::NOP, CPU6502::IMP));        //3
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),8, CPU6502::ILL, CPU6502::IMP));        //4
    lookup.push(Instruction::new("CPY".to_string(), "ZP0".to_string(),3, CPU6502::CPY, CPU6502::ZP0));        //5
    lookup.push(Instruction::new("CMP".to_string(), "ZP0".to_string(),3, CPU6502::CMP, CPU6502::ZP0));        //6
    lookup.push(Instruction::new("DEC".to_string(), "ZP0".to_string(),5, CPU6502::DEC, CPU6502::ZP0));        //7
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),5, CPU6502::ILL, CPU6502::IMP));        //8
    lookup.push(Instruction::new("INY".to_string(), "IMP".to_string(),2, CPU6502::INY, CPU6502::IMP));        //9
    lookup.push(Instruction::new("CMP".to_string(), "IMM".to_string(),2, CPU6502::CMP, CPU6502::IMM));        //10
    lookup.push(Instruction::new("DEX".to_string(), "IMP".to_string(),2, CPU6502::DEX, CPU6502::IMP));        //11
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),2, CPU6502::ILL, CPU6502::IMP));        //12
    lookup.push(Instruction::new("CPY".to_string(), "ABS".to_string(),4, CPU6502::CPY, CPU6502::ABS));        //13
    lookup.push(Instruction::new("CMP".to_string(), "ABS".to_string(),4, CPU6502::CMP, CPU6502::ABS));        //14
    lookup.push(Instruction::new("DEC".to_string(), "ABS".to_string(),6, CPU6502::DEC, CPU6502::ABS));        //15
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),6, CPU6502::ILL, CPU6502::IMP));        //16


    lookup.push(Instruction::new("BNE".to_string(), "REL".to_string(),2, CPU6502::BNE, CPU6502::REL));        //1
    lookup.push(Instruction::new("CMP".to_string(), "IZY".to_string(),5, CPU6502::CMP, CPU6502::IZY));        //2
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),2, CPU6502::ILL, CPU6502::IMP));        //3
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),8, CPU6502::ILL, CPU6502::IMP));        //4
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),4, CPU6502::NOP, CPU6502::IMP));        //5
    lookup.push(Instruction::new("CMP".to_string(), "ZPX".to_string(),4, CPU6502::CMP, CPU6502::ZPX));        //6
    lookup.push(Instruction::new("DEC".to_string(), "ZPX".to_string(),6, CPU6502::DEC, CPU6502::ZPX));        //7
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),6, CPU6502::ILL, CPU6502::IMP));        //8
    lookup.push(Instruction::new("CLD".to_string(), "IMP".to_string(),2, CPU6502::CLD, CPU6502::IMP));        //9
    lookup.push(Instruction::new("CMP".to_string(), "ABY".to_string(),4, CPU6502::CMP, CPU6502::ABY));        //10
    lookup.push(Instruction::new("NOP".to_string(), "IMP".to_string(),2, CPU6502::NOP, CPU6502::IMP));        //11
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),7, CPU6502::ILL, CPU6502::IMP));        //12
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),4, CPU6502::NOP, CPU6502::IMP));        //13
    lookup.push(Instruction::new("CMP".to_string(), "ABX".to_string(),4, CPU6502::CMP, CPU6502::ABX));        //14
    lookup.push(Instruction::new("DEC".to_string(), "ABX".to_string(),7, CPU6502::DEC, CPU6502::ABX));        //15
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),7, CPU6502::ILL, CPU6502::IMP));        //16


    lookup.push(Instruction::new("CPX".to_string(), "IMM".to_string(),2, CPU6502::CPX, CPU6502::IMM));        //1
    lookup.push(Instruction::new("SBC".to_string(), "IZX".to_string(),6, CPU6502::SBC, CPU6502::IZX));        //2
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),2, CPU6502::NOP, CPU6502::IMP));        //3
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),8, CPU6502::ILL, CPU6502::IMP));        //4
    lookup.push(Instruction::new("CPX".to_string(), "ZP0".to_string(),3, CPU6502::CPX, CPU6502::ZP0));        //5
    lookup.push(Instruction::new("SBC".to_string(), "ZP0".to_string(),3, CPU6502::SBC, CPU6502::ZP0));        //6
    lookup.push(Instruction::new("INC".to_string(), "ZP0".to_string(),5, CPU6502::INC, CPU6502::ZP0));        //7
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),5, CPU6502::ILL, CPU6502::IMP));        //8
    lookup.push(Instruction::new("INX".to_string(), "IMP".to_string(),2, CPU6502::INX, CPU6502::IMP));        //9
    lookup.push(Instruction::new("SBC".to_string(), "IMM".to_string(),2, CPU6502::SBC, CPU6502::IMM));        //10
    lookup.push(Instruction::new("NOP".to_string(), "IMP".to_string(),2, CPU6502::NOP, CPU6502::IMP));        //11
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),2, CPU6502::SBC, CPU6502::IMP));        //12
    lookup.push(Instruction::new("CPX".to_string(), "ABS".to_string(),4, CPU6502::CPX, CPU6502::ABS));        //13
    lookup.push(Instruction::new("SBC".to_string(), "ABS".to_string(),4, CPU6502::SBC, CPU6502::ABS));        //14
    lookup.push(Instruction::new("INC".to_string(), "ABS".to_string(),6, CPU6502::INC, CPU6502::ABS));        //15
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),6, CPU6502::ILL, CPU6502::IMP));        //16


    lookup.push(Instruction::new("BEQ".to_string(), "REL".to_string(),2, CPU6502::BEQ, CPU6502::REL));        //1
    lookup.push(Instruction::new("SBC".to_string(), "IZY".to_string(),5, CPU6502::SBC, CPU6502::IZY));        //2
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),2, CPU6502::ILL, CPU6502::IMP));        //3
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),8, CPU6502::ILL, CPU6502::IMP));        //4
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),4, CPU6502::NOP, CPU6502::IMP));        //5
    lookup.push(Instruction::new("SBC".to_string(), "ZPX".to_string(),4, CPU6502::SBC, CPU6502::ZPX));        //6
    lookup.push(Instruction::new("INC".to_string(), "ZPX".to_string(),6, CPU6502::INC, CPU6502::ZPX));        //7
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),6, CPU6502::ILL, CPU6502::IMP));        //8
    lookup.push(Instruction::new("SED".to_string(), "IMP".to_string(),2, CPU6502::SED, CPU6502::IMP));        //9
    lookup.push(Instruction::new("SBC".to_string(), "ABY".to_string(),4, CPU6502::SBC, CPU6502::ABY));        //10
    lookup.push(Instruction::new("NOP".to_string(), "IMP".to_string(),2, CPU6502::NOP, CPU6502::IMP));        //11
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),7, CPU6502::ILL, CPU6502::IMP));        //12
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),4, CPU6502::NOP, CPU6502::IMP));        //13
    lookup.push(Instruction::new("SBC".to_string(), "ABX".to_string(),4, CPU6502::SBC, CPU6502::ABX));        //14
    lookup.push(Instruction::new("INC".to_string(), "ABX".to_string(),7, CPU6502::INC, CPU6502::ABX));        //15
    lookup.push(Instruction::new("???".to_string(), "IMP".to_string(),7, CPU6502::ILL, CPU6502::IMP));        //16


    return lookup;

}

#[allow(non_snake_case)]
impl CPU6502 {
    //Addressing modes
    fn IMP(&mut self) -> u8{
        self.fetched = self.a;
        return 0;
    }

    fn IMM(&mut self) -> u8{
        self.addr_absolute = self.pc;
        self.pc = self.pc + 1;
        return 0;
    }

    fn ZP0(&mut self) -> u8{
        self.addr_absolute = self.read(self.pc) as u16; //KEEP AN EYE ON THIS
        self.pc = self.pc + 1;
        self.addr_absolute &= 0x00FF;
        return 0;
    }

    fn ZPX(&mut self) -> u8{
        self.addr_absolute = (self.read(self.pc) + self.x) as u16; //KEEP AN EYE ON THIS
        self.pc = self.pc + 1;
        self.addr_absolute &= 0x00FF;
        return 0;
    }

    fn ZPY(&mut self) -> u8{
        self.addr_absolute = (self.read(self.pc) + self.y) as u16; //KEEP AN EYE ON THIS
        self.addr_absolute &= 0x00FF;
        return 0;
    }

    fn ABS(&mut self) -> u8{
        let low = self.read(self.pc) as u16;
        self.pc = self.pc + 1;

        let high = self.read(self.pc) as u16;
        self.pc = self.pc + 1;

        self.addr_absolute = (high << 8 ) | low;
        return 0;
    }

    fn ABX(&mut self) -> u8{
        let low = self.read(self.pc) as u16;
        self.pc = self.pc + 1;

        let high = self.read(self.pc) as u16;
        self.pc = self.pc + 1;

        self.addr_absolute = (high << 8 ) | low;
        self.addr_absolute = self.addr_absolute + self.x as u16;

        if(self.addr_absolute & 0xFF00) != high << 8{
            return 1;
        }else{
            return 0;
        }
    }

    fn ABY(&mut self) -> u8{
        let low = self.read(self.pc) as u16;
        self.pc = self.pc + 1;

        let high = self.read(self.pc) as u16;
        self.pc = self.pc + 1;

        self.addr_absolute = (high << 8 ) | low;
        self.addr_absolute = self.addr_absolute + self.y as u16;

        if(self.addr_absolute & 0xFF00) != high << 8{
            return 1;
        }else{
            return 0;
        }
    }

    fn IND(&mut self) -> u8{
        let ptr_low = self.read(self.pc) as u16;
        self.pc = self.pc + 1;

        let ptr_high = self.read(self.pc) as u16;
        self.pc = self.pc + 1;

        let ptr = (ptr_high << 8) | ptr_low;

        if ptr_low == 0x00FF 
        {
            let low = self.read(((self.sptr as u16) & 0xFF00) << 8) as u16;
            let high = self.read(ptr + 0) as u16;
            self.addr_absolute = (low | high) as u16; //OVERFLOW
        }
        else 
        {
            let low = self.read((((self.sptr + 1) as u16)) << 8) as u16;
            self.addr_absolute = (low | self.read(ptr + 0) as u16) as u16; //OVERFLOW
        }
        
        return 0;
    }
    fn IZX(&mut self) -> u8{
        let t = self.read(self.pc) as u16;
        self.pc = self.pc + 1;
    
        let low = self.read((t + self.x as u16) & 0x00FF) as u16;
        let high = self.read((t + self.x  as u16 + 1) & 0x00FF) as u16;
    
        self.addr_absolute = ((high << 8)) as u16 | low as u16;
        return 0;
    }
    fn IZY(&mut self) -> u8{
        let t = self.read(self.pc) as u16;
        self.pc = self.pc + 1;
    
        let low = self.read(t & 0x00FF) as u16;
        let high = self.read((t + 1) & 0x00FF) as u16;
    
        self.addr_absolute = (high << 8) | low;
        self.addr_absolute = self.addr_absolute + self.y as u16;
        
        if(self.addr_absolute & 0xFF00) != (high << 8){
            return 1;
        }else{
            return 0;
        }
    }
    fn REL(&mut self) -> u8{
        self.addr_relative = self.read(self.pc) as u16;
        self.pc = self.pc + 1;
        if  (self.addr_relative & 0x80) != 0 { //BUG??
            self.addr_relative |= 0xFF00;
        }
        return 0;
    }
}

#[allow(non_snake_case)]
impl CPU6502{
    //Operations

    //Addition
    fn ADC(&mut self) -> u8{
        self.fetch();
        let flagC = self.get_flag(Flags::C);
        let tmp = (self.a + self.fetched + flagC) as u16;

        if tmp > 255 {
            self.set_flag(Flags::C, true);
        }else{
            self.set_flag(Flags::C, false);
        }
        self.set_flag(Flags::Z, (tmp & 0x00FF) == 1);

        let a16 = self.a as u16;
        let fetched16 = self.fetched as u16;
        let flag = (!(a16 ^ fetched16) & (a16 ^ tmp)) & 0x0080;
        if flag == 1{
            self.set_flag(Flags::V, true);
        }else{
            self.set_flag(Flags::V, false);
        }

        self.a = (tmp & 0x00FF).to_be_bytes()[1];

        return 1;
    }

    //Subtraction
    fn SBC(&mut self) -> u8{
        self.fetch();
        let inversion = self.fetched ^ 0x00FF;

        let tmp = (self.a + inversion + self.get_flag(Flags::C)) as u16;

        self.set_flag(Flags::C, tmp > 255);
        self.set_flag(Flags::Z, (tmp & 0x00FF) == 1);

        let a16 = self.a as u16;
        let fetched16 = self.fetched as u16;
        let flag = (!(a16 ^ fetched16) & (a16 ^ tmp)) & 0x0080;
        if flag == 1{
            self.set_flag(Flags::V, true);
        }else{
            self.set_flag(Flags::V, false);
        }

        self.a = (tmp & 0x00FF).to_be_bytes()[1];

        return 0;
    }

    //Bitwise AND
    fn AND(&mut self) -> u8{
        self.fetch();
        self.a = self.a & self.fetched;
        self.set_flag(Flags::Z, self.a == 0x00);

        if self.a & 0x80 > 0 {
            self.set_flag(Flags::N, true);
        }else{
            self.set_flag(Flags::N, false)
        }
        return 0;
    }

    //Shift left
    fn ASL(&mut self) -> u8{
        self.fetch();
        let tmp = (self.fetched << 1) as u16;
        let temp = self.fetched << 1;
        self.set_flag(Flags::C, (tmp & 0xFF00) > 0);
        self.set_flag(Flags::Z, (tmp & 0x00FF) == 0x00);
        self.set_flag(Flags::N, tmp & 0x80 > 0);

        if self.lookup[self.opcode as usize].addr_name == "IMP"{
            self.a = temp & 0x00FF;
        }
        else{
            self.write(self.addr_absolute, &mut (temp & 0x00FF));
        }
        return 0;
    }

    //Branch if carry bit 0
    fn BCC(&mut self) -> u8{
        if self.get_flag(Flags::C) == 0 {
            self.cycles = self.cycles + 1;
            let wrapped_pc = Wrapping(self.pc);
            let wrapped_addr = Wrapping(self.addr_relative);
            self.addr_absolute = (wrapped_pc + wrapped_addr).0;

            if self.addr_absolute & 0xFF00 != self.pc & 0xFF00{
                self.cycles = self.cycles + 1;
            }
            self.pc = self.addr_absolute;
        }
        return 0;
    }

    //Branch if carry bit set
    fn BCS(&mut self) -> u8{
        if self.get_flag(Flags::C) == 1 {
            self.cycles = self.cycles + 1;
            let wrapped_pc = Wrapping(self.pc);
            let wrapped_addr = Wrapping(self.addr_relative);
            self.addr_absolute = (wrapped_pc + wrapped_addr).0;

            if self.addr_absolute & 0xFF00 != self.pc & 0xFF00{
                self.cycles = self.cycles + 1;
            }
            self.pc = self.addr_absolute;
        }
        return 0;
    }

    //Branch if equal
    fn BEQ(&mut self) -> u8{
        if self.get_flag(Flags::Z) == 1 {
            self.cycles = self.cycles + 1;
            let wrapped_pc = Wrapping(self.pc);
            let wrapped_addr = Wrapping(self.addr_relative);
            self.addr_absolute = (wrapped_pc + wrapped_addr).0;

            if self.addr_absolute & 0xFF00 != self.pc & 0xFF00{
                self.cycles = self.cycles + 1;
            }
            self.pc = self.addr_absolute;
        }
        return 0;
    }


    fn BIT(&mut self) -> u8{
        self.fetch();
        let tmp = self.a & self.fetched;
        self.set_flag(Flags::Z, (tmp & 0x00FF) == 0x00);
        self.set_flag(Flags::N, self.fetched & (1 << 7) > 0);
        self.set_flag(Flags::V, self.fetched & (1 << 6) > 0);
        return 0;
    }

    //Branch if negative
    fn BMI(&mut self) -> u8{
        if self.get_flag(Flags::N) == 1 {
            self.cycles = self.cycles + 1;
            let wrapped_pc = Wrapping(self.pc);
            let wrapped_addr = Wrapping(self.addr_relative);
            self.addr_absolute = (wrapped_pc + wrapped_addr).0;

            if self.addr_absolute & 0xFF00 != self.pc & 0xFF00{
                self.cycles = self.cycles + 1;
            }
            self.pc = self.addr_absolute;
        }
        return 0;
    }

    //Branch if not equal 
    fn BNE(&mut self) -> u8{
        if self.get_flag(Flags::Z) == 0 {
            self.cycles = self.cycles + 1;
            let wrapped_pc = Wrapping(self.pc);
            let wrapped_addr = Wrapping(self.addr_relative);
            self.addr_absolute = (wrapped_pc + wrapped_addr).0;

            if self.addr_absolute & 0xFF00 != self.pc & 0xFF00{
                self.cycles = self.cycles + 1;
            }
            self.pc = self.addr_absolute;
        }
        return 0;
    }

    //Branch if positive
    fn BPL(&mut self) -> u8{
        if self.get_flag(Flags::N) == 0 {
            self.cycles = self.cycles + 1;
            let wrapped_pc = Wrapping(self.pc);
            let wrapped_addr = Wrapping(self.addr_relative);
            self.addr_absolute = (wrapped_pc + wrapped_addr).0;

            if self.addr_absolute & 0xFF00 != self.pc & 0xFF00{
                self.cycles = self.cycles + 1;
            }
            self.pc = self.addr_absolute;
        }
        return 0;
    }

    //Break
    fn BRK(&mut self) -> u8{
        self.pc = self.pc + 1;
	
        self.set_flag(Flags::I, true);
        let addr = 0x0100 + self.sptr as u16;
        let mut data = (self.pc >> 8) as u8 & 0x00FF;
        self.write(addr, &mut data);
        self.subtract_stack();
        self.write(0x0100 + self.sptr as u16, &mut ((self.pc & 0x00FF) as u8));
        self.subtract_stack();
    
        self.set_flag(Flags::B, true);
        let mut sr = self.sr;
        self.write(0x0100 + self.sptr as u16, &mut sr);
        self.subtract_stack();
        self.set_flag(Flags::B, false);
    
        self.pc = self.read(0xFFFE) as u16 | ((self.read(0xFFFF) as u16) << 8);
        return 0;
    }

    //Branch if overflow 0
    fn BVC(&mut self) -> u8{
        if self.get_flag(Flags::V) == 0 {
            self.cycles = self.cycles + 1;
            let wrapped_pc = Wrapping(self.pc);
            let wrapped_addr = Wrapping(self.addr_relative);
            self.addr_absolute = (wrapped_pc + wrapped_addr).0;

            if self.addr_absolute & 0xFF00 != self.pc & 0xFF00{
                self.cycles = self.cycles + 1;
            }
            self.pc = self.addr_absolute;
        }
        return 0; 
    }

    //Branch if overflow 1
    fn BVS(&mut self) -> u8{
        if self.get_flag(Flags::V) == 1 {
            self.cycles = self.cycles + 1;
            let wrapped_pc = Wrapping(self.pc);
            let wrapped_addr = Wrapping(self.addr_relative);
            self.addr_absolute = (wrapped_pc + wrapped_addr).0;

            if self.addr_absolute & 0xFF00 != self.pc & 0xFF00{
                self.cycles = self.cycles + 1;
            }
            self.pc = self.addr_absolute;
        }
        return 0;
    }

    //Clear carry flag
    fn CLC(&mut self) -> u8{
        self.set_flag(Flags::C, false);
        return 0;
    }

    //Clear decimal flag
    fn CLD(&mut self) -> u8{
        self.set_flag(Flags::D, false);
        return 0;
    }

    //Clear interrupt flag
    fn CLI(&mut self) -> u8{
        self.set_flag(Flags::I, false);
        return 0;
    }

    //Clear overflow flag
    fn CLV(&mut self) -> u8{
        self.set_flag(Flags::V, false);
        return 0;
    }

    //Compare Accumulator
    fn CMP(&mut self) -> u8{
        self.fetch();
        let tmp_a = Wrapping(self.a as u16);
        let tmp_fetched = Wrapping(self.fetched as u16);
        let tmp = (tmp_a - tmp_fetched).0;
        self.set_flag(Flags::C, self.a >= self.fetched);
        self.set_flag(Flags::Z, (tmp & 0x00FF) == 0x0000);
        self.set_flag(Flags::N, (tmp & 0x0080) > 0);
        return 0;
    }

    //Compare X register 
    fn CPX(&mut self) -> u8{
        self.fetch();
        let tmp_x = Wrapping(self.x as u16);
        let tmp_fetched = Wrapping(self.fetched as u16);
        let tmp = (tmp_x - tmp_fetched).0;
        self.set_flag(Flags::C, self.x >= self.fetched);
        self.set_flag(Flags::Z, (tmp & 0x00FF) == 0x0000);
        self.set_flag(Flags::N, (tmp & 0x0080) > 0);    //Check
        return 0;
    }

    //Compare Y register
    fn CPY(&mut self) -> u8{
        self.fetch();
        let tmp_y = Wrapping(self.y as u16);
        let tmp_fetched = Wrapping(self.fetched as u16);
        let tmp = (tmp_y - tmp_fetched).0;
        self.set_flag(Flags::C, self.y >= self.fetched);
        self.set_flag(Flags::Z, (tmp & 0x00FF) == 0x0000);
        self.set_flag(Flags::N, (tmp & 0x0080) > 0);    //Check
        return 0;
    }

    //Decrement value
    fn DEC(&mut self) -> u8{
        self.fetch();
        let tmp = self.fetched - 1;
        self.write(self.addr_absolute, &mut (tmp & 0x00FF));
        self.set_flag(Flags::Z, (tmp & 0x00FF) == 0x0000);
        self.set_flag(Flags::N, (tmp & 0x0080) > 0);    //Check
        return 0;
    }

    //Decrement X registe
    fn DEX(&mut self) -> u8{
        if self.x == 0{
            self.x = 255
        }else {
            self.x = self.x - 1;
        }
        self.set_flag(Flags::Z, self.x == 0x00);
        self.set_flag(Flags::N, (self.x & 0x80) > 0);   //Check
        return 0;
    }

    //Decrement Y register
    fn DEY(&mut self) -> u8{
        if self.y == 0{
            self.y = 255
        }else {
            self.y = self.y - 1;
        }
        
        self.set_flag(Flags::Z, self.y == 0x00);
        self.set_flag(Flags::N, (self.y & 0x80) > 0); //Check
        return 0;
    }


    //XOR 
    fn EOR(&mut self) -> u8{
        self.fetch();
        self.a = self.a ^ self.fetched;	
        self.set_flag(Flags::Z, self.a == 0x00);
        self.set_flag(Flags::N, (self.a & 0x80) > 0); //Check
        return 1;
    }

    //Increment value
    fn INC(&mut self) -> u8{
        self.fetch();
        let tmp = self.fetched + 1;
        self.write(self.addr_absolute, &mut (tmp & 0x00FF));
        self.set_flag(Flags::Z, (tmp & 0x00FF) == 0x0000);
        self.set_flag(Flags::N, (tmp & 0x0080) > 0);
        return 0;
    }

    //Increment X reg
    fn INX(&mut self) -> u8{
        if self.x == 255{
            self.x = 0;
        }else{
            self.x = self.x + 1;
        }
        self.set_flag(Flags::Z, self.x == 0x00);
        self.set_flag(Flags::N, (self.x & 0x80) > 0);
        return 0;
    }

    //Increment Y reg
    fn INY(&mut self) -> u8{
        if self.y == 255{
            self.y = 0
        }else {
            self.y = self.y + 1;
        }
        self.set_flag(Flags::Z, self.y == 0x00);
        self.set_flag(Flags::N, (self.y & 0x80) > 0);
        return 0;
    }

    //Jump
    fn JMP(&mut self) -> u8{
        self.pc = self.addr_absolute;
        return 0;
    }

    //Jump to sub-routine
    fn JSR(&mut self) -> u8{
        self.pc = self.pc - 1;

        self.write(0x0100 + self.sptr as u16, &mut ((self.pc >> 8) & 0x00FF).to_be_bytes()[1]);
        self.subtract_stack();
        self.write(0x0100 + self.sptr as u16, &mut (self.pc & 0x00FF).to_be_bytes()[1]);
        self.subtract_stack();
    
        self.pc = self.addr_absolute;
        return 0;
    }

    //Load accumulator 
    fn LDA(&mut self) -> u8{
        self.fetch();
        self.a = self.fetched;
        self.set_flag(Flags::Z, self.a == 0x00);
        self.set_flag(Flags::N, (self.a & 0x80) > 0);
        return 1;
    }

    //Load X reg
    fn LDX(&mut self) -> u8{
        self.fetch();
        self.x = self.fetched;
        self.set_flag(Flags::Z, self.x == 0x00);
        self.set_flag(Flags::N, (self.x & 0x80) > 0);
        return 1;
    }

    //Load Y reg
    fn LDY(&mut self) -> u8{
        self.fetch();
        self.y = self.fetched;
        self.set_flag(Flags::Z, self.y == 0x00);
        self.set_flag(Flags::N, (self.y & 0x80) > 0);
        return 1;
    }

    //Shift Right
    fn LSR(&mut self) -> u8{
        self.fetch();
        self.set_flag(Flags::C, (self.fetched & 0x0001) == 1);
        let tmp = self.fetched >> 1;	
        self.set_flag(Flags::Z, (tmp & 0x00FF) == 0x0000);
        self.set_flag(Flags::N, (tmp & 0x0080) > 0);
        if self.lookup[self.opcode as usize].addr_name == "IMP"{
            self.a = tmp & 0x00FF;
        }
        else{
            self.write(self.addr_absolute, &mut (tmp & 0x00FF));
        }
        return 0;
    }

    //No-Operation
    fn NOP(&mut self) -> u8{
        match self.opcode{
            0x1C => return 1,
            0x3C => return 1,
            0x5C => return 1,
            0x7C => return 1,
            0xDC => return 1,
            0xFC => return 1,
            _ => return 0,
        }
    }

    //Bitwise OR 
    fn ORA(&mut self) -> u8{
        self.fetch();
        self.a = self.a | self.fetched;
        self.set_flag(Flags::Z, self.a == 0x00);
        self.set_flag(Flags::N, (self.a & 0x80) > 0);
        return 1;
    }

    //Stack accumulator push
    fn PHA(&mut self) -> u8{
        let mut a = self.a;
        self.write(0x0100 + (self.sptr as u16), &mut a);
        self.subtract_stack();
        return 0;
    }

    //Stack register push 
    fn PHP(&mut self) -> u8{
        self.write(0x0100 + self.sptr as u16, &mut (self.sr | Flags::B.bits | Flags::U.bits));
        self.set_flag(Flags::B, false);
        self.set_flag(Flags::U, false);
        self.subtract_stack();
        return 0;
    }

    //Pop accumulator
    fn PLA(&mut self) -> u8{
        self.sptr = self.sptr + 1;
        let data = 0x0100 + self.sptr as u16;
        self.a = self.read(data);

        self.set_flag(Flags::Z, self.a == 0x000);
        self.set_flag(Flags::N, (self.a & 0x80) > 0);
        return 0;
    }

    //Pop Register
    fn PLP(&mut self) -> u8{
        self.sptr = self.sptr + 1;
        self.sr = self.read(0x0100 + self.sptr as u16);
        self.set_flag(Flags::U, true);
        return 0;
    }

    //Rotate left
    fn ROL(&mut self) -> u8{
        self.fetch();
        let tmp = (self.fetched << 1) as u16 | (self.get_flag(Flags::C)) as u16;
        self.set_flag(Flags::C, (tmp & 0xFF00) == 1);
        self.set_flag(Flags::Z, (tmp & 0x00FF) == 0x0000);
        self.set_flag(Flags::N, (tmp & 0x0080) > 0);
        if self.lookup[self.opcode as usize].addr_name == "IMP"{
            self.a = (tmp & 0x00FF).to_be_bytes()[1];
        }
        else{
            self.write(self.addr_absolute, &mut ((tmp & 0x00FF).to_be_bytes()[1]));

        }
        return 0;
    }

    //Rotate right
    fn ROR(&mut self) -> u8{
        self.fetch();
        let tmp = ((self.get_flag(Flags::C) << 7) | (self.fetched >> 1)) as u16;
        self.set_flag(Flags::C, (self.fetched & 0x01) == 1);
        self.set_flag(Flags::Z, (tmp & 0x00FF) == 0x00);
        self.set_flag(Flags::N, (tmp & 0x0080) > 0);
        if self.lookup[self.opcode as usize].addr_name == "IMP"{
            self.a = (tmp & 0x00FF).to_be_bytes()[1];
        }else{
            self.write(self.addr_absolute, &mut ((tmp & 0x00FF).to_be_bytes()[1]));

        }
        return 0;
    }

    //Return from interrupt
    fn RTI(&mut self) -> u8{
        self.sptr = self.sptr + 1;
        self.sr = self.read(0x0100 + self.sptr as u16);
        self.sr &= Flags::B.bits;
        self.sr &= Flags::U.bits;

        self.sptr = self.sptr + 1;
        let sptr = self.sptr as u16;
        self.pc = self.read(0x0100 + sptr) as u16;
        self.sptr = self.sptr + 1;
        let sptr = self.sptr as u16;
        self.pc |= (self.read(0x0100 + sptr as u16) as u16) << 8;

        return 0;
    }

    //Return from sub-routine
    fn RTS(&mut self) -> u8{
        self.sptr = self.sptr + 1;
        self.pc = self.read(0x0100 + self.sptr as u16) as u16;
        self.sptr = self.sptr + 1;
        self.pc |= (self.read(0x0100 + self.sptr as u16) as u16) << 8;
        
        self.pc = self.pc + 1;
        return 0;
    }

    //Set carry flag
    fn SEC(&mut self) -> u8{
        self.set_flag(Flags::C, true);
        return 0;
    }

    //Set decimal flag
    fn SED(&mut self) -> u8{
        self.set_flag(Flags::D, true);
        return 0;
    }

    //Set interrupt flag
    fn SEI(&mut self) -> u8{
        self.set_flag(Flags::I, true);
        return 0;
    }

    //Store accumulator 
    fn STA(&mut self) -> u8{
        let mut sr = self.a;
        self.write(self.addr_absolute, &mut sr);

        return 0;
    }

    //Store X 
    fn STX(&mut self) -> u8{
        let mut x = self.x;
        self.write(self.addr_absolute, &mut x);
        return 0;
    }

    //Store Y 
    fn STY(&mut self) -> u8{
        let mut y = self.y;
        self.write(self.addr_absolute, &mut y);
        return 0;
    }

    //Transfer accumulator to X
    fn TAX(&mut self) -> u8{
        self.x = self.a;
        self.set_flag(Flags::Z, self.x == 0x00);
        self.set_flag(Flags::N, (self.x & 0x80) > 0);
        
        return 0;
    }

    //Transfer accumulator to Y 
    fn TAY(&mut self) -> u8{
        self.y = self.a;
        self.set_flag(Flags::Z, self.y == 0x00);
        self.set_flag(Flags::N, (self.y & 0x80) > 0);

        return 0;
    }

    //Transfer stack ptr
    fn TSX(&mut self) -> u8{
        self.x = self.sptr;
        self.set_flag(Flags::Z, self.x == 0x00);
        self.set_flag(Flags::N, (self.x & 0x80) > 0);

        return 0;
    }

    //Transfer X reg to accumulator 
    fn TXA(&mut self) -> u8{
        self.a = self.x;
        self.set_flag(Flags::Z, self.a == 0x00);
        self.set_flag(Flags::N, (self.a & 0x80) > 0);
        return 0;
    }

    //Transfer X reg to stack 
    fn TXS(&mut self) -> u8{
        self.sptr = self.x;
        return 0;
    }

    //Transfer Y reg to accumulator 
    fn TYA(&mut self) -> u8{
        self.a = self.y;
	    self.set_flag(Flags::Z, self.a == 0x00);
	    self.set_flag(Flags::N, (self.a & 0x80) > 1);
        return 0;
    }

    //Illegal opcode
    fn ILL(&mut self) -> u8{
        return 0;
    }
}

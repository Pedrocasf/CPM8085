pub mod regs;
use regs::Registers;

use super::CPM;

#[macro_export]
macro_rules! getLsb{
    ($a:expr) => {
        !($a - 1) & $a
    }
}

pub struct CPU{
    lut:[fn(&mut CPU,&mut [u8]);0x100],
    pub regs:Registers,
}

impl CPU{
    pub fn new()->CPU{
        CPU{
            lut:[CPU::fault;0x100],
            regs:Registers::default(),
        }
    }
    pub fn next(&mut self,mem:&mut [u8]){
        print!("PC: {:04X} ", self.regs.PC);
        self.lut[mem[self.regs.PC as usize] as usize](self,mem);
        println!(" {:X?}",self.regs);
    }
    fn jmp(&mut self,mem:&mut [u8]){
        let lb = mem[self.regs.PC as usize +1];
        let hb = mem[self.regs.PC as usize +2];
        let addr = (hb as u16) << 8 | lb as u16;
        self.regs.PC = addr;
        print!("JMP {:04X}",addr);
    }
    fn lxi(&mut self,mem:&mut [u8]){
        let lb = mem[self.regs.PC as usize +1];
        let hb = mem[self.regs.PC as usize +2];
        let val = (hb as u16) << 8 | lb as u16;
        self.regs.setRP(val,mem[self.regs.PC as usize]);
        self.regs.PC+=3;
        print!("LXI {:04X}", val);
    }
    fn ani(&mut self,mem:&mut [u8]){
        let db = mem[self.regs.PC as usize +1];
        self.regs.A &= db;
        self.regs.set_flags(self.regs.A,false);
        self.regs.PC+=2;
        print!("ANI {:02X}", db);
    }
    fn jccc(&mut self,mem:&mut [u8]){
        let mut addr = 0;
        if self.regs.cond(mem[self.regs.PC as usize]){
            let lb = mem[self.regs.PC as usize +1];
            let hb = mem[self.regs.PC as usize +2];
            addr = (hb as u16) << 8 | lb as u16;
            self.regs.PC = addr;
        }else{
            self.regs.PC+=3;
        }
        print!("Jccc {:04X}",addr);
    }
    fn adi(&mut self,mem:&mut [u8]){
        let db = mem[self.regs.PC as usize +1];
        let (a,v) = self.regs.A.overflowing_add(db);
        self.regs.A = a;
        self.regs.set_flags(self.regs.A,v);
        self.regs.PC+=2;
        print!("ADI {:02X}", db);
    }
    fn call(&mut self,mem:&mut [u8]){
        mem[self.regs.SP as usize -1] = (self.regs.PC >> 8) as u8;
        mem[self.regs.SP as usize -2] = self.regs.PC as u8;
        self.regs.SP-=2;
        let lb = mem[self.regs.PC as usize +1];
        let hb = mem[self.regs.PC as usize +2];
        let addr = (hb as u16) << 8 | lb as u16;
        self.regs.PC = addr;
        print!("CALL {:04X}",addr);
    }
    fn push(&mut self,mem:&mut [u8]){
        let rp = self.regs.getRP(mem[self.regs.PC as usize]);
        mem[self.regs.SP as usize -1] = (rp >> 8) as u8;
        mem[self.regs.SP as usize -2] = rp as u8;
        self.regs.SP-=2;
        self.regs.PC+=1;
        print!("PUSH {:04X}",rp )
    }
    fn xchg(&mut self,_mem:&mut [u8]){
        let hl = self.regs.getRP(0x20);
        let de = self.regs.getRP(0x10);
        self.regs.setRP(hl,0x10);
        self.regs.setRP(de,0x20);
        self.regs.PC += 1;
        print!("XCHG {:04X}", de);
    }
    fn mvi(&mut self,mem:&mut [u8]){
        self.regs.setD(mem[self.regs.PC as usize], mem, mem[self.regs.PC as usize +1]);
        self.regs.PC+=2;
        print!("MVI {:02X}", mem[self.regs.PC as usize +1]);
    }
    fn nop(&mut self,_mem:&mut [u8]){
        self.regs.PC+=1;
        print!("NOP {:04X}", self.regs.PC);
    }
    fn fault(&mut self,mem:&mut [u8]){
        panic!("regs:{:x?}, instr:{:08b}, {:02x}",self.regs, mem[self.regs.PC as usize],mem[self.regs.PC as usize])
    }
    fn syscall(&mut self,mem:&mut [u8]){
        CPM::syscall(&CPM(self.regs.C,self.regs.PC),self,mem);
    }
    fn mov(&mut self,mem:&mut [u8]){
        let s = self.regs.getS(mem[self.regs.PC as usize], mem);
        self.regs.setD(mem[self.regs.PC as usize], mem, s);
        self.regs.PC+=1;
        print!("MOV {:02X}", s);
    }
    fn lda(&mut self,mem:&mut [u8]){
        let lb = mem[self.regs.PC as usize +1];
        let hb = mem[self.regs.PC as usize +2];
        let addr = (hb as u16) << 8 | lb as u16;
        self.regs.A = mem[addr as usize];
        self.regs.PC+=3;
        print!("LDA {:04X}", addr);
    }
    fn sda(&mut self,mem:&mut [u8]){
        let lb = mem[self.regs.PC as usize +1];
        let hb = mem[self.regs.PC as usize +2];
        let addr = (hb as u16) << 8 | lb as u16;
        mem[addr as usize] = self.regs.A ;
        self.regs.PC+=3;
        print!("SDA {:04X}", addr);
    }
    fn lhld(&mut self,mem:&mut [u8]){
        let lb = mem[self.regs.PC as usize +1];
        let hb = mem[self.regs.PC as usize +2];
        let addr = ((hb as u16) << 8 | lb as u16) as usize;
        let val = (mem[addr+1] as u16) << 8 | mem[addr] as u16;
        self.regs.setRP(val, 0x20);
        self.regs.PC+=3;
        print!("LHLD {:04X}", val);
    }
    fn shld(&mut self,mem:&mut [u8]){
        let lb = mem[self.regs.PC as usize +1];
        let hb = mem[self.regs.PC as usize +2];
        let addr = ((hb as u16) << 8 | lb as u16) as usize;
        let val = self.regs.getRP(0x20);
        mem[addr] = val as u8;
        mem[addr+1] = (val >> 8) as u8;
        self.regs.PC+=3;
        print!("LHLD {:04X}", val);
    }
    fn ldax(&mut self,mem:&mut [u8]){
        let rp = self.regs.getRP(mem[self.regs.PC as usize]);
        self.regs.A = mem[rp as usize];
        self.regs.PC+=1;
        print!("LDAX {:04X}", rp);
    }
    fn stax(&mut self,mem:&mut [u8]){
        let rp = self.regs.getRP(mem[self.regs.PC as usize]);
        mem[rp as usize] = self.regs.A;
        self.regs.PC+=1;
        print!("STAX {:04X}", rp);
    }
    fn add(&mut self,mem:&mut [u8]){
        let s = self.regs.getS(mem[self.regs.PC as usize], mem);
        let (a,v) = self.regs.A.overflowing_add(s);
        self.regs.set_flags(a, v);
        self.regs.A = a;
        self.regs.PC+=1;
        print!("ADD {:02X}",s);
    }
    fn adc(&mut self,mem:&mut [u8]){
        let s = self.regs.getS(mem[self.regs.PC as usize], mem);
        let (a0,v0) = self.regs.A.overflowing_add(s);
        let (a1,v1) = a0.overflowing_add(self.regs.F.get_carry() as u8);
        self.regs.A = a1;
        self.regs.set_flags(self.regs.A, v0|v1);
        self.regs.PC+=1;
        print!("ADC {:02X}",mem[self.regs.PC as usize +1]);
    }
    fn aci(&mut self,mem:&mut [u8]){
        let (a0,v0) = self.regs.A.overflowing_add(mem[self.regs.PC as usize +1]);
        let (a1,v1) = a0.overflowing_add(self.regs.F.get_carry() as u8);
        self.regs.A = a1;
        self.regs.set_flags(self.regs.A, v0|v1);
        self.regs.PC+=2;
        print!("ACI {:02X}",mem[self.regs.PC as usize +1]);
    }
    fn sub(&mut self,mem:&mut [u8]){
        let s = self.regs.getS(mem[self.regs.PC as usize], mem);
        let (a,v) = self.regs.A.overflowing_sub(s);
        self.regs.set_flags(a, v);
        self.regs.A = a;
        self.regs.PC+=1;
        print!("SUB {:02X}",s);
    }
    fn sui(&mut self,mem:&mut [u8]){
        let (a,v) = self.regs.A.overflowing_sub(mem[self.regs.PC as usize +1]);
        self.regs.A = a;
        self.regs.set_flags(self.regs.A, v);
        self.regs.PC+=2;
        print!("SUI {:02X}",mem[self.regs.PC as usize +1]);
    }
    fn sbb(&mut self,mem:&mut [u8]){
        let s = self.regs.getS(mem[self.regs.PC as usize], mem);
        let (a0,v0) = self.regs.A.overflowing_sub(s);
        let (a1,v1) = a0.overflowing_sub(self.regs.F.get_carry() as u8);
        self.regs.A = a1;
        self.regs.set_flags(self.regs.A, v0|v1);
        self.regs.PC+=1;
        print!("SBB {:02X}",mem[self.regs.PC as usize +1]);
    }
    fn sbi(&mut self,mem:&mut [u8]){
        let (a0,v0) = self.regs.A.overflowing_sub(mem[self.regs.PC as usize +1]);
        let (a1,v1) = a0.overflowing_sub(self.regs.F.get_carry() as u8);
        self.regs.A = a1;
        self.regs.set_flags(self.regs.A, v0|v1);
        self.regs.PC+=2;
        print!("SBI {:02X}",mem[self.regs.PC as usize +1]);
    }
    fn inr(&mut self,mem:&mut [u8]){
        let r = self.regs.getD(mem[self.regs.PC as usize], mem).wrapping_add(1);
        self.regs.setD(mem[self.regs.PC as usize], mem, r);
        self.regs.set_flags(r, false);
        self.regs.PC+=1;
        print!("INR {:02X}", r);
    }
    fn dcr(&mut self,mem:&mut [u8]){
        let r = self.regs.getD(mem[self.regs.PC as usize], mem).wrapping_sub(1);
        self.regs.setD(mem[self.regs.PC as usize], mem, r);
        self.regs.set_flags(r, false);
        self.regs.PC+=1;
        print!("DCR {:02X}", r);
    }
    fn inx(&mut self,mem:&mut [u8]){
        let rp = self.regs.getRP(mem[self.regs.PC as usize]);
        self.regs.setRP(rp.wrapping_add(1), mem[self.regs.PC as usize]);
        self.regs.PC+=1;
        print!("INX {:02x}", rp);
    }
    fn dcx(&mut self,mem:&mut [u8]){
        let rp = self.regs.getRP(mem[self.regs.PC as usize]);
        self.regs.setRP(rp.wrapping_sub(1), mem[self.regs.PC as usize]);
        self.regs.PC+=1;
        print!("DCX {:02x}", rp);
    }
    fn dad(&mut self,mem:&mut [u8]){
        let rp = self.regs.getRP(mem[self.regs.PC as usize]);
        let hl = self.regs.getRP(0x20);
        let (hl,v) = hl.overflowing_add(rp);
        self.regs.setRP(hl,0x20);
        self.regs.F.set_carry(v);
        self.regs.PC+=1;
        print!("DAD {:04x}", hl);
    }
    fn daa(&mut self,mem:&mut [u8]){
        panic!("DAA");
    }
    fn ana(&mut self,mem:&mut [u8]){
        let s = self.regs.getS(mem[self.regs.PC as usize], mem);
        self.regs.A&=s;
        self.regs.set_flags(self.regs.A, false);
        self.regs.PC+=1;
        print!("ANA {:02X}",s);
    }
    fn ora(&mut self,mem:&mut [u8]){
        let s = self.regs.getS(mem[self.regs.PC as usize], mem);
        self.regs.A|=s;
        self.regs.set_flags(self.regs.A, false);
        self.regs.PC+=1;
        print!("ORA {:02X}",s);
    }
    fn ori(&mut self,mem:&mut [u8]){
        self.regs.A|=mem[self.regs.PC as usize +1];
        self.regs.set_flags(self.regs.A, false);
        self.regs.PC+=2;
        print!("ORI {:02X}",mem[self.regs.PC as usize +1]);
    }
    fn xra(&mut self,mem:&mut [u8]){
        let s = self.regs.getS(mem[self.regs.PC as usize], mem);
        self.regs.A^=s;
        self.regs.set_flags(self.regs.A, false);
        self.regs.PC+=1;
        print!("XRA {:02X}",s);
    }
    fn xri(&mut self,mem:&mut [u8]){
        self.regs.A^=mem[self.regs.PC as usize +1];
        self.regs.set_flags(self.regs.A, false);
        self.regs.PC+=2;
        print!("XRI {:02X}",mem[self.regs.PC as usize +1]);
    }
    fn cmp(&mut self,mem:&mut [u8]){
        let s = self.regs.getS(mem[self.regs.PC as usize], mem);
        let (a, v) = self.regs.A.overflowing_sub(s);
        self.regs.set_flags(a,v);
        self.regs.PC+=1;
        print!("CMP {:02X}", mem[self.regs.PC as usize +1]);
    }
    fn cpi(&mut self,mem:&mut [u8]){
        let (a, v) = self.regs.A.overflowing_sub(mem[self.regs.PC as usize +1]);
        self.regs.set_flags(a,v);
        self.regs.PC+=2;
        print!("CPI {:02X}", mem[self.regs.PC as usize +1]);
    }
    fn rlc(&mut self,mem:&mut [u8]){
        let (a,c) = self.regs.A.overflowing_mul(2);
        self.regs.A = a+c as u8;
        self.regs.F.set_carry(c);
        self.regs.PC+=1;
        print!("RLC {:02x}", a);
    }
    fn rrc(&mut self,mem:&mut [u8]){
        let a = self.regs.A.rotate_right(1);
        self.regs.A = a;
        self.regs.F.set_carry(a & 0x80 == 0x80);
        self.regs.PC+=1;
        print!("RRC {:02x}", a);
    }
    fn ral(&mut self,mem:&mut [u8]){
        let (a,c) = self.regs.A.overflowing_mul(2);
        self.regs.A = a+self.regs.F.get_carry() as u8;
        self.regs.F.set_carry(c);
        self.regs.PC+=1;
        print!("RAL {:02x}", a);
    }
    fn rar(&mut self,mem:&mut [u8]){
        self.regs.F.set_carry(self.regs.A & 1 == 1);
        let a = (self.regs.A as i8) >>1;
        self.regs.A = a as u8;
        self.regs.PC+=1;
        print!("RAL {:02x}", a);
    }
    fn cma(&mut self,mem:&mut [u8]){
        self.regs.A = !self.regs.A;
        self.regs.PC+=1;
        print!("CMA {:02x}", self.regs.A);
    }
    fn cmc(&mut self,mem:&mut [u8]){
        let c = !self.regs.F.get_carry();
        self.regs.F.set_carry(c);
        self.regs.PC+=1;
        print!("CMC {}", self.regs.F.get_carry());
    }
    fn stc(&mut self,mem:&mut [u8]){
        self.regs.F.set_carry(true);
        self.regs.PC+=1;
        print!("STC");
    }
    fn Cccc(&mut self,mem:&mut [u8]){
        let mut addr = 0;
        if self.regs.cond(mem[self.regs.PC as usize]){
            mem[self.regs.SP as usize -1] = (self.regs.PC >> 8) as u8;
            mem[self.regs.SP as usize -2] = self.regs.PC as u8;
            self.regs.SP-=2;
            let lb = mem[self.regs.PC as usize +1];
            let hb = mem[self.regs.PC as usize +2];
            addr = (hb as u16) << 8 | lb as u16;
            self.regs.PC = addr;
        }else{
            self.regs.PC+=3;
        }
        print!("Cccc {:04X}",addr);
    }
    pub fn ret(&mut self,mem:&mut [u8]){
        let lb = mem[self.regs.SP as usize];
        let hb = mem[self.regs.SP as usize +1];
        self.regs.SP+=2;
        let addr = (hb as u16) << 8 | lb as u16;
        self.regs.PC = addr + 3;
        print!("RET {:04X}",addr);
    }
    fn Rccc(&mut self,mem:&mut [u8]){
        let mut addr = 0;
        if self.regs.cond(mem[self.regs.PC as usize]){
            let lb = mem[self.regs.SP as usize];
            let hb = mem[self.regs.SP as usize +1];
            self.regs.SP+=2;
            addr = (hb as u16) << 8 | lb as u16;
            self.regs.PC = addr + 3;
        }else{
            self.regs.PC+=1;
        }
        print!("Rccc {:04X}",addr);
    }
    fn rst(&mut self,mem:&mut [u8]){panic!("")}
    fn pchl(&mut self,mem:&mut [u8]){
        print!("PCHL {:04x}", self.regs.PC);
        self.regs.PC = self.regs.getRP(0x20);
    }
    fn pop(&mut self,mem:&mut [u8]){
        let lb = mem[self.regs.SP as usize];
        let hb = mem[self.regs.SP as usize +1];
        self.regs.SP+=2;
        let val = (hb as u16) << 8 | lb as u16;
        self.regs.setRP(val, mem[self.regs.PC as usize]);
        self.regs.PC+=1;
        print!("POP {:04x}", val);
    }
    fn xthl(&mut self,mem:&mut [u8]){
        let l = self.regs.L;
        let h = self.regs.H;
        self.regs.L = mem[self.regs.SP as usize];
        self.regs.H = mem[self.regs.SP as usize +1];
        mem[self.regs.SP as usize]=l;
        mem[self.regs.SP as usize +1]=h;
        self.regs.PC+=1;
        print!("XTHL {:04x}", self.regs.getRP(0x20));
    }
    fn sphl(&mut self,mem:&mut [u8]){
        self.regs.SP = self.regs.getRP(0x20);
        self.regs.PC+=1;
        print!("SPHL {:04x}", self.regs.getRP(0x20));
    }
    fn r#in(&mut self,mem:&mut [u8]){panic!("")}
    fn out(&mut self,mem:&mut [u8]){panic!("")}
    fn ei(&mut self,mem:&mut [u8]){panic!("")}
    fn di(&mut self,mem:&mut [u8]){panic!("")}
    fn hlt(&mut self,mem:&mut [u8]){panic!("")}
    pub fn index(&mut self) {
        self.regs.SP = 0xFFFF;
        self.regs.PC = 0x100;
        for i in INDEX.iter(){
            self.place(i.0, i.1);
        };
    }
    fn place(&mut self,s:&str, v:fn(&mut CPU,&mut [u8])){
        let mut xmask:u8 = 0;
        let mut kmask:u8  = 0;
        for (i, c) in s.chars().enumerate() {
            match c{
                '0' => {},
                '1' => kmask |= 1<<(7-i),
                'N'|'D'|'R'|'P'|'S'|'C' => xmask |= 1<<(7-i),
                _ => panic!("worng pattern"),
            }        
        }
        //print!("kmask:{:04X}", kmask);
        //print!("xmask:{:04X}", xmask);
        self.recursive(kmask,xmask,v);
    }
    fn recursive(&mut self,kmask:u8,xmask:u8,val:fn(&mut CPU,&mut [u8])){
        if xmask == 0{
            //print!("kmask:{:03X}", kmask);
            self.lut[kmask as usize] = val;
        }else{
            let xmask_lsb = getLsb!(xmask);
            let xmask_without_lsb = xmask & !xmask_lsb;
            self.recursive(kmask, xmask_without_lsb, val);
            self.recursive(kmask | xmask_lsb, xmask_without_lsb, val);
        }
    }
}
const INDEX:[(&str,fn(&mut CPU,&mut [u8]));58] = [
("00001000",CPU::syscall),
("01DDDSSS",CPU::mov),
("00DDD110",CPU::mvi),
("00RP0001",CPU::lxi),
("00RP1010",CPU::ldax),
("00RP0010",CPU::stax),
("00111010",CPU::lda),
("00110010",CPU::sda),
("00101010",CPU::lhld),
("00100010",CPU::shld),
("11101011",CPU::xchg),
("10000SSS",CPU::add),
("11000110",CPU::adi),
("10001SSS",CPU::adc),
("11001110",CPU::aci),
("10010SSS",CPU::sub),
("11010110",CPU::sui),
("10011SSS",CPU::sbb),
("11011110",CPU::sbi),
("00DDD100",CPU::inr),
("00DDD101",CPU::dcr),
("00RP0011",CPU::inx),
("00RP1011",CPU::dcx),
("00RP1001",CPU::dad),
("00100111",CPU::daa),
("10100SSS",CPU::ana),
("11100110",CPU::ani),
("10110SSS",CPU::ora),
("11110110",CPU::ori),
("10101SSS",CPU::xra),
("11101110",CPU::xri),
("10111SSS",CPU::cmp),
("11111110",CPU::cpi),
("00000111",CPU::rlc),
("00001111",CPU::rrc),
("00010111",CPU::ral),
("00011111",CPU::rar),
("00101111",CPU::cma),
("00111111",CPU::cmc),
("00110111",CPU::stc),
("11000011",CPU::jmp),
("11CCC010",CPU::jccc),
("11001101",CPU::call),
("11CCC100",CPU::Cccc),
("11001001",CPU::ret),
("11CCC000",CPU::Rccc),
("11NNN111",CPU::rst),
("11101001",CPU::pchl),
("11RP0101",CPU::push),
("11RP0001",CPU::pop),
("11100011",CPU::xthl),
("11111001",CPU::sphl),
("11011011",CPU::r#in),
("11010011",CPU::out),
("11111011",CPU::ei),
("11110011",CPU::di),
("01110110",CPU::hlt),
("00000000",CPU::nop),
];
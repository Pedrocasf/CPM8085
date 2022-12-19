pub mod regs;
use regs::Registers;
use log::{trace, error};
#[macro_export]
macro_rules! getLsb{
    ($a:expr) => {
        !($a - 1) & $a
    }
}
pub struct CPU{
    i:u8,
    regs:Registers,
    syscall:bool
}
impl CPU{
    pub fn new()->CPU{
        let mut cpu = CPU{
            i:0,
            regs:Registers::default(),
            syscall:false
        };
        cpu.regs.pc = 0x100;
        cpu.regs.sp = u16::MAX;
        cpu
    }
    pub fn get_syscall(&self)->bool{
        self.syscall
    }
    pub fn syscall_clear(&mut self){
        self.syscall = false;
    }
    pub fn get_regs(&self) -> Registers{
        self.regs
    }
    fn get_16(&self, mem:&[u8])->u16{
        let lb = mem[self.regs.pc as usize +1];
        let hb = mem[self.regs.pc as usize +2];
        (hb as u16) << 8 | lb as u16
    }
    fn pop_16(&mut self, mem:&[u8])->u16{
        let lb = mem[self.regs.sp as usize];
        let hb = mem[self.regs.sp as usize +1];
        self.regs.sp+=2;
        (hb as u16) << 8 | lb as u16
    }
    pub fn next(&mut self,mem:&mut [u8]){
        trace!("PC: {:04X} ", self.regs.pc);
        self.i = mem[self.regs.pc as usize];
        LUT[self.i as usize](self,mem);
        trace!("{:X?}\n",self.regs);
    }
    fn jmp(&mut self,mem:&mut [u8]){
        let addr = self.get_16(mem);
        self.regs.pc = addr;
        trace!("JMP {:04X}",addr);
    }
    fn lxi(&mut self,mem:&mut [u8]){
        let val = self.get_16(mem);
        self.regs.set_rp(val,self.i);
        self.regs.pc+=3;
        trace!("LXI {:04X}", val);
    }
    fn ani(&mut self,mem:&mut [u8]){
        let db = mem[self.regs.pc as usize +1];
        self.regs.a &= db;
        self.regs.set_flags(self.regs.a,false,false);
        self.regs.pc+=2;
        trace!("ANI {:02X}", db);
    }
    fn jccc(&mut self,mem:&mut [u8]){
        let mut addr = 0;
        if self.regs.cond(self.i){
            addr = self.get_16(mem);
            self.regs.pc = addr;
        }else{
            self.regs.pc+=3;
        }
        trace!("Jccc {:04X}",addr);
    }
    fn adi(&mut self,mem:&mut [u8]){
        let db = mem[self.regs.pc as usize +1];
        let (a,v) = self.regs.a.overflowing_add(db);
        let h = ((self.regs.a & 0xF) + (db & 0xF))& 0x10 == 0x10;
        self.regs.a = a;
        self.regs.set_flags(self.regs.a,v,h);
        self.regs.pc+=2;
        trace!("ADI {:02X}", db);
    }
    fn call(&mut self,mem:&mut [u8]){
        mem[self.regs.sp as usize -1] = (self.regs.pc >> 8) as u8;
        mem[self.regs.sp as usize -2] = self.regs.pc as u8;
        self.regs.sp-=2;
        let addr = self.get_16(mem);
        self.regs.pc = addr;
        trace!("CALL {:04X}",addr);
    }
    fn push(&mut self,mem:&mut [u8]){
        let rp = self.regs.get_rp(self.i);
        mem[self.regs.sp as usize -1] = (rp >> 8) as u8;
        mem[self.regs.sp as usize -2] = rp as u8;
        self.regs.sp-=2;
        self.regs.pc+=1;
        trace!("PUSH {:04X}",rp )
    }
    fn xchg(&mut self,_mem:&mut [u8]){
        let hl = self.regs.get_rp(0x20);
        let de = self.regs.get_rp(0x10);
        self.regs.set_rp(hl,0x10);
        self.regs.set_rp(de,0x20);
        self.regs.pc += 1;
        trace!("XCHG {:04X}", de);
    }
    fn mvi(&mut self,mem:&mut [u8]){
        self.regs.set_d(self.i, mem, mem[self.regs.pc as usize +1]);
        self.regs.pc+=2;
        trace!("MVI {:02X}", mem[self.regs.pc as usize +1]);
    }
    fn nop(&mut self,_mem:&mut [u8]){
        self.regs.pc+=1;
        trace!("NOP {:04X}", self.regs.pc);
    }
    fn fault(&mut self,_mem:&mut [u8]){
        error!("regs:{:x?}, instr:{:08b}, {:02x}",self.regs, self.i,self.i)
    }
    fn syscall(&mut self,_mem:&mut [u8]){
        self.syscall = true;
    }
    fn mov(&mut self,mem:&mut [u8]){
        let s = self.regs.get_s(self.i, mem);
        self.regs.set_d(self.i, mem, s);
        self.regs.pc+=1;
        trace!("MOV {:02X}", s);
    }
    fn lda(&mut self,mem:&mut [u8]){
        let addr = self.get_16(mem);
        self.regs.a = mem[addr as usize];
        self.regs.pc+=3;
        trace!("LDA {:04X}", addr);
    }
    fn sda(&mut self,mem:&mut [u8]){
        let addr = self.get_16(mem);
        mem[addr as usize] = self.regs.a ;
        self.regs.pc+=3;
        trace!("SDA {:04X}", addr);
    }
    fn lhld(&mut self,mem:&mut [u8]){
        let addr = self.get_16(mem) as usize;
        let val = (mem[addr+1] as u16) << 8 | mem[addr] as u16;
        self.regs.set_rp(val, 0x20);
        self.regs.pc+=3;
        trace!("LHLD {:04X}", val);
    }
    fn shld(&mut self,mem:&mut [u8]){
        let addr = self.get_16(mem) as usize;
        let val = self.regs.get_rp(0x20);
        mem[addr] = val as u8;
        mem[addr+1] = (val >> 8) as u8;
        self.regs.pc+=3;
        trace!("LHLD {:04X}", val);
    }
    fn ldax(&mut self,mem:&mut [u8]){
        let rp = self.regs.get_rp(self.i);
        self.regs.a = mem[rp as usize];
        self.regs.pc+=1;
        trace!("LDAX {:04X}", rp);
    }
    fn stax(&mut self,mem:&mut [u8]){
        let rp = self.regs.get_rp(self.i);
        mem[rp as usize] = self.regs.a;
        self.regs.pc+=1;
        trace!("STAX {:04X}", rp);
    }
    fn add(&mut self,mem:&mut [u8]){
        let s = self.regs.get_s(self.i, mem);
        let (a,v) = self.regs.a.overflowing_add(s);
        let h = ((self.regs.a & 0xF) + (s & 0xF))& 0x10 == 0x10;
        self.regs.set_flags(a, v, h);
        self.regs.a = a;
        self.regs.pc+=1;
        trace!("ADD {:02X}",s);
    }
    fn adc(&mut self,mem:&mut [u8]){
        let s = self.regs.get_s(self.i, mem);
        let (a0,v0) = self.regs.a.overflowing_add(s);
        let (a1,v1) = a0.overflowing_add(self.regs.f.get_carry() as u8);
        let h = ((self.regs.a & 0xF) + (s & 0xF) + self.regs.f.get_carry() as u8)& 0x10 == 0x10;
        self.regs.a = a1;
        self.regs.set_flags(self.regs.a, v0|v1, h);
        self.regs.pc+=1;
        trace!("ADC {:02X}",mem[self.regs.pc as usize +1]);
    }
    fn aci(&mut self,mem:&mut [u8]){
        let s = mem[self.regs.pc as usize +1];
        let (a0,v0) = self.regs.a.overflowing_add(s);
        let (a1,v1) = a0.overflowing_add(self.regs.f.get_carry() as u8);
        let h = ((self.regs.a & 0xF) + (s & 0xF) + self.regs.f.get_carry() as u8)& 0x10 == 0x10;
        self.regs.a = a1;
        self.regs.set_flags(self.regs.a, v0|v1, h);
        self.regs.pc+=2;
        trace!("ACI {:02X}",mem[self.regs.pc as usize +1]);
    }
    fn sub(&mut self,mem:&mut [u8]){
        let s = self.regs.get_s(self.i, mem);
        let (a,v) = self.regs.a.overflowing_sub(s);
        let h = (self.regs.a & 0xF).wrapping_sub(s & 0xF)& 0x10 == 0x10;
        self.regs.set_flags(a, v, h);
        self.regs.a = a;
        self.regs.pc+=1;
        trace!("SUB {:02X}",s);
    }
    fn sui(&mut self,mem:&mut [u8]){
        let s = mem[self.regs.pc as usize +1];
        let (a,v) = self.regs.a.overflowing_sub(s);
        let h = ((self.regs.a & 0xF) + (s & 0xF))& 0x10 == 0x10;
        self.regs.a = a;
        self.regs.set_flags(self.regs.a, v, h);
        self.regs.pc+=2;
        trace!("SUI {:02X}",mem[self.regs.pc as usize +1]);
    }
    fn sbb(&mut self,mem:&mut [u8]){
        let s = self.regs.get_s(self.i, mem);
        let (a0,v0) = self.regs.a.overflowing_sub(s);
        let (a1,v1) = a0.overflowing_sub(self.regs.f.get_carry() as u8);
        let h = (self.regs.a & 0xF).wrapping_sub(s & 0xF).wrapping_sub(self.regs.f.get_carry() as u8)& 0x10 == 0x10;
        self.regs.a = a1;
        self.regs.set_flags(self.regs.a, v0|v1,h);
        self.regs.pc+=1;
        trace!("SBB {:02X}",mem[self.regs.pc as usize +1]);
    }
    fn sbi(&mut self,mem:&mut [u8]){
        let s = mem[self.regs.pc as usize +1];
        let (a0,v0) = self.regs.a.overflowing_sub(s);
        let (a1,v1) = a0.overflowing_sub(self.regs.f.get_carry() as u8);
        let h = (self.regs.a & 0xF).wrapping_sub(s & 0xF).wrapping_sub(self.regs.f.get_carry() as u8)& 0x10 == 0x10;
        self.regs.a = a1;
        self.regs.set_flags(self.regs.a, v0|v1,h);
        self.regs.pc+=2;
        trace!("SBI {:02X}",mem[self.regs.pc as usize +1]);
    }
    fn inr(&mut self,mem:&mut [u8]){
        let r = self.regs.get_d(self.i, mem);
        let i = r.wrapping_add(1);
        let h = ((r & 0xF)+1)& 0x10 == 0x10;
        self.regs.set_d(self.i, mem, i);
        self.regs.set_flags(r, false, h);
        self.regs.pc+=1;
        trace!("INR {:02X}", r);
    }
    fn dcr(&mut self,mem:&mut [u8]){
        let r = self.regs.get_d(self.i, mem);
        let i = r.wrapping_sub(1);
        let h = ((r & 0xF).wrapping_sub(1))& 0x10 == 0x10;
        self.regs.set_d(self.i, mem, i);
        self.regs.set_flags(r, false, h);
        self.regs.pc+=1;
        trace!("DCR {:02X}", r);
    }
    fn inx(&mut self,_mem:&mut [u8]){
        let rp = self.regs.get_rp(self.i);
        self.regs.set_rp(rp.wrapping_add(1), self.i);
        self.regs.pc+=1;
        trace!("INX {:02x}", rp);
    }
    fn dcx(&mut self,_mem:&mut [u8]){
        let rp = self.regs.get_rp(self.i);
        self.regs.set_rp(rp.wrapping_sub(1), self.i);
        self.regs.pc+=1;
        trace!("DCX {:02x}", rp);
    }
    fn dad(&mut self,_mem:&mut [u8]){
        let rp = self.regs.get_rp(self.i);
        let hl = self.regs.get_rp(0x20);
        let (hl,v) = hl.overflowing_add(rp);
        self.regs.set_rp(hl,0x20);
        self.regs.f.set_carry(v);
        self.regs.pc+=1;
        trace!("DAD {:04x}", hl);
    }
    fn daa(&mut self,_mem:&mut [u8]){
        error!("DAA");
    }
    fn ana(&mut self,mem:&mut [u8]){
        let s = self.regs.get_s(self.i, mem);
        self.regs.a&=s;
        self.regs.set_flags(self.regs.a, false, false);
        self.regs.pc+=1;
        trace!("ANA {:02X}",s);
    }
    fn ora(&mut self,mem:&mut [u8]){
        let s = self.regs.get_s(self.i, mem);
        self.regs.a|=s;
        self.regs.set_flags(self.regs.a, false, false);
        self.regs.pc+=1;
        trace!("ORA {:02X}",s);
    }
    fn ori(&mut self,mem:&mut [u8]){
        self.regs.a|=mem[self.regs.pc as usize +1];
        self.regs.set_flags(self.regs.a, false, false);
        self.regs.pc+=2;
        trace!("ORI {:02X}",mem[self.regs.pc as usize +1]);
    }
    fn xra(&mut self,mem:&mut [u8]){
        let s = self.regs.get_s(self.i, mem);
        self.regs.a^=s;
        self.regs.set_flags(self.regs.a, false, false);
        self.regs.pc+=1;
        trace!("XRA {:02X}",s);
    }
    fn xri(&mut self,mem:&mut [u8]){
        self.regs.a^=mem[self.regs.pc as usize +1];
        self.regs.set_flags(self.regs.a, false, false);
        self.regs.pc+=2;
        trace!("XRI {:02X}",mem[self.regs.pc as usize +1]);
    }
    fn cmp(&mut self,mem:&mut [u8]){
        let s = self.regs.get_s(self.i, mem);
        let h = (self.regs.a & 0xF).wrapping_sub(s)& 0x10 == 0x10;
        let (a, v) = self.regs.a.overflowing_sub(s);
        self.regs.set_flags(a,v,h);
        self.regs.pc+=1;
        trace!("CMP {:02X}", mem[self.regs.pc as usize +1]);
    }
    fn cpi(&mut self,mem:&mut [u8]){
        let s = mem[self.regs.pc as usize +1];
        let h = (self.regs.a & 0xF).wrapping_sub(s)& 0x10 == 0x10;
        let (a, v) = self.regs.a.overflowing_sub(s);
        self.regs.set_flags(a,v,h);
        self.regs.pc+=2;
        trace!("CPI {:02X}", mem[self.regs.pc as usize +1]);
    }
    fn rlc(&mut self,_mem:&mut [u8]){
        let (a,c) = self.regs.a.overflowing_mul(2);
        self.regs.a = a+c as u8;
        self.regs.f.set_carry(c);
        self.regs.pc+=1;
        trace!("RLC {:02x}", a);
    }
    fn rrc(&mut self,_mem:&mut [u8]){
        let a = self.regs.a.rotate_right(1);
        self.regs.a = a;
        self.regs.f.set_carry(a & 0x80 == 0x80);
        self.regs.pc+=1;
        trace!("RRC {:02x}", a);
    }
    fn ral(&mut self,_mem:&mut [u8]){
        let (a,c) = self.regs.a.overflowing_mul(2);
        self.regs.a = a+self.regs.f.get_carry() as u8;
        self.regs.f.set_carry(c);
        self.regs.pc+=1;
        trace!("RAL {:02x}", a);
    }
    fn rar(&mut self,_mem:&mut [u8]){
        self.regs.f.set_carry(self.regs.a & 1 == 1);
        let a = (self.regs.a as i8) >>1;
        self.regs.a = a as u8;
        self.regs.pc+=1;
        trace!("RAL {:02x}", a);
    }
    fn cma(&mut self,_mem:&mut [u8]){
        self.regs.a = !self.regs.a;
        self.regs.pc+=1;
        trace!("CMA {:02x}", self.regs.a);
    }
    fn cmc(&mut self,_mem:&mut [u8]){
        let c = !self.regs.f.get_carry();
        self.regs.f.set_carry(c);
        self.regs.pc+=1;
        trace!("CMC {}", self.regs.f.get_carry());
    }
    fn stc(&mut self,_mem:&mut [u8]){
        self.regs.f.set_carry(true);
        self.regs.pc+=1;
        trace!("STC");
    }
    fn c_ccc(&mut self,mem:&mut [u8]){
        let mut addr = 0;
        if self.regs.cond(self.i){
            mem[self.regs.sp as usize -1] = (self.regs.pc >> 8) as u8;
            mem[self.regs.sp as usize -2] = self.regs.pc as u8;
            self.regs.sp-=2;
            addr = self.get_16(mem);
            self.regs.pc = addr;
        }else{
            self.regs.pc+=3;
        }
        trace!("Cccc {:04X}",addr);
    }
    pub fn ret(&mut self,mem:&mut [u8]){
        let addr = self.pop_16(mem);
        self.regs.pc = addr + 3;
        trace!("RET {:04X}",addr);
    }
    fn r_ccc(&mut self,mem:&mut [u8]){
        let mut addr = 0;
        if self.regs.cond(self.i){
            addr = self.pop_16(mem);
            self.regs.pc = addr + 3;
        }else{
            self.regs.pc+=1;
        }
        trace!("Rccc {:04X}",addr);
    }
    fn rst(&mut self,_mem:&mut [u8]){error!("")}
    fn pchl(&mut self,_mem:&mut [u8]){
        trace!("PCHL {:04x}", self.regs.pc);
        self.regs.pc = self.regs.get_rp(0x20);
    }
    fn pop(&mut self,mem:&mut [u8]){
        let val = self.pop_16(mem);
        self.regs.set_rp(val, self.i);
        self.regs.pc+=1;
        trace!("POP {:04x}", val);
    }
    fn xthl(&mut self,mem:&mut [u8]){
        let l = self.regs.l;
        let h = self.regs.h;
        self.regs.l = mem[self.regs.sp as usize];
        self.regs.h = mem[self.regs.sp as usize +1];
        mem[self.regs.sp as usize]=l;
        mem[self.regs.sp as usize +1]=h;
        self.regs.pc+=1;
        trace!("XTHL {:04x}", self.regs.get_rp(0x20));
    }
    fn sphl(&mut self,_mem:&mut [u8]){
        self.regs.sp = self.regs.get_rp(0x20);
        self.regs.pc+=1;
        trace!("SPHL {:04x}", self.regs.get_rp(0x20));
    }
    fn r#in(&mut self,_mem:&mut [u8]){error!("")}
    fn out(&mut self,_mem:&mut [u8]){error!("")}
    fn ei(&mut self,_mem:&mut [u8]){error!("")}
    fn di(&mut self,_mem:&mut [u8]){error!("")}
    fn hlt(&mut self,_mem:&mut [u8]){error!("")}
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
("11CCC100",CPU::c_ccc),
("11001001",CPU::ret),
("11CCC000",CPU::r_ccc),
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
const LUT:[fn(&mut CPU,&mut [u8]);0x100] = index();
const fn recursive(lut: &mut [fn(&mut CPU,&mut [u8]);0x100],kmask:u8,xmask:u8,val:fn(&mut CPU,&mut [u8])){
    if xmask == 0{
        //trace!("kmask:{:03X}", kmask);
        lut[kmask as usize] = val;
    }else{
        let xmask_lsb = getLsb!(xmask);
        let xmask_without_lsb = xmask & !xmask_lsb;
        recursive(lut,kmask, xmask_without_lsb, val);
        recursive(lut,kmask | xmask_lsb, xmask_without_lsb, val);
    }
}
const fn place(lut: &mut [fn(&mut CPU,&mut [u8]);0x100],s:&str, v:fn(&mut CPU,&mut [u8])){
    let mut xmask:u8 = 0;
    let mut kmask:u8  = 0;
    let mut i = 0;
    let s = s.as_bytes();
    while i < s.len(){
        let c = s[i] as char;
        match c{
            '0' => {},
            '1' => kmask |= 1<<(7-i),
            'N'|'D'|'R'|'P'|'S'|'C' => xmask |= 1<<(7-i),
            _ => {}
        }    
        i = i + 1;
    }
    recursive(lut,kmask,xmask,v);
}

const fn index() ->[fn(&mut CPU,&mut [u8]);0x100]{
    let mut lut:[fn(&mut CPU,&mut [u8]);0x100] = [CPU::fault;0x100];
    let mut i = 0;
    while i < INDEX.len(){
        place(&mut lut,INDEX[i].0,INDEX[i].1);
        i = i + 1;
    }
    return lut;
}



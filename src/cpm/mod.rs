use crate::CPU;
#[derive(Clone,Copy,Debug,Default)]
pub struct CPM(pub u8);
impl CPM{
    pub fn syscall(&self,cpu:&mut CPU, mem:&mut [u8]){
        match self.0{
            0x09 => self.c_writestr(cpu,mem),
            0x02 => self.c_write(cpu,mem),
            _ => panic!("regs:{:x?}, instr:{:08b}, {:02x}",cpu.regs, mem[cpu.regs.pc as usize],mem[cpu.regs.pc as usize]),
        }
        cpu.ret(mem);
    }
    fn c_writestr(&self,cpu:&mut CPU, mem:&[u8]){
        let off = cpu.regs.get_rp(0x10);
        let mut c:char = ' ';
        let mut count = 0;
        println!("");
        while c != '$'{
            c=mem[off as usize + 3 +count] as char;
            print!("{}",c);
            count +=1;
        }
        println!("");
    }
    fn c_write(&self, cpu:&mut CPU, _mem:&mut [u8]){
        let c = cpu.regs.e as char;
        println!("{}", c);
    }
}

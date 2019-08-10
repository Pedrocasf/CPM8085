use super::CPU;
pub struct CPM(pub u8);
impl CPM{
    pub fn syscall(&self,cpu:&mut CPU, mem:&mut [u8]){
        match self.0{
            0x0009 => self.print_string(cpu,mem),
            _ => panic!("regs:{:x?}, instr:{:08b}, {:02x}",cpu.regs, mem[cpu.regs.PC as usize],mem[cpu.regs.PC as usize]),
        }
        cpu.ret(mem);
    }
    fn print_string(&self,cpu:&mut CPU, mem:&mut [u8]){
        let off = cpu.regs.getRP(0x10);
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
}

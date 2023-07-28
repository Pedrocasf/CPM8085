use i8080_core::cpu::CPU;
use log::{debug, error};
#[derive(Clone, Copy, Debug, Default)]
pub struct CPM(pub u8);
impl CPM {
    pub fn syscall(&self, cpu: &mut CPU, mem: &mut [u8]) {
        match self.0 {
            0x09 => self.c_writestr(cpu, mem),
            0x02 => self.c_write(cpu, mem),
            _ => {
                error!(
                    "regs:{:x?}, instr:{:08b}, {:02x}",
                    cpu.get_regs(),
                    mem[cpu.get_regs().pc as usize],
                    mem[cpu.get_regs().pc as usize]
                );
                panic!(
                    "Unimplemented CPM syscall: {:02x}",
                    mem[cpu.get_regs().pc as usize],
                );
            }
        }
        cpu.ret(mem);
    }
    fn c_writestr(&self, cpu: &mut CPU, mem: &[u8]) {
        let off = cpu.get_regs().get_rp(0x10);
        let mut c: char = ' ';
        let mut count = 0;
        debug!("\n");
        while c != '$' {
            c = mem[off as usize + 3 + count] as char;
            debug!("{}", c);
            count += 1;
        }
        debug!("\n");
    }
    fn c_write(&self, cpu: &mut CPU, _mem: &mut [u8]) {
        let c = cpu.get_regs().e as char;
        debug!("{}", c);
    }
}

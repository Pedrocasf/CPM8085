use crate::{CPM, CPU};
pub struct Sys {
    cpu: CPU,
    os: CPM,
    mem: [u8; 0x10000],
}
impl Sys {
    pub fn new(com_file: &[u8]) -> Sys {
        let com_file_len = com_file.len();
        let mut mem_arr = [0xfd; 0x10000];
        mem_arr[5] = 8;
        mem_arr[0x100..0x100 + com_file_len].copy_from_slice(&com_file[0..com_file_len]);
        Sys {
            cpu: CPU::new(),
            os: CPM(0),
            mem: mem_arr,
        }
    }
    pub fn run_instruction(&mut self) {
        let m = &mut self.mem;
        self.cpu.next(m);
        if self.cpu.get_syscall() {
            let c_reg = self.cpu.get_regs().c;
            self.os.0 = c_reg;
            let cpu = &mut self.cpu;
            let mem = &mut self.mem;
            self.os.syscall(cpu, mem);
        }
    }
}

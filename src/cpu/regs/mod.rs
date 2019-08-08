pub mod flags;
use flags::Flags;

#[derive(Clone,Copy,Debug,Default)]
pub struct Registers{
    pub A:u8,
    pub B:u8,
    pub C:u8,
    pub D:u8,
    pub E:u8,
    pub H:u8,
    pub L:u8,
    pub SP:u16,
    pub PC:u16,
    pub F:Flags,
}
impl Registers{
    pub fn setRP(&mut self,val:u16,instr:u8){
        match (instr & 0x30)>>4{
            0b00 => {self.B = (val >> 8) as u8;
                    self.C = val as u8},
            0b01 => {self.D = (val >> 8) as u8;
                    self.E = val as u8},
            0b10 => {self.H = (val >> 8) as u8;
                    self.L = val as u8},
            0b11 => if instr == 0xF5 || instr == 0xF1{
                self.A = (val >> 8) as u8;
                self.F.set(val as u8);
            }else{
                self.SP = val;
            },
            _=>{},
        }
    }
    pub fn getRP(&self,instr:u8)->u16{
        match (instr & 0x30)>>4{
            0b00 => {(self.B as u16) << 8 | self.C as u16},
            0b01 => {(self.D as u16) << 8 | self.E as u16},
            0b10 => {(self.H as u16) << 8 | self.L as u16},
            0b11 => if instr == 0xF5 || instr == 0xF1{
                (self.A as u16) << 8 | self.F.get() as u16
            }else{
                self.SP
            },
            _=>panic!("Impossible RP get pattern"),
        }
    }
    pub fn getD(&mut self,i:u8,mem:&[u8])->u8{
        match (i & 0x38) >> 3{
            0 => self.B,
            1 => self.C,
            2 => self.D,
            3 => self.E,
            4 => self.H,
            5 => self.L,
            6 => mem[self.getRP(0x20) as usize],
            7 => self.A,
            _ => panic!("Impossible DDD get pattern")
        }
    }
    pub fn setD(&mut self,i:u8,mem:&mut [u8],val:u8){
        match (i & 0x38) >> 3{
            0 => self.B = val,
            1 => self.C = val,
            2 => self.D = val,
            3 => self.E = val,
            4 => self.H = val,
            5 => self.L = val,
            6 => mem[self.getRP(0x20) as usize] = val,
            7 => self.A = val,
            _ => {}
        };
    }
    pub fn getS(&mut self,i:u8,mem:&[u8])->u8{
        match i & 7{
            0 => self.B,
            1 => self.C,
            2 => self.D,
            3 => self.E,
            4 => self.H,
            5 => self.L,
            6 => mem[self.getRP(0x20) as usize],
            7 => self.A,
            _ => panic!("Impossible SSS get pattern")
        }
    }
    pub fn setS(&mut self,i:u8,mem:&mut [u8],val:u8){
        match i & 7{
            0 => self.B = val,
            1 => self.C = val,
            2 => self.D = val,
            3 => self.E = val,
            4 => self.H = val,
            5 => self.L = val,
            6 => mem[self.getRP(0x20) as usize] = val,
            7 => self.A = val,
            _ => {}
        };
    }
    pub fn set_flags(&mut self,r:u8,c:bool,h:bool){
        self.F.set_aux(h);
        self.F.set_carry(c);
        self.F.set_zero(r == 0);
        self.F.set_parity(r.count_ones() & 1 == 0);
        self.F.set_sign(r & 0x80 == 0x80);
    }
    pub fn cond(&mut self,i:u8)->bool{
        match (i & 0x38) >> 3{
            0 => !self.F.get_zero(),
            1 => self.F.get_zero(),
            2 => !self.F.get_carry(),
            3 => self.F.get_carry(),
            4 => !self.F.get_pairity(),
            5 => self.F.get_pairity(),
            6 => !self.F.get_sign(),
            7 => self.F.get_sign(),
            _ => panic!("Impossible ccc pattern"),
        }
    }
}
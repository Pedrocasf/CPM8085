#![no_std]
#![no_main]
#[macro_export] 
macro_rules! log {
    ($t:ident) => {

        #[export_name = stringify!($t)]
        #[used]
        #[no_mangle]
        static $t :u8 = 0;
    }
} 
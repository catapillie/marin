opcodes! {
    0x10 bundle(u8)
    0xa0 ld_const(u16)
    0xb0 jump(u32)
    0xb1 jump_if(u32)
    0xb2 jump_if_not(u32)
    0xe0 pop
    0xff halt
}

macro_rules! opcodes {
    (
        $(
            $byte:literal $name:ident
            $(
                ( $($arg:ty),* )
            )?
        )*
    ) => {
        #[allow(non_camel_case_types)]
        pub enum Opcode {
            $(
                $name $(($($arg),*))?,
            )*
        }


        $(
            #[allow(non_upper_case_globals)]
            pub const $name: u8 = $byte;
        )*
    };
}

use opcodes;

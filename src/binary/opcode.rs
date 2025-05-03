opcodes! {
    0x0a load_fun(u32)
    0x10 bundle(u8)
    0x11 bundle_big(u64)
    0x12 index_dup(u8)
    0x13 index_big_dup(u64)
    0x14 index(u8)
    0x16 index_big(u64)
    0xa0 load_const(u16)
    0xa2 load_local(u8)
    0xa3 set_local(u8)
    0xa4 load_nil
    0xb0 jump(u32)
    0xb1 jump_if(u32)
    0xb2 jump_if_not(u32)
    0xb3 jump_eq(u32)
    0xb4 jump_ne(u32)
    0xb5 do_frame
    0xb6 end_frame
    0xbe call(u8)
    0xbf ret
    0xe0 pop
    0xe1 dup
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

opcodes! {
    0x0a load_fun(u32)
    0x10 bundle(u8)
    0x11 bundle_big(u64)
    0x12 index_dup(u8)
    0x13 index_big_dup(u64)
    0x14 index(u8)
    0x16 index_big(u64)
    0x17 index_dyn
    0x18 spill(u16)
    0x20 add
    0x21 sub
    0x22 mul
    0x23 div
    0x24 modulo
    0x25 and
    0x26 or
    0x27 xor
    0x28 pow
    0x30 exp
    0x31 ln
    0x40 pos
    0x41 neg
    0x42 not
    0x50 eq
    0x51 ne
    0x52 lt
    0x53 le
    0x54 gt
    0x55 ge
    0x70 sin
    0x71 cos
    0x72 tan
    0x73 asin
    0x74 acos
    0x75 atan
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
    0xe1 pop_offset(u16)
    0xe2 dup
    0xfe panic
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
        #[derive(Debug, Copy, Clone)]
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

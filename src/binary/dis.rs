use colored::Colorize;
use std::io;

pub fn dissasemble<R: io::Read + io::Seek>(r: &mut R) -> super::Result<()> {
    let len = r.seek(io::SeekFrom::End(0))?;
    r.rewind()?;

    super::read_magic(r)?;
    let constants = super::read_constant_pool(r)?;
    let function_table = super::read_function_table(r)?;

    println!("         ╥");
    println!(
        "    {} ║ :: {} {}",
        "info".bold(),
        "constant pool size".underline(),
        constants.len().to_string().bold()
    );
    println!(
        "         ║ :: {} {}",
        "function table size".underline(),
        function_table.len().to_string().bold()
    );
    println!("         ║ ");

    let orig = r.stream_position()?;

    use super::Opcode as Op;
    loop {
        let pos = r.stream_position()?;
        if pos >= len {
            break;
        }

        let opcode = super::read_opcode(r)?;
        print!("{:0>8} ║ ", pos - orig);
        match opcode {
            Op::bundle(count) => print!("{:>12} [{}]", "bundle", count.to_string().bold()),
            Op::index(count) => print!("{:>12} {}", "index", count.to_string().bold()),
            Op::load_const(x) => print!(
                "{:>12} #{} = {}",
                "load_const",
                x.to_string().bold(),
                constants[x as usize].to_string().bold()
            ),
            Op::load_local(x) => print!("{:>12} {}", "load_local", x.to_string().bold()),
            Op::set_local(x) => print!("{:>12} {}", "set_local", x.to_string().bold()),
            Op::load_nil => print!("{:>12}", "load_nil"),
            Op::jump(pos) => print!("{:>12} -> <{:0>8}>", "jump", pos.to_string().bold()),
            Op::jump_if(pos) => {
                print!("{:>12} -> <{:0>8}>", "jump_if", pos.to_string().bold())
            }
            Op::jump_if_not(pos) => {
                print!("{:>12} -> <{:0>8}>", "jump_if_not", pos.to_string().bold())
            }
            Op::do_frame => print!("{:>12}", "do_frame"),
            Op::end_frame => print!("{:>12}", "end_frame"),
            Op::ret => print!("{:>12}", "ret"),
            Op::pop => print!("{:>12}", "pop"),
            Op::dup => print!("{:>12}", "dup"),
        }
        println!();
    }

    println!("         ╨");

    Ok(())
}

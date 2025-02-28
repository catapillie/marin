use colored::Colorize;
use std::io;

pub fn dissasemble<R: io::Read + io::Seek>(r: &mut R) -> super::Result<()> {
    let len = r.seek(io::SeekFrom::End(0))?;
    r.rewind()?;

    super::read_magic(r)?;
    let constants = super::read_constant_pool(r)?;

    println!("         ╥");
    println!(
        "    {} ║ :: {} {}",
        "info".bold(),
        "constant pool size".underline(),
        constants.len().to_string().bold()
    );
    println!("         ║ ");

    use super::Opcode as Op;
    loop {
        let pos = r.stream_position()?;
        if pos >= len {
            break;
        }

        let opcode = super::read_opcode(r)?;
        print!("{pos:0>8} ║ ");
        match opcode {
            Op::bundle(count) => print!("{:>12} [{}]", "bundle", count.to_string().bold()),
            Op::ld_const(x) => print!(
                "{:>12} #{} = {}",
                "ld_const",
                x.to_string().bold(),
                constants[x as usize].to_string().bold()
            ),
            Op::pop => print!("{:>12}", "pop"),
            Op::halt => print!("{:>12}", "halt"),
        }
        println!();
    }

    println!("         ╨");

    Ok(())
}

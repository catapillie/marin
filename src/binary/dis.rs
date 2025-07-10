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
    for (i, constant) in constants.iter().enumerate() {
        println!(
            "         ║      #{i} = {}",
            constant.to_string().bold().yellow()
        );
    }

    println!(
        "         ║ :: {} {}",
        "function table size".underline(),
        function_table.len().to_string().bold()
    );
    for (pos, name) in &function_table {
        println!(
            "         ║      {} -> <{:0>8}>",
            name.bold().bright_blue(),
            pos.to_string().bold()
        );
    }

    println!("         ║ ");

    let orig = r.stream_position()?;

    use super::Opcode as Op;
    loop {
        let pos = r.stream_position()?;
        if pos >= len {
            break;
        }

        let pos = pos - orig;

        match function_table.get(&(pos as u32)) {
            Some(fun_name) => {
                println!("         ║ :: {}", fun_name.bold().bright_blue());
                print!("{pos:0>8} ║ ");
            }
            None => print!("{pos:0>8} ║ "),
        }

        let opcode = super::read_opcode(r)?;
        match opcode {
            Op::load_fun(pos) => print!(
                "{:>14} {} -> <{:0>8}>",
                "load_fun",
                function_table.get(&pos).unwrap().bold().bright_blue(),
                pos.to_string().bold()
            ),
            Op::bundle(count) => print!("{:>14} [{}]", "bundle", count.to_string().bold()),
            Op::bundle_big(count) => print!("{:>14} [{}]", "bundle_big", count.to_string().bold()),
            Op::index_dup(count) => print!("{:>14} {}", "index_dup", count.to_string().bold()),
            Op::index_big_dup(count) => {
                print!("{:>14} {}", "index_big_dup", count.to_string().bold())
            }
            Op::index(count) => print!("{:>14} {}", "index", count.to_string().bold()),
            Op::index_big(count) => print!("{:>14} {}", "index_big", count.to_string().bold()),
            Op::spill(offset) => {
                print!("{:>14} {}", "spill", offset.to_string().bold())
            }
            Op::add => print!("{:>14}", "add"),
            Op::sub => print!("{:>14}", "sub"),
            Op::mul => print!("{:>14}", "mul"),
            Op::div => print!("{:>14}", "div"),
            Op::modulo => print!("{:>14}", "modulo"),
            Op::pow => print!("{:>14}", "pow"),
            Op::and => print!("{:>14}", "and"),
            Op::or => print!("{:>14}", "or"),
            Op::xor => print!("{:>14}", "xor"),
            Op::exp => print!("{:>14}", "exp"),
            Op::ln => print!("{:>14}", "ln"),
            Op::pos => print!("{:>14}", "pos"),
            Op::neg => print!("{:>14}", "neg"),
            Op::not => print!("{:>14}", "not"),
            Op::eq => print!("{:>14}", "eq"),
            Op::ne => print!("{:>14}", "ne"),
            Op::lt => print!("{:>14}", "lt"),
            Op::le => print!("{:>14}", "le"),
            Op::gt => print!("{:>14}", "gt"),
            Op::ge => print!("{:>14}", "ge"),
            Op::sin => print!("{:>14}", "sin"),
            Op::cos => print!("{:>14}", "cos"),
            Op::tan => print!("{:>14}", "tan"),
            Op::asin => print!("{:>14}", "asin"),
            Op::acos => print!("{:>14}", "acos"),
            Op::atan => print!("{:>14}", "atan"),
            Op::load_const(x) => print!(
                "{:>14} #{} = {}",
                "load_const",
                x.to_string().bold(),
                constants[x as usize].to_string().bold().yellow()
            ),
            Op::load_local(x) => print!("{:>14} {}", "load_local", x.to_string().bold().red()),
            Op::set_local(x) => print!("{:>14} {}", "set_local", x.to_string().bold().red()),
            Op::load_nil => print!("{:>14}", "load_nil"),
            Op::jump(pos) => print!("{:>14} -> <{:0>8}>", "jump", pos.to_string().bold()),
            Op::jump_if(pos) => {
                print!("{:>14} -> <{:0>8}>", "jump_if", pos.to_string().bold())
            }
            Op::jump_if_not(pos) => {
                print!("{:>14} -> <{:0>8}>", "jump_if_not", pos.to_string().bold())
            }
            Op::jump_eq(pos) => {
                print!("{:>14} -> <{:0>8}>", "jump_eq", pos.to_string().bold())
            }
            Op::jump_ne(pos) => {
                print!("{:>14} -> <{:0>8}>", "jump_ne", pos.to_string().bold())
            }
            Op::do_frame => print!("{:>14}", "do_frame"),
            Op::end_frame => print!("{:>14}", "end_frame"),
            Op::call(count) => print!("{:>14} [{}]", "call", count.to_string().bold()),
            Op::ret => print!("{:>14}", "ret"),
            Op::pop => print!("{:>14}", "pop"),
            Op::pop_offset(offset) => print!("{:>14} {}", "pop_offset", offset.to_string().bold()),
            Op::dup => print!("{:>14}", "dup"),
            Op::panic => print!("{:>14}", "panic"),
        }
        println!();
    }

    println!("         ╨");

    Ok(())
}

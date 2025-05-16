mod error;
pub use error::{Error, Result};

pub mod value;

pub mod opcode;
pub use opcode::Opcode;

use crate::exe::Value;
use byteorder::{LE, ReadBytesExt, WriteBytesExt};
use std::{
    collections::{BTreeMap, HashMap},
    io::{self},
};

pub const MAGIC: &[u8] = "exemarin".as_bytes();

pub fn read_magic<R: io::Read>(r: &mut R) -> Result<()> {
    let mut buf = [0; MAGIC.len()];
    r.read_exact(&mut buf)?;
    match buf == MAGIC {
        true => Ok(()),
        false => Err(Error::MagicMismatch),
    }
}

pub fn write_magic<W: io::Write>(w: &mut W) -> Result<()> {
    w.write_all(MAGIC)?;
    Ok(())
}

pub fn read_opcode<R: io::Read>(r: &mut R) -> Result<Opcode> {
    match r.read_u8()? {
        opcode::load_fun => Ok(Opcode::load_fun(r.read_u32::<LE>()?)),
        opcode::bundle => Ok(Opcode::bundle(r.read_u8()?)),
        opcode::bundle_big => Ok(Opcode::bundle_big(r.read_u64::<LE>()?)),
        opcode::index_dup => Ok(Opcode::index_dup(r.read_u8()?)),
        opcode::index_big_dup => Ok(Opcode::index_big_dup(r.read_u64::<LE>()?)),
        opcode::index => Ok(Opcode::index(r.read_u8()?)),
        opcode::index_big => Ok(Opcode::index_big(r.read_u64::<LE>()?)),
        opcode::spill => Ok(Opcode::spill(r.read_u16::<LE>()?)),
        opcode::add => Ok(Opcode::add),
        opcode::sub => Ok(Opcode::sub),
        opcode::mul => Ok(Opcode::mul),
        opcode::div => Ok(Opcode::div),
        opcode::modulo => Ok(Opcode::modulo),
        opcode::and => Ok(Opcode::and),
        opcode::or => Ok(Opcode::or),
        opcode::xor => Ok(Opcode::xor),
        opcode::pow => Ok(Opcode::pow),
        opcode::exp => Ok(Opcode::exp),
        opcode::ln => Ok(Opcode::ln),
        opcode::pos => Ok(Opcode::pos),
        opcode::neg => Ok(Opcode::neg),
        opcode::not => Ok(Opcode::not),
        opcode::eq => Ok(Opcode::eq),
        opcode::ne => Ok(Opcode::ne),
        opcode::lt => Ok(Opcode::lt),
        opcode::le => Ok(Opcode::le),
        opcode::gt => Ok(Opcode::gt),
        opcode::ge => Ok(Opcode::ge),
        opcode::sin => Ok(Opcode::sin),
        opcode::cos => Ok(Opcode::cos),
        opcode::tan => Ok(Opcode::tan),
        opcode::asin => Ok(Opcode::asin),
        opcode::acos => Ok(Opcode::acos),
        opcode::atan => Ok(Opcode::atan),
        opcode::load_const => Ok(Opcode::load_const(r.read_u16::<LE>()?)),
        opcode::load_local => Ok(Opcode::load_local(r.read_u8()?)),
        opcode::set_local => Ok(Opcode::set_local(r.read_u8()?)),
        opcode::load_nil => Ok(Opcode::load_nil),
        opcode::jump => Ok(Opcode::jump(r.read_u32::<LE>()?)),
        opcode::jump_if => Ok(Opcode::jump_if(r.read_u32::<LE>()?)),
        opcode::jump_if_not => Ok(Opcode::jump_if_not(r.read_u32::<LE>()?)),
        opcode::jump_eq => Ok(Opcode::jump_eq(r.read_u32::<LE>()?)),
        opcode::jump_ne => Ok(Opcode::jump_ne(r.read_u32::<LE>()?)),
        opcode::do_frame => Ok(Opcode::do_frame),
        opcode::end_frame => Ok(Opcode::end_frame),
        opcode::call => Ok(Opcode::call(r.read_u8()?)),
        opcode::ret => Ok(Opcode::ret),
        opcode::pop => Ok(Opcode::pop),
        opcode::pop_offset => Ok(Opcode::pop_offset(r.read_u16::<LE>()?)),
        opcode::dup => Ok(Opcode::dup),
        opcode::panic => Ok(Opcode::panic),
        byte => Err(Error::IllegalOpcode(byte)),
    }
}

pub fn write_opcode<W: io::Write>(w: &mut W, opcode: &Opcode) -> Result<()> {
    match opcode {
        Opcode::load_fun(addr) => {
            w.write_u8(opcode::load_fun)?;
            w.write_u32::<LE>(*addr)?;
            Ok(())
        }
        Opcode::bundle(count) => {
            w.write_u8(opcode::bundle)?;
            w.write_u8(*count)?;
            Ok(())
        }
        Opcode::bundle_big(count) => {
            w.write_u8(opcode::bundle_big)?;
            w.write_u64::<LE>(*count)?;
            Ok(())
        }
        Opcode::index_dup(count) => {
            w.write_u8(opcode::index_dup)?;
            w.write_u8(*count)?;
            Ok(())
        }
        Opcode::index_big_dup(count) => {
            w.write_u8(opcode::index_big_dup)?;
            w.write_u64::<LE>(*count)?;
            Ok(())
        }
        Opcode::index(count) => {
            w.write_u8(opcode::index)?;
            w.write_u8(*count)?;
            Ok(())
        }
        Opcode::index_big(count) => {
            w.write_u8(opcode::index_big)?;
            w.write_u64::<LE>(*count)?;
            Ok(())
        }
        Opcode::spill(offset) => {
            w.write_u8(opcode::spill)?;
            w.write_u16::<LE>(*offset)?;
            Ok(())
        }
        Opcode::add => {
            w.write_u8(opcode::add)?;
            Ok(())
        }
        Opcode::sub => {
            w.write_u8(opcode::sub)?;
            Ok(())
        }
        Opcode::mul => {
            w.write_u8(opcode::mul)?;
            Ok(())
        }
        Opcode::div => {
            w.write_u8(opcode::div)?;
            Ok(())
        }
        Opcode::modulo => {
            w.write_u8(opcode::modulo)?;
            Ok(())
        }
        Opcode::pow => {
            w.write_u8(opcode::pow)?;
            Ok(())
        }
        Opcode::and => {
            w.write_u8(opcode::and)?;
            Ok(())
        }
        Opcode::or => {
            w.write_u8(opcode::or)?;
            Ok(())
        }
        Opcode::xor => {
            w.write_u8(opcode::xor)?;
            Ok(())
        }
        Opcode::exp => {
            w.write_u8(opcode::exp)?;
            Ok(())
        }
        Opcode::ln => {
            w.write_u8(opcode::ln)?;
            Ok(())
        }
        Opcode::pos => {
            w.write_u8(opcode::pos)?;
            Ok(())
        }
        Opcode::neg => {
            w.write_u8(opcode::neg)?;
            Ok(())
        }
        Opcode::not => {
            w.write_u8(opcode::not)?;
            Ok(())
        }
        Opcode::eq => {
            w.write_u8(opcode::eq)?;
            Ok(())
        }
        Opcode::ne => {
            w.write_u8(opcode::ne)?;
            Ok(())
        }
        Opcode::lt => {
            w.write_u8(opcode::lt)?;
            Ok(())
        }
        Opcode::le => {
            w.write_u8(opcode::le)?;
            Ok(())
        }
        Opcode::gt => {
            w.write_u8(opcode::gt)?;
            Ok(())
        }
        Opcode::ge => {
            w.write_u8(opcode::ge)?;
            Ok(())
        }
        Opcode::sin => {
            w.write_u8(opcode::sin)?;
            Ok(())
        }
        Opcode::cos => {
            w.write_u8(opcode::cos)?;
            Ok(())
        }
        Opcode::tan => {
            w.write_u8(opcode::tan)?;
            Ok(())
        }
        Opcode::asin => {
            w.write_u8(opcode::asin)?;
            Ok(())
        }
        Opcode::acos => {
            w.write_u8(opcode::acos)?;
            Ok(())
        }
        Opcode::atan => {
            w.write_u8(opcode::atan)?;
            Ok(())
        }
        Opcode::load_const(x) => {
            w.write_u8(opcode::load_const)?;
            w.write_u16::<LE>(*x)?;
            Ok(())
        }
        Opcode::load_local(x) => {
            w.write_u8(opcode::load_local)?;
            w.write_u8(*x)?;
            Ok(())
        }
        Opcode::set_local(x) => {
            w.write_u8(opcode::set_local)?;
            w.write_u8(*x)?;
            Ok(())
        }
        Opcode::load_nil => {
            w.write_u8(opcode::load_nil)?;
            Ok(())
        }
        Opcode::jump(pos) => {
            w.write_u8(opcode::jump)?;
            w.write_u32::<LE>(*pos)?;
            Ok(())
        }
        Opcode::jump_if(pos) => {
            w.write_u8(opcode::jump_if)?;
            w.write_u32::<LE>(*pos)?;
            Ok(())
        }
        Opcode::jump_if_not(pos) => {
            w.write_u8(opcode::jump_if_not)?;
            w.write_u32::<LE>(*pos)?;
            Ok(())
        }
        Opcode::jump_eq(pos) => {
            w.write_u8(opcode::jump_eq)?;
            w.write_u32::<LE>(*pos)?;
            Ok(())
        }
        Opcode::jump_ne(pos) => {
            w.write_u8(opcode::jump_ne)?;
            w.write_u32::<LE>(*pos)?;
            Ok(())
        }
        Opcode::do_frame => {
            w.write_u8(opcode::do_frame)?;
            Ok(())
        }
        Opcode::end_frame => {
            w.write_u8(opcode::end_frame)?;
            Ok(())
        }
        Opcode::call(count) => {
            w.write_u8(opcode::call)?;
            w.write_u8(*count)?;
            Ok(())
        }
        Opcode::ret => {
            w.write_u8(opcode::ret)?;
            Ok(())
        }
        Opcode::pop => {
            w.write_u8(opcode::pop)?;
            Ok(())
        }
        Opcode::pop_offset(offset) => {
            w.write_u8(opcode::pop_offset)?;
            w.write_u16::<LE>(*offset)?;
            Ok(())
        }
        Opcode::dup => {
            w.write_u8(opcode::dup)?;
            Ok(())
        }
        Opcode::panic => {
            w.write_u8(opcode::panic)?;
            Ok(())
        }
    }
}

fn read_string<R: io::Read>(r: &mut R) -> Result<String> {
    let len = r.read_u64::<LE>()? as usize;
    let mut buf = vec![0; len];
    r.read_exact(&mut buf)?;
    Ok(String::from_utf8(buf)?)
}

fn write_string<W: io::Write>(w: &mut W, s: &str) -> Result<()> {
    w.write_u64::<LE>(s.len() as u64)?;
    w.write_all(s.as_bytes())?;
    Ok(())
}

pub fn read_value<R: io::Read>(r: &mut R) -> Result<Value> {
    match r.read_u8()? {
        value::int => Ok(Value::Int(r.read_i64::<LE>()?)),
        value::float => Ok(Value::Float(r.read_f64::<LE>()?)),
        value::string => Ok(Value::String(read_string(r)?)),
        value::bool => Ok(Value::Bool(r.read_u8()? != 0)),
        value::bundle => {
            let count = r.read_u8()? as usize;
            let mut items = Vec::with_capacity(count);
            for _ in 0..count {
                items.push(read_value(r)?);
            }
            Ok(Value::Bundle(items.into()))
        }
        _ => Err(Error::IllegalValue),
    }
}

pub fn write_value<W: io::Write>(w: &mut W, value: &Value) -> Result<()> {
    match value {
        Value::Nil => unimplemented!(),
        Value::Int(n) => {
            w.write_u8(value::int)?;
            w.write_i64::<LE>(*n)?;
            Ok(())
        }
        Value::Float(f) => {
            w.write_u8(value::float)?;
            w.write_f64::<LE>(*f)?;
            Ok(())
        }
        Value::String(s) => {
            w.write_u8(value::string)?;
            write_string(w, s)?;
            Ok(())
        }
        Value::Bool(b) => {
            w.write_u8(value::bool)?;
            w.write_u8(*b as u8)?;
            Ok(())
        }
        Value::Func => unimplemented!(),
        Value::Bundle(items) => {
            w.write_u8(value::bundle)?;
            w.write_u8(
                items
                    .len()
                    .try_into()
                    .expect("bundle has more than 255 items"),
            )?;
            for item in items {
                write_value(w, item)?;
            }
            Ok(())
        }
    }
}

pub fn read_constant_pool<R: io::Read>(r: &mut R) -> Result<Vec<Value>> {
    let count = r.read_u16::<LE>()? as usize;
    let mut constants = Vec::with_capacity(count);
    for _ in 0..count {
        constants.push(read_value(r)?);
    }
    Ok(constants)
}

pub fn write_constant_pool<W: io::Write>(w: &mut W, constants: &[Value]) -> Result<()> {
    w.write_u16::<LE>(
        constants
            .len()
            .try_into()
            .expect("constant pool has more than 65535 values"),
    )?;
    for value in constants {
        write_value(w, value)?;
    }
    Ok(())
}

pub fn read_function_info<R: io::Read>(r: &mut R) -> Result<(u32, String)> {
    let pos = r.read_u32::<LE>()?;
    let name = read_string(r)?;
    Ok((pos, name))
}

pub fn write_function_info<W: io::Write>(w: &mut W, pos: u32, name: &str) -> Result<()> {
    w.write_u32::<LE>(pos)?;
    write_string(w, name)?;
    Ok(())
}

pub fn read_function_table<R: io::Read>(r: &mut R) -> Result<BTreeMap<u32, String>> {
    let count = r.read_u16::<LE>()? as usize;
    let mut table = BTreeMap::new();
    for _ in 0..count {
        let (pos, name) = read_function_info(r)?;
        table.insert(pos, name);
    }
    Ok(table)
}

pub fn write_function_table<W: io::Write>(w: &mut W, table: &HashMap<u32, String>) -> Result<()> {
    w.write_u16::<LE>(
        table
            .len()
            .try_into()
            .expect("function table has more than 65535 entries"),
    )?;
    for (pos, name) in table {
        write_function_info(w, *pos, name)?;
    }
    Ok(())
}

mod dis;
pub use dis::dissasemble;

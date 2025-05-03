mod error;
pub use error::{Error, Result};

pub mod value;

pub mod opcode;
pub use opcode::Opcode;

use crate::exe::Value;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::{
    collections::HashMap,
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
        opcode::dup => Ok(Opcode::dup),
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
        Opcode::dup => {
            w.write_u8(opcode::dup)?;
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

pub fn read_function_table<R: io::Read>(r: &mut R) -> Result<HashMap<u32, String>> {
    let count = r.read_u16::<LE>()? as usize;
    let mut table = HashMap::with_capacity(count);
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

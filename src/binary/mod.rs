mod error;
pub use error::{Error, Result};

pub mod value;

pub mod opcode;
pub use opcode::Opcode;

use crate::exe::Value;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::io::{self};

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
        opcode::bundle => Ok(Opcode::bundle(r.read_u8()?)),
        opcode::ld_const => Ok(Opcode::ld_const(r.read_u16::<LE>()?)),
        opcode::pop => Ok(Opcode::pop),
        opcode::halt => Ok(Opcode::halt),
        _ => Err(Error::IllegalOpcode),
    }
}

pub fn write_opcode<W: io::Write>(w: &mut W, opcode: &Opcode) -> Result<()> {
    match opcode {
        Opcode::bundle(count) => {
            w.write_u8(opcode::bundle)?;
            w.write_u8(*count)?;
            Ok(())
        }
        Opcode::ld_const(x) => {
            w.write_u8(opcode::ld_const)?;
            w.write_u16::<LE>(*x)?;
            Ok(())
        }
        Opcode::pop => {
            w.write_u8(opcode::pop)?;
            Ok(())
        }
        Opcode::halt => {
            w.write_u8(opcode::halt)?;
            Ok(())
        }
    }
}

pub fn read_value<R: io::Read>(r: &mut R) -> Result<Value> {
    match r.read_u8()? {
        value::int => Ok(Value::Int(r.read_i64::<LE>()?)),
        value::float => Ok(Value::Float(r.read_f64::<LE>()?)),
        value::string => {
            let len = r.read_u64::<LE>()? as usize;
            let mut buf = vec![0; len];
            r.read_exact(&mut buf)?;
            Ok(Value::String(String::from_utf8(buf)?))
        }
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
            w.write_u64::<LE>(s.len() as u64)?;
            w.write_all(s.as_bytes())?;
            Ok(())
        }
        Value::Bool(b) => {
            w.write_u8(value::bool)?;
            w.write_u8(*b as u8)?;
            Ok(())
        }
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

mod dis;
pub use dis::dissasemble;

use untrusted;
use codec;
use error::CodecError;

#[derive(Debug, PartialEq)]
pub enum ValueType {
    i32,
    i64,
    f32,
    f64,
}

impl ValueType {
    pub fn decode(rd: &mut untrusted::Reader) -> Result<ValueType, CodecError> {
        let ty = codec::read_vari7(rd)?;
        println!("valtype {:?}", ty);
        match ty {
            -0x01 => Ok(ValueType::i32),
            -0x02 => Ok(ValueType::i64),
            -0x03 => Ok(ValueType::f32),
            -0x04 => Ok(ValueType::f64),
            _ => Err(CodecError::BadType),
        }
    }
}

#[derive(Debug)]
pub enum BlockType {
    Single(ValueType),
    Void,
}

impl BlockType {
    pub fn decode(rd: &mut untrusted::Reader) -> Result<BlockType, CodecError> {
        if rd.peek(0x40) {
            let _ = rd.read_byte();
            Ok(BlockType::Void)
        } else {
            Ok(BlockType::Single(ValueType::decode(rd)?))
        }
    }
}

#[derive(Debug)]
pub struct AnyFunction;

impl AnyFunction {
    pub fn decode(rd: &mut untrusted::Reader) -> Result<AnyFunction, CodecError> {
        let ty = codec::read_vari7(rd)?;

        match ty {
            -0x10 => Ok(AnyFunction),
            _ => Err(CodecError::BadType),
        }
    }
}

pub type ElementType = AnyFunction;

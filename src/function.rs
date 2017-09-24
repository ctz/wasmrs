use ty::ValueType;
use expr::Op;
use error::CodecError;
use codec;

use untrusted;

#[derive(Debug)]
struct Local {
    count: u32,
    ty: ValueType,
}

impl Local {
    pub fn decode(rd: &mut untrusted::Reader) -> Result<Local, CodecError> {
        let count = codec::read_varu32(rd)?;
        let ty = ValueType::decode(rd)?;
        Ok(Local { count, ty })
    }
}

#[derive(Debug)]
pub struct FunctionBody {
    locals: Vec<Local>,
    ops: Vec<Op>,
}

impl FunctionBody {
    pub fn decode(rd: &mut untrusted::Reader) -> Result<FunctionBody, CodecError> {
        let body_size = codec::read_varu32(rd)?;

        let mut body = rd.skip_and_get_input(body_size as usize)
            .map_err(|_| CodecError::Truncated)
            .map(|inp| untrusted::Reader::new(inp))?;

        let mut locals = vec![];
        let local_count = codec::read_varu32(&mut body)?;

        for _ in 0..local_count {
            locals.push(Local::decode(&mut body)?);
        }

        let mut ops = vec![];
        loop {
            let op = Op::decode(&mut body)?;

            if body.at_end() {
                match op {
                    Op::End => break,
                    _ => return Err(CodecError::BadFunctionEnd)
                }
            }

            ops.push(op);
        }

        Ok(FunctionBody { locals, ops })
    }
}

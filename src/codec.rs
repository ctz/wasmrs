use error::CodecError;

use untrusted;
use byteorder::{ByteOrder, LittleEndian};
use std::str;

pub fn read_u8(rd: &mut untrusted::Reader) -> Result<u8, CodecError> {
    rd.read_byte()
        .map_err(|_| CodecError::Truncated)
}

pub fn read_u32(rd: &mut untrusted::Reader) -> Result<u32, CodecError> {
    rd.skip_and_get_input(4)
        .map_err(|_| CodecError::Truncated)
        .map(|inp| LittleEndian::read_u32(inp.as_slice_less_safe()))
}

pub fn read_u64(rd: &mut untrusted::Reader) -> Result<u64, CodecError> {
    rd.skip_and_get_input(8)
        .map_err(|_| CodecError::Truncated)
        .map(|inp| LittleEndian::read_u64(inp.as_slice_less_safe()))
}

pub fn read_varu(rd: &mut untrusted::Reader, mut len: usize) -> Result<u64, CodecError> {
    let mut r = 0u64;
    let mut shift = 0;

    loop {
        let b = rd.read_byte()
            .map_err(|_| CodecError::Truncated)?;
        len -= 1;

        let topbit = b >> 7;
        println!("byte {:02x} top {} len {}", b, topbit, len);

        match (topbit, len) {
            (1, 0) => return Err(CodecError::BadVarInt),
            (1, _) | (0, _) => {
                r |= ((b & 0x7f) as u64) << shift;
                shift += 7;

                if topbit == 0 || len == 0 {
                    return Ok(r);
                }
            }
            (_, _) => unreachable!()
        }
    }
}

pub fn read_varu7(rd: &mut untrusted::Reader) -> Result<u8, CodecError> {
    Ok(read_varu(rd, 1)? as u8)
}

pub fn read_varu1(rd: &mut untrusted::Reader) -> Result<u8, CodecError> {
    let v = read_varu7(rd)?;
    match v {
        0 | 1 => Ok(v),
        _ => Err(CodecError::BadVarInt),
    }
}

pub fn read_vari7(rd: &mut untrusted::Reader) -> Result<i8, CodecError> {
    let v = read_varu7(rd)?;
    if v & 0x40 != 0 {
        Ok(((v & 0x3f) as i8) - 0x40)
    } else {
        Ok(v as i8)
    }
}

pub fn read_varu32(rd: &mut untrusted::Reader) -> Result<u32, CodecError> {
    Ok(read_varu(rd, 5)? as u32)
}

pub fn read_vari32(rd: &mut untrusted::Reader) -> Result<i32, CodecError> {
    let v = read_varu32(rd)?;
    if v & 0x4000_0000 != 0 {
        Ok(((v & 0x3fff_ffff) as i32) - 0x4000_0000)
    } else {
        Ok(v as i32)
    }
}

pub fn read_vari64(rd: &mut untrusted::Reader) -> Result<i64, CodecError> {
    let v = read_varu(rd, 10)?;
    if v & 0x4000_0000_0000_0000 != 0 {
        Ok(((v & 0x3fff_ffff_ffff_ffff) as i64) - 0x4000_0000_0000_0000)
    } else {
        Ok(v as i64)
    }
}

pub fn read_utf8<'a>(rd: &mut untrusted::Reader<'a>, len: usize) -> Result<&'a str, CodecError> {
    rd.skip_and_get_input(len)
        .map_err(|_| CodecError::Truncated)
        .and_then(|inp| str::from_utf8(inp.as_slice_less_safe())
                  .map_err(|_| CodecError::BadUTF8))
}

#[test]
fn test_read_varu7() {
    let b = [0x7f];
    assert_eq!(read_varu7(&mut untrusted::Reader::new(untrusted::Input::from(&b))),
               Ok(0x7f));
}

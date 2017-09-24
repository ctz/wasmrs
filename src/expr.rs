use untrusted;
use error::CodecError;
use ty::BlockType;
use codec;

#[derive(Debug)]
pub struct MemoryImmed {
    pub align: u8,
    pub offset: u32
}

impl MemoryImmed {
    pub fn decode(rd: &mut untrusted::Reader) -> Result<MemoryImmed, CodecError> {
        let flags = codec::read_varu32(rd)?;
        let offset = codec::read_varu32(rd)?;
        let align = flags as u8;

        Ok(MemoryImmed { align, offset })
    }
}

#[derive(Debug)]
pub struct BranchTable {
    targets: Vec<u32>,
    default: u32,
}

impl BranchTable {
    pub fn decode(rd: &mut untrusted::Reader) -> Result<BranchTable, CodecError> {
        let count = codec::read_varu32(rd)?;

        let mut targets = vec![];
        for _ in 0..count {
            targets.push(codec::read_varu32(rd)?);
        }

        let default = codec::read_varu32(rd)?;

        Ok(BranchTable { targets, default })
    }
}


#[derive(Debug)]
pub enum Op {
    Unreachable,
    Nop,
    Block(BlockType),
    Loop(BlockType),
    If(BlockType),
    Else,
    End,
    Branch(u32),
    BranchIf(u32),
    BranchTable(BranchTable),
    Return,
    Call(u32),
    CallIndirect(u32),
    Drop,
    Select,
    GetLocal(u32),
    SetLocal(u32),
    TeeLocal(u32),
    GetGlobal(u32),
    SetGlobal(u32),
    I32Const(i32),
    I64Const(i64),
    F32Const(f32),
    F64Const(f64),
    I32Eqz,
    I32Eq,
    I32Ne,
    I32LtSigned,
    I32LtUnsigned,
    I32GtSigned,
    I32GtUnsigned,
    I32LeSigned,
    I32LeUnsigned,
    I32GeSigned,
    I32GeUnsigned,
    I64Eqz,
    I64Eq,
    I64Ne,
    I64LtSigned,
    I64LtUnsigned,
    I64GtSigned,
    I64GtUnsigned,
    I64LeSigned,
    I64LeUnsigned,
    I64GeSigned,
    I64GeUnsigned,
    F32Eq,
    F32Ne,
    F32Lt,
    F32Gt,
    F32Le,
    F32Ge,
    F64Eq,
    F64Ne,
    F64Lt,
    F64Gt,
    F64Le,
    F64Ge,
    I32Clz,
    I32Ctz,
    I32Popcnt,
    I32Add,
    I32Sub,
    I32Mul,
    I32DivSigned,
    I32DivUnsigned,
    I32RemSigned,
    I32RemUnsigned,
    I32And,
    I32Or,
    I32Xor,
    I32Shl,
    I32ShrSigned,
    I32ShrUnsigned,
    I32Rotl,
    I32Rotr,
    I64Clz,
    I64Ctz,
    I64Popcnt,
    I64Add,
    I64Sub,
    I64Mul,
    I64DivSigned,
    I64DivUnsigned,
    I64RemSigned,
    I64RemUnsigned,
    I64And,
    I64Or,
    I64Xor,
    I64Shl,
    I64ShrSigned,
    I64ShrUnsigned,
    I64Rotl,
    I64Rotr,
    F32Abs,
    F32Neg,
    F32Ceil,
    F32Floor,
    F32Trunc,
    F32Nearest,
    F32Sqrt,
    F32Add,
    F32Sub,
    F32Mul,
    F32Div,
    F32Min,
    F32Max,
    F32Copysign,
    F64Abs,
    F64Neg,
    F64Ceil,
    F64Floor,
    F64Trunc,
    F64Nearest,
    F64Sqrt,
    F64Add,
    F64Sub,
    F64Mul,
    F64Div,
    F64Min,
    F64Max,
    F64Copysign,
    I32WrapI64,
    I32TruncSignedF32,
    I32TruncUnsignedF32,
    I32TruncSignedF64,
    I32TruncUnsignedF64,
    I64ExtendSignedI32,
    I64ExtendUnsignedI32,
    I64TruncSignedF32,
    I64TruncUnsignedF32,
    I64TruncSignedF64,
    I64TruncUnsignedF64,
    F32ConvertSignedI32,
    F32ConvertUnsignedI32,
    F32ConvertSignedI64,
    F32ConvertUnsignedI64,
    F32DemoteF64,
    F64ConvertSignedI32,
    F64ConvertUnsignedI32,
    F64ConvertSignedI64,
    F64ConvertUnsignedI64,
    F64PromoteF32,
    I32ReinterpretF32,
    I64ReinterpretF64,
    F32ReinterpretI32,
    F64ReinterpretI64,
    I32Load(MemoryImmed),
    I64Load(MemoryImmed),
    F32Load(MemoryImmed),
    F64Load(MemoryImmed),
    I32Load8Signed(MemoryImmed),
    I32Load8Unsigned(MemoryImmed),
    I32Load16Signed(MemoryImmed),
    I32Load16Unsigned(MemoryImmed),
    I64Load8Signed(MemoryImmed),
    I64Load8Unsigned(MemoryImmed),
    I64Load16Signed(MemoryImmed),
    I64Load16Unsigned(MemoryImmed),
    I64Load32Signed(MemoryImmed),
    I64Load32Unsigned(MemoryImmed),
    I32Store(MemoryImmed),
    I64Store(MemoryImmed),
    F32Store(MemoryImmed),
    F64Store(MemoryImmed),
    I32Store8(MemoryImmed),
    I32Store16(MemoryImmed),
    I64Store8(MemoryImmed),
    I64Store16(MemoryImmed),
    I64Store32(MemoryImmed),
    CurrentMemory(u8),
    GrowMemory(u8),
}

fn call_indirect(rd: &mut untrusted::Reader) -> Result<u32, CodecError> {
    let index = codec::read_varu32(rd)?;
    if codec::read_varu1(rd)? != 0 {
        Err(CodecError::BadOpArgs)
    } else {
        Ok(index)
    }
}

impl Op {
    pub fn decode(rd: &mut untrusted::Reader) -> Result<Op, CodecError> {
        let op = codec::read_u8(rd)?;
        println!("op {:02x}", op);
        match op {
            0x00 => Ok(Op::Unreachable),
            0x01 => Ok(Op::Nop),
            0x02 => Ok(Op::Block(BlockType::decode(rd)?)),
            0x03 => Ok(Op::Loop(BlockType::decode(rd)?)),
            0x04 => Ok(Op::If(BlockType::decode(rd)?)),
            0x05 => Ok(Op::Else),
            0x0b => Ok(Op::End),
            0x0c => Ok(Op::Branch(codec::read_varu32(rd)?)),
            0x0d => Ok(Op::BranchIf(codec::read_varu32(rd)?)),
            0x0e => Ok(Op::BranchTable(BranchTable::decode(rd)?)),
            0x0f => Ok(Op::Return),
            0x10 => Ok(Op::Call(codec::read_varu32(rd)?)),
            0x11 => Ok(Op::CallIndirect(call_indirect(rd)?)),
            0x1a => Ok(Op::Drop),
            0x1b => Ok(Op::Select),
            0x20 => Ok(Op::GetLocal(codec::read_varu32(rd)?)),
            0x21 => Ok(Op::SetLocal(codec::read_varu32(rd)?)),
            0x22 => Ok(Op::TeeLocal(codec::read_varu32(rd)?)),
            0x23 => Ok(Op::GetGlobal(codec::read_varu32(rd)?)),
            0x24 => Ok(Op::SetGlobal(codec::read_varu32(rd)?)),
            0x28 => Ok(Op::I32Load(MemoryImmed::decode(rd)?)),
            0x29 => Ok(Op::I64Load(MemoryImmed::decode(rd)?)),
            0x2a => Ok(Op::F32Load(MemoryImmed::decode(rd)?)),
            0x2b => Ok(Op::F64Load(MemoryImmed::decode(rd)?)),
            0x2c => Ok(Op::I32Load8Signed(MemoryImmed::decode(rd)?)),
            0x2d => Ok(Op::I32Load8Unsigned(MemoryImmed::decode(rd)?)),
            0x2e => Ok(Op::I32Load16Signed(MemoryImmed::decode(rd)?)),
            0x2f => Ok(Op::I32Load16Unsigned(MemoryImmed::decode(rd)?)),
            0x30 => Ok(Op::I64Load8Signed(MemoryImmed::decode(rd)?)),
            0x31 => Ok(Op::I64Load8Unsigned(MemoryImmed::decode(rd)?)),
            0x32 => Ok(Op::I64Load16Signed(MemoryImmed::decode(rd)?)),
            0x33 => Ok(Op::I64Load16Unsigned(MemoryImmed::decode(rd)?)),
            0x34 => Ok(Op::I64Load32Signed(MemoryImmed::decode(rd)?)),
            0x35 => Ok(Op::I64Load32Unsigned(MemoryImmed::decode(rd)?)),
            0x36 => Ok(Op::I32Store(MemoryImmed::decode(rd)?)),
            0x37 => Ok(Op::I64Store(MemoryImmed::decode(rd)?)),
            0x38 => Ok(Op::F32Store(MemoryImmed::decode(rd)?)),
            0x39 => Ok(Op::F64Store(MemoryImmed::decode(rd)?)),
            0x3a => Ok(Op::I32Store8(MemoryImmed::decode(rd)?)),
            0x3b => Ok(Op::I32Store16(MemoryImmed::decode(rd)?)),
            0x3c => Ok(Op::I64Store8(MemoryImmed::decode(rd)?)),
            0x3d => Ok(Op::I64Store16(MemoryImmed::decode(rd)?)),
            0x3e => Ok(Op::I64Store32(MemoryImmed::decode(rd)?)),
            0x3f => Ok(Op::CurrentMemory(codec::read_varu1(rd)?)),
            0x40 => Ok(Op::GrowMemory(codec::read_varu1(rd)?)),
            0x41 => Ok(Op::I32Const(codec::read_vari32(rd)?)),
            0x42 => Ok(Op::I64Const(codec::read_vari64(rd)?)),
            0x43 => Ok(Op::F32Const(codec::read_u32(rd)? as f32)),
            0x44 => Ok(Op::F64Const(codec::read_u64(rd)? as f64)),
            0x45 => Ok(Op::I32Eqz),
            0x46 => Ok(Op::I32Eq),
            0x47 => Ok(Op::I32Ne),
            0x48 => Ok(Op::I32LtSigned),
            0x49 => Ok(Op::I32LtUnsigned),
            0x4a => Ok(Op::I32GtSigned),
            0x4b => Ok(Op::I32GtUnsigned),
            0x4c => Ok(Op::I32LeSigned),
            0x4d => Ok(Op::I32LeUnsigned),
            0x4e => Ok(Op::I32GeSigned),
            0x4f => Ok(Op::I32GeUnsigned),
            0x50 => Ok(Op::I64Eqz),
            0x51 => Ok(Op::I64Eq),
            0x52 => Ok(Op::I64Ne),
            0x53 => Ok(Op::I64LtSigned),
            0x54 => Ok(Op::I64LtUnsigned),
            0x55 => Ok(Op::I64GtSigned),
            0x56 => Ok(Op::I64GtUnsigned),
            0x57 => Ok(Op::I64LeSigned),
            0x58 => Ok(Op::I64LeUnsigned),
            0x59 => Ok(Op::I64GeSigned),
            0x5a => Ok(Op::I64GeUnsigned),
            0x5b => Ok(Op::F32Eq),
            0x5c => Ok(Op::F32Ne),
            0x5d => Ok(Op::F32Lt),
            0x5e => Ok(Op::F32Gt),
            0x5f => Ok(Op::F32Le),
            0x60 => Ok(Op::F32Ge),
            0x61 => Ok(Op::F64Eq),
            0x62 => Ok(Op::F64Ne),
            0x63 => Ok(Op::F64Lt),
            0x64 => Ok(Op::F64Gt),
            0x65 => Ok(Op::F64Le),
            0x66 => Ok(Op::F64Ge),
            0x67 => Ok(Op::I32Clz),
            0x68 => Ok(Op::I32Ctz),
            0x69 => Ok(Op::I32Popcnt),
            0x6a => Ok(Op::I32Add),
            0x6b => Ok(Op::I32Sub),
            0x6c => Ok(Op::I32Mul),
            0x6d => Ok(Op::I32DivSigned),
            0x6e => Ok(Op::I32DivUnsigned),
            0x6f => Ok(Op::I32RemSigned),
            0x70 => Ok(Op::I32RemUnsigned),
            0x71 => Ok(Op::I32And),
            0x72 => Ok(Op::I32Or),
            0x73 => Ok(Op::I32Xor),
            0x74 => Ok(Op::I32Shl),
            0x75 => Ok(Op::I32ShrSigned),
            0x76 => Ok(Op::I32ShrUnsigned),
            0x77 => Ok(Op::I32Rotl),
            0x78 => Ok(Op::I32Rotr),
            0x79 => Ok(Op::I64Clz),
            0x7a => Ok(Op::I64Ctz),
            0x7b => Ok(Op::I64Popcnt),
            0x7c => Ok(Op::I64Add),
            0x7d => Ok(Op::I64Sub),
            0x7e => Ok(Op::I64Mul),
            0x7f => Ok(Op::I64DivSigned),
            0x80 => Ok(Op::I64DivUnsigned),
            0x81 => Ok(Op::I64RemSigned),
            0x82 => Ok(Op::I64RemUnsigned),
            0x83 => Ok(Op::I64And),
            0x84 => Ok(Op::I64Or),
            0x85 => Ok(Op::I64Xor),
            0x86 => Ok(Op::I64Shl),
            0x87 => Ok(Op::I64ShrSigned),
            0x88 => Ok(Op::I64ShrUnsigned),
            0x89 => Ok(Op::I64Rotl),
            0x8a => Ok(Op::I64Rotr),
            0x8b => Ok(Op::F32Abs),
            0x8c => Ok(Op::F32Neg),
            0x8d => Ok(Op::F32Ceil),
            0x8e => Ok(Op::F32Floor),
            0x8f => Ok(Op::F32Trunc),
            0x90 => Ok(Op::F32Nearest),
            0x91 => Ok(Op::F32Sqrt),
            0x92 => Ok(Op::F32Add),
            0x93 => Ok(Op::F32Sub),
            0x94 => Ok(Op::F32Mul),
            0x95 => Ok(Op::F32Div),
            0x96 => Ok(Op::F32Min),
            0x97 => Ok(Op::F32Max),
            0x98 => Ok(Op::F32Copysign),
            0x99 => Ok(Op::F64Abs),
            0x9a => Ok(Op::F64Neg),
            0x9b => Ok(Op::F64Ceil),
            0x9c => Ok(Op::F64Floor),
            0x9d => Ok(Op::F64Trunc),
            0x9e => Ok(Op::F64Nearest),
            0x9f => Ok(Op::F64Sqrt),
            0xa0 => Ok(Op::F64Add),
            0xa1 => Ok(Op::F64Sub),
            0xa2 => Ok(Op::F64Mul),
            0xa3 => Ok(Op::F64Div),
            0xa4 => Ok(Op::F64Min),
            0xa5 => Ok(Op::F64Max),
            0xa6 => Ok(Op::F64Copysign),
            0xa7 => Ok(Op::I32WrapI64),
            0xa8 => Ok(Op::I32TruncSignedF32),
            0xa9 => Ok(Op::I32TruncUnsignedF32),
            0xaa => Ok(Op::I32TruncSignedF64),
            0xab => Ok(Op::I32TruncUnsignedF64),
            0xac => Ok(Op::I64ExtendSignedI32),
            0xad => Ok(Op::I64ExtendUnsignedI32),
            0xae => Ok(Op::I64TruncSignedF32),
            0xaf => Ok(Op::I64TruncUnsignedF32),
            0xb0 => Ok(Op::I64TruncSignedF64),
            0xb1 => Ok(Op::I64TruncUnsignedF64),
            0xb2 => Ok(Op::F32ConvertSignedI32),
            0xb3 => Ok(Op::F32ConvertUnsignedI32),
            0xb4 => Ok(Op::F32ConvertSignedI64),
            0xb5 => Ok(Op::F32ConvertUnsignedI64),
            0xb6 => Ok(Op::F32DemoteF64),
            0xb7 => Ok(Op::F64ConvertSignedI32),
            0xb8 => Ok(Op::F64ConvertUnsignedI32),
            0xb9 => Ok(Op::F64ConvertSignedI64),
            0xba => Ok(Op::F64ConvertUnsignedI64),
            0xbb => Ok(Op::F64PromoteF32),
            0xbc => Ok(Op::I32ReinterpretF32),
            0xbd => Ok(Op::I64ReinterpretF64),
            0xbe => Ok(Op::F32ReinterpretI32),
            0xbf => Ok(Op::F64ReinterpretI64),

            op => Err(CodecError::BadOpcode(op)),
        }
    }

    fn is_init_op(&self) -> bool {
        match *self {
            Op::I32Const(_) |
                Op::I64Const(_) |
                Op::F32Const(_) |
                Op::F64Const(_) |
                Op::GetGlobal(_) => true,
            _ => false
        }
    }
}

#[derive(Debug)]
pub struct InitExpr {
    ops: Vec<Op>,
}

impl InitExpr {
    pub fn decode(rd: &mut untrusted::Reader) -> Result<InitExpr, CodecError> {
        let mut ops = vec![];

        loop {
            let op = Op::decode(rd)?;

            if let Op::End = op {
                break;
            }

            if !op.is_init_op() {
                return Err(CodecError::BadInitExpr)
            }

            ops.push(op);
        }

        Ok(InitExpr { ops })
    }
}

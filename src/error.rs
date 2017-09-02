#[derive(Debug, PartialEq)]
pub enum CodecError {
    BadMagic,
    BadVersion,
    Unimpl,
    TrailingData,
    Truncated,
    BadVarInt,
    BadUTF8,
    BadType,
    BadOpcode(u8),
    BadInitExpr,
    BadFunctionEnd,
    BadOpArgs,
}

use mem;
use expr;
use value::Value;
use error::RuntimeError;

pub struct Context {
    mem: mem::Memory,
    stack: Vec<Value>,
}

macro_rules! mem_load {
    ($self:ident, $immed:ident, $access:tt, $value:path) => (
        $self.mem.$access(&$immed)
            .ok_or(RuntimeError::MemoryFault)
            .and_then(|v| $self.push($value(v)))
    );
}

macro_rules! mem_store {
    ($self:ident, $immed:ident, $access:tt, $pop:ident) => (
        $self.$pop()
            .and_then(|v| $self.mem.$access(v, &$immed)
                      .ok_or(RuntimeError::MemoryFault))
    );
}

macro_rules! stack_pop {
    ($name:ident, $ty:tt, $vty:path) => (
        fn $name(&mut self) -> Result<$ty, RuntimeError> {
            match self.stack.pop() {
                Some(v) => {
                    println!("pop {:?}", v);
                    match v {
                        $vty(vv) => Ok(vv),
                        _ => Err(RuntimeError::TypeFault),
                    }
                },
                None => Err(RuntimeError::StackUnderflow),
            }
        }
    );
}

/// Stack operation with one input and one output
macro_rules! unary {
    ($self:ident, $pop:ident, $vty:ident, $val:expr) => (
        $self.$pop()
             .map($val)
             .and_then(|v| $self.push(Value::$vty(v)))
    );
}

/// Stack operation with two inputs and one output
macro_rules! binary {
    ($self:ident, $poplhs:ident, $poprhs:ident, $vty:ident, $val:expr) => ({
        let lhs = $self.$poplhs()?;
        $self.$poprhs()
             .map(|rhs| $val(lhs, rhs))
             .and_then(|v| $self.push(Value::$vty(v)))
    });
}

/// Stack operation with two inputs and one output, that may trap.
macro_rules! binary_trap {
    ($self:ident, $poplhs:ident, $poprhs:ident, $vty:ident, $val:expr) => ({
        let lhs = $self.$poplhs()?;
        $self.$poprhs()
             .and_then(|rhs| $val(lhs, rhs))
             .and_then(|v| $self.push(Value::$vty(v)))
    });
}

fn div_i32(x: i32, y: i32) -> Result<i32, RuntimeError> {
    x.checked_div(y)
        .ok_or(RuntimeError::DivideByZero)
}

fn div_u32(x: i32, y: i32) -> Result<i32, RuntimeError> {
    (x as u32).checked_div(y as u32)
        .ok_or(RuntimeError::DivideByZero)
        .map(|u| u as i32)
}

fn rem_i32(x: i32, y: i32) -> Result<i32, RuntimeError> {
    x.checked_rem(y)
        .ok_or(RuntimeError::DivideByZero)
}

fn rem_u32(x: i32, y: i32) -> Result<i32, RuntimeError> {
    (x as u32).checked_rem(y as u32)
        .ok_or(RuntimeError::DivideByZero)
        .map(|u| u as i32)
}

fn div_i64(x: i64, y: i64) -> Result<i64, RuntimeError> {
    x.checked_div(y)
        .ok_or(RuntimeError::DivideByZero)
}

fn div_u64(x: i64, y: i64) -> Result<i64, RuntimeError> {
    (x as u64).checked_div(y as u64)
        .ok_or(RuntimeError::DivideByZero)
        .map(|u| u as i64)
}

fn rem_i64(x: i64, y: i64) -> Result<i64, RuntimeError> {
    x.checked_rem(y)
        .ok_or(RuntimeError::DivideByZero)
}

fn rem_u64(x: i64, y: i64) -> Result<i64, RuntimeError> {
    (x as u64).checked_rem(y as u64)
        .ok_or(RuntimeError::DivideByZero)
        .map(|u| u as i64)
}

impl Context {
    pub fn new() -> Context {
        let mut mem = mem::Memory::new();
        mem.grow(1);

        let stack = vec![];

        Context { mem, stack }
    }

    fn push(&mut self, v: Value) -> Result<(), RuntimeError> {
        // TODO: stack limit
        println!("push {:?}", v);
        self.stack.push(v);
        Ok(())
    }

    stack_pop!(pop_I32, i32, Value::I32);
    stack_pop!(pop_I64, i64, Value::I64);
    stack_pop!(pop_F32, f32, Value::F32);
    stack_pop!(pop_F64, f64, Value::F64);

    fn exec(&mut self, op: &expr::Op) -> Result<(), RuntimeError> {
        use expr::Op::*;

        match op {
            &Unreachable => Err(RuntimeError::Unreachable),
            &Nop => Ok(()),

            &I32Clz => unary!(self, pop_I32, I32, |i| i.leading_zeros() as i32),
            &I32Ctz => unary!(self, pop_I32, I32, |i| i.trailing_zeros() as i32),
            &I32Popcnt => unary!(self, pop_I32, I32, |i| i.count_ones() as i32),
            &I32Add => binary!(self, pop_I32, pop_I32, I32, |x, y| x + y),
            &I32Sub => binary!(self, pop_I32, pop_I32, I32, |x, y| x - y),
            &I32Mul => binary!(self, pop_I32, pop_I32, I32, |x, y| x * y),
            &I32DivSigned => binary_trap!(self, pop_I32, pop_I32, I32, div_i32),
            &I32DivUnsigned => binary_trap!(self, pop_I32, pop_I32, I32, div_u32),
            &I32RemSigned => binary_trap!(self, pop_I32, pop_I32, I32, rem_i32),
            &I32RemUnsigned => binary_trap!(self, pop_I32, pop_I32, I32, rem_u32),
            &I32And => binary!(self, pop_I32, pop_I32, I32, |x, y| x & y),
            &I32Or => binary!(self, pop_I32, pop_I32, I32, |x, y| x | y),
            &I32Xor => binary!(self, pop_I32, pop_I32, I32, |x, y| x ^ y),
            &I32Shl => binary!(self, pop_I32, pop_I32, I32,
                               |x: i32, y| x.wrapping_shl(y as u32)),
            &I32ShrSigned => binary!(self, pop_I32, pop_I32, I32, 
                                     |x: i32, y| x.wrapping_shr(y as u32)),
            &I32ShrUnsigned => binary!(self, pop_I32, pop_I32, I32,
                                       |x: i32, y| (x as u32).wrapping_shr(y as u32) as i32),
            &I32Rotl => binary!(self, pop_I32, pop_I32, I32,
                                |x: i32, y| x.rotate_left(y as u32 % 32)),
            &I32Rotr => binary!(self, pop_I32, pop_I32, I32,
                                |x: i32, y| x.rotate_right(y as u32 % 32)),

            &I64Clz => unary!(self, pop_I64, I64, |i| i.leading_zeros() as i64),
            &I64Ctz => unary!(self, pop_I64, I64, |i| i.trailing_zeros() as i64),
            &I64Popcnt => unary!(self, pop_I64, I64, |i| i.count_ones() as i64),
            &I64Add => binary!(self, pop_I64, pop_I64, I64, |x, y| x + y),
            &I64Sub => binary!(self, pop_I64, pop_I64, I64, |x, y| x - y),
            &I64Mul => binary!(self, pop_I64, pop_I64, I64, |x, y| x * y),
            &I64DivSigned => binary_trap!(self, pop_I64, pop_I64, I64, div_i64),
            &I64DivUnsigned => binary_trap!(self, pop_I64, pop_I64, I64, div_u64),
            &I64RemSigned => binary_trap!(self, pop_I64, pop_I64, I64, rem_i64),
            &I64RemUnsigned => binary_trap!(self, pop_I64, pop_I64, I64, rem_u64),
            &I64And => binary!(self, pop_I64, pop_I64, I64, |x, y| x & y),
            &I64Or => binary!(self, pop_I64, pop_I64, I64, |x, y| x | y),
            &I64Xor => binary!(self, pop_I64, pop_I64, I64, |x, y| x ^ y),
            &I64Shl => binary!(self, pop_I64, pop_I64, I64,
                               |x: i64, y| x.wrapping_shl(y as u32)),
            &I64ShrSigned => binary!(self, pop_I64, pop_I64, I64, 
                                     |x: i64, y| x.wrapping_shr(y as u32)),
            &I64ShrUnsigned => binary!(self, pop_I64, pop_I64, I64,
                                       |x: i64, y| (x as u64).wrapping_shr(y as u32) as i64),
            &I64Rotl => binary!(self, pop_I64, pop_I64, I64,
                                |x: i64, y| x.rotate_left(y as u32 % 64)),
            &I64Rotr => binary!(self, pop_I64, pop_I64, I64,
                                |x: i64, y| x.rotate_right(y as u32 % 64)),

            &F32Abs => unary!(self, pop_F32, F32, |f| f.abs()),
            &F32Neg => unary!(self, pop_F32, F32, |f| -f),
            &F32Ceil => unary!(self, pop_F32, F32, |f| f.ceil()),
            &F32Floor => unary!(self, pop_F32, F32, |f| f.floor()),
            &F32Trunc => unary!(self, pop_F32, F32, |f| f.trunc()),
            &F32Nearest => unary!(self, pop_F32, F32, |f| f.round()),
            &F32Sqrt => unary!(self, pop_F32, F32, |f| f.sqrt()),
            &F32Add => binary!(self, pop_F32, pop_F32, F32, |x, y| x + y),
            &F32Sub => binary!(self, pop_F32, pop_F32, F32, |x, y| x - y),
            &F32Mul => binary!(self, pop_F32, pop_F32, F32, |x, y| x * y),
            &F32Div => binary!(self, pop_F32, pop_F32, F32, |x, y| x / y),
            &F32Min => binary!(self, pop_F32, pop_F32, F32, |x: f32, y| x.min(y)),
            &F32Max => binary!(self, pop_F32, pop_F32, F32, |x: f32, y| x.max(y)),
            &F32Copysign => binary!(self, pop_F32, pop_F32, F32, |x, y: f32| x * x.signum() * y.signum()),

            &F64Abs => unary!(self, pop_F64, F64, |f| f.abs()),
            &F64Neg => unary!(self, pop_F64, F64, |f| -f),
            &F64Ceil => unary!(self, pop_F64, F64, |f| f.ceil()),
            &F64Floor => unary!(self, pop_F64, F64, |f| f.floor()),
            &F64Trunc => unary!(self, pop_F64, F64, |f| f.trunc()),
            &F64Nearest => unary!(self, pop_F64, F64, |f| f.round()),
            &F64Sqrt => unary!(self, pop_F64, F64, |f| f.sqrt()),
            &F64Add => binary!(self, pop_F64, pop_F64, F64, |x, y| x + y),
            &F64Sub => binary!(self, pop_F64, pop_F64, F64, |x, y| x - y),
            &F64Mul => binary!(self, pop_F64, pop_F64, F64, |x, y| x * y),
            &F64Div => binary!(self, pop_F64, pop_F64, F64, |x, y| x / y),
            &F64Min => binary!(self, pop_F64, pop_F64, F64, |x: f64, y| x.min(y)),
            &F64Max => binary!(self, pop_F64, pop_F64, F64, |x: f64, y| x.max(y)),
            &F64Copysign => binary!(self, pop_F64, pop_F64, F64, |x, y: f64| x * x.signum() * y.signum()),

            &I32WrapI64 => unary!(self, pop_I64, I32, |i| i as i32),

            &I32TruncSignedF32 => unary!(self, pop_F32, I32, |f| f.trunc() as i32),
            &I32TruncUnsignedF32 => unary!(self, pop_F32, I32, |f| f.trunc() as u32 as i32),
            &I32TruncSignedF64 => unary!(self, pop_F64, I32, |f| f.trunc() as i32),
            &I32TruncUnsignedF64 => unary!(self, pop_F64, I32, |f| f.trunc() as u32 as i32),

            &I64ExtendSignedI32 => unary!(self, pop_I32, I64, |i| i as i64),
            &I64ExtendUnsignedI32 => unary!(self, pop_I32, I64, |i| i as u32 as i64),
            &I64TruncSignedF32 => unary!(self, pop_F32, I64, |f| f.trunc() as i64),
            &I64TruncUnsignedF32 => unary!(self, pop_F32, I64, |f| f.trunc() as u64 as i64),
            &I64TruncSignedF64 => unary!(self, pop_F64, I64, |f| f.trunc() as i64),
            &I64TruncUnsignedF64 => unary!(self, pop_F64, I64, |f| f.trunc() as u64 as i64),

            &F32ConvertSignedI32 => unary!(self, pop_I32, F32, |i| i as f32),
            &F32ConvertUnsignedI32 => unary!(self, pop_I32, F32, |i| i as u32 as f32),
            &F32ConvertSignedI64 => unary!(self, pop_I64, F32, |i| i as f32),
            &F32ConvertUnsignedI64 => unary!(self, pop_I64, F32, |i| i as u64 as f32),
            &F32DemoteF64 => unary!(self, pop_F64, F32, |f| f as f32),

            &F64ConvertSignedI32 => unary!(self, pop_I32, F64, |i| i as f64),
            &F64ConvertUnsignedI32 => unary!(self, pop_I32, F64, |i| i as u32 as f64),
            &F64ConvertSignedI64 => unary!(self, pop_I64, F64, |i| i as f64),
            &F64ConvertUnsignedI64 => unary!(self, pop_I64, F64, |i| i as u64 as f64),
            &F64PromoteF32 => unary!(self, pop_F32, F64, |f| f as f64),

            &F32ReinterpretI32 => unary!(self, pop_I32, F32, |i| f32::from_bits(i as u32)),
            &F64ReinterpretI64 => unary!(self, pop_I64, F64, |i| f64::from_bits(i as u64)),
            &I32ReinterpretF32 => unary!(self, pop_F32, I32, |f| f.to_bits() as i32),
            &I64ReinterpretF64 => unary!(self, pop_F64, I64, |f| f.to_bits() as i64),

            &I32Const(c) => self.push(Value::I32(c)),
            &I64Const(c) => self.push(Value::I64(c)),
            &F32Const(c) => self.push(Value::F32(c)),
            &F64Const(c) => self.push(Value::F64(c)),

            &I32Load(ref immed) => mem_load!(self, immed, i32_load, Value::I32),
            &I64Load(ref immed) => mem_load!(self, immed, i64_load, Value::I64),
            &F32Load(ref immed) => mem_load!(self, immed, f32_load, Value::F32),
            &F64Load(ref immed) => mem_load!(self, immed, f64_load, Value::F64),
            &I32Load8Signed(ref immed) => mem_load!(self, immed, i32_load8_s, Value::I32),
            &I32Load8Unsigned(ref immed) => mem_load!(self, immed, i32_load8_u, Value::I32),
            &I32Load16Signed(ref immed) => mem_load!(self, immed, i32_load16_s, Value::I32),
            &I32Load16Unsigned(ref immed) => mem_load!(self, immed, i32_load16_u, Value::I32),
            &I64Load8Signed(ref immed) => mem_load!(self, immed, i64_load8_s, Value::I64),
            &I64Load8Unsigned(ref immed) => mem_load!(self, immed, i64_load8_u, Value::I64),
            &I64Load16Signed(ref immed) => mem_load!(self, immed, i64_load16_s, Value::I64),
            &I64Load16Unsigned(ref immed) => mem_load!(self, immed, i64_load16_u, Value::I64),
            &I64Load32Signed(ref immed) => mem_load!(self, immed, i64_load32_s, Value::I64),
            &I64Load32Unsigned(ref immed) => mem_load!(self, immed, i64_load32_u, Value::I64),

            &I32Store(ref immed) => mem_store!(self, immed, i32_store, pop_I32),
            &I64Store(ref immed) => mem_store!(self, immed, i64_store, pop_I64),
            &F32Store(ref immed) => mem_store!(self, immed, f32_store, pop_F32),
            &F64Store(ref immed) => mem_store!(self, immed, f64_store, pop_F64),
            &I32Store8(ref immed) => mem_store!(self, immed, i32_store8, pop_I32),
            &I32Store16(ref immed) => mem_store!(self, immed, i32_store16, pop_I32),
            &I64Store8(ref immed) => mem_store!(self, immed, i64_store8, pop_I64),
            &I64Store16(ref immed) => mem_store!(self, immed, i64_store16, pop_I64),
            &I64Store32(ref immed) => mem_store!(self, immed, i64_store32, pop_I64),

            &CurrentMemory(0) => {
                let pages = self.mem.len_pages() as i32;
                self.push(Value::I32(pages))
            },
            &GrowMemory(0) => {
                self.pop_I32()
                    .map(|v| self.mem.grow(v))
                    .and_then(|v| self.push(Value::I32(v)))
            },
            _ => Err(RuntimeError::Unimpl),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic() {
        let mut ctx = Context::new();
        ctx.exec(&expr::Op::I32Const(3))
            .unwrap();
        ctx.exec(&expr::Op::I32Store(expr::MemoryImmed { align: 0, offset: 0 }))
            .unwrap();
    }
}

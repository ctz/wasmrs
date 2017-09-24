use expr::MemoryImmed;
use byteorder::{ByteOrder, NativeEndian};

const PAGE_SIZE: usize = 64 * 1024;
const PAGE_SHIFT: usize = 16;

struct Page([u8; PAGE_SIZE]);

pub struct Memory {
    pages: Vec<Box<Page>>
}

macro_rules! load {
    ($name:ident, $result:ty, $size:expr, $conv:expr) => (
        pub fn $name(&self, immed: &MemoryImmed) -> Option<$result> {
            self.get(immed, $size)
                .map(|xs: &[u8]| $conv(xs) as $result)
        }
    );
}

macro_rules! store {
    ($name:ident, $type:ty, $size:expr, $conv:expr) => (
        pub fn $name(&mut self, value: $type, immed: &MemoryImmed) -> Option<()> {
            self.get_mut(immed, $size)
                .map(|xs: &mut [u8]| $conv(xs, value))
        }
    );
}

impl Memory {
    pub fn new() -> Memory {
        Memory { pages: vec![] }
    }

    pub fn len_pages(&self) -> usize {
        self.pages.len()
    }

    pub fn grow(&mut self, delta: i32) -> i32 {
        if delta < 0 {
            return -1;
        }

        let old_pages = self.pages.len();
        let new_pages = old_pages.checked_add(delta as usize);

        match new_pages {
            Some(_) => {
                for _ in 0..delta {
                    self.pages.push(Box::new(Page([0u8; PAGE_SIZE])));
                }
                old_pages as i32
            }

            None => -1
        }
    }

    fn get(&self, immed: &MemoryImmed, sz: usize) -> Option<&[u8]> {
        let page = (immed.offset >> PAGE_SHIFT) as usize;
        let offset = (immed.offset & 0xffff) as usize;
        let offset_end = offset + sz;

        if page < self.pages.len() && offset_end <= PAGE_SIZE {
            Some(&self.pages[page].0[offset..offset_end])
        } else {
            None
        }
    }

    fn get_mut(&mut self, immed: &MemoryImmed, sz: usize) -> Option<&mut [u8]> {
        let page = (immed.offset >> PAGE_SHIFT) as usize;
        let offset = (immed.offset & 0xffff) as usize;
        let offset_end = offset + sz;

        if page < self.pages.len() && offset_end <= PAGE_SIZE {
            Some(&mut self.pages[page].0[offset..offset_end])
        } else {
            None
        }
    }

    load!(i32_load8_s, i32, 1, |xs: &[u8]| xs[0] as i8);
    load!(i32_load8_u, i32, 1, |xs: &[u8]| xs[0] as u32);
    load!(i32_load16_s, i32, 2, NativeEndian::read_i16);
    load!(i32_load16_u, i32, 2, NativeEndian::read_u16);
    load!(i32_load, i32, 4, NativeEndian::read_i32);

    load!(i64_load8_s, i64, 1, |xs: &[u8]| xs[0] as i8);
    load!(i64_load8_u, i64, 1, |xs: &[u8]| xs[0] as u64);
    load!(i64_load16_s, i64, 2, NativeEndian::read_i16);
    load!(i64_load16_u, i64, 2, NativeEndian::read_u16);
    load!(i64_load32_s, i64, 4, NativeEndian::read_i32);
    load!(i64_load32_u, i64, 4, NativeEndian::read_u32);
    load!(i64_load, i64, 8, NativeEndian::read_i64);

    load!(f32_load, f32, 4, NativeEndian::read_f32);
    load!(f64_load, f64, 8, NativeEndian::read_f64);

    fn store8_32(xs: &mut [u8], value: i32) {
        xs[0] = (value as i8) as u8;
    }

    fn store16_32(xs: &mut [u8], value: i32) {
        NativeEndian::write_i16(xs, value as i16)
    }

    store!(i32_store8, i32, 1, Memory::store8_32);
    store!(i32_store16, i32, 2, Memory::store16_32);
    store!(i32_store, i32, 4, NativeEndian::write_i32);

    fn store8_64(xs: &mut [u8], value: i64) {
        xs[0] = (value as i8) as u8;
    }

    fn store16_64(xs: &mut [u8], value: i64) {
        NativeEndian::write_i16(xs, value as i16)
    }

    fn store32_64(xs: &mut [u8], value: i64) {
        NativeEndian::write_i32(xs, value as i32)
    }

    store!(i64_store8, i64, 1, Memory::store8_64);
    store!(i64_store16, i64, 2, Memory::store16_64);
    store!(i64_store32, i64, 4, Memory::store32_64);
    store!(i64_store, i64, 8, NativeEndian::write_i64);

    store!(f32_store, f32, 4, NativeEndian::write_f32);
    store!(f64_store, f64, 8, NativeEndian::write_f64);
}

#[cfg(test)]
mod test {
    use super::*;

    fn immed(offs: usize) -> MemoryImmed {
        MemoryImmed { align: 1, offset: offs as u32 }
    }

    #[test]
    fn test_load() {
        let mut m = Memory::new();
        m.grow(1);
        m.i32_store(0xffff, &immed(0));

        assert_eq!(Some(-1), m.i32_load8_s(&immed(0)));
        assert_eq!(Some(-1), m.i32_load8_s(&immed(1)));
        assert_eq!(Some(0), m.i32_load8_s(&immed(2)));
        assert_eq!(Some(0), m.i32_load8_s(&immed(3)));

        assert_eq!(Some(0xff), m.i32_load8_u(&immed(0)));
        assert_eq!(Some(0xff), m.i32_load8_u(&immed(1)));
        assert_eq!(Some(0), m.i32_load8_u(&immed(2)));
        assert_eq!(Some(0), m.i32_load8_u(&immed(3)));

        assert_eq!(Some(-1), m.i32_load16_s(&immed(0)));
        assert_eq!(Some(0), m.i32_load16_s(&immed(2)));

        assert_eq!(Some(0xffff), m.i32_load16_u(&immed(0)));
        assert_eq!(Some(0), m.i32_load16_u(&immed(2)));

        assert_eq!(Some(0xffff), m.i32_load(&immed(0)));
        assert_eq!(Some(0xff), m.i32_load(&immed(1)));

        assert_eq!(Some(-1), m.i64_load8_s(&immed(0)));
        assert_eq!(Some(-1), m.i64_load8_s(&immed(1)));
        assert_eq!(Some(0), m.i64_load8_s(&immed(2)));
        assert_eq!(Some(0), m.i64_load8_s(&immed(3)));

        assert_eq!(Some(0xff), m.i64_load8_u(&immed(0)));
        assert_eq!(Some(0xff), m.i64_load8_u(&immed(1)));
        assert_eq!(Some(0), m.i64_load8_u(&immed(2)));
        assert_eq!(Some(0), m.i64_load8_u(&immed(3)));

        assert_eq!(Some(-1), m.i64_load16_s(&immed(0)));
        assert_eq!(Some(0), m.i64_load16_s(&immed(2)));

        assert_eq!(Some(0xffff), m.i64_load16_u(&immed(0)));
        assert_eq!(Some(0), m.i64_load16_u(&immed(2)));

        assert_eq!(Some(0xffff), m.i64_load32_s(&immed(0)));
        assert_eq!(Some(0), m.i64_load32_s(&immed(2)));

        assert_eq!(Some(0xffff), m.i64_load32_u(&immed(0)));
        assert_eq!(Some(0), m.i64_load32_u(&immed(2)));

        assert_eq!(Some(0xffff), m.i64_load(&immed(0)));
        assert_eq!(Some(0xff), m.i64_load(&immed(1)));
    }
}

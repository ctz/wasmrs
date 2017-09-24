extern crate untrusted;
extern crate byteorder;

mod error;
mod codec;
mod expr;
mod function;
mod section;
mod ty;
mod mem;
mod value;
mod exec;

#[cfg(test)]
mod tests {
    extern crate atoms;
    /*
    #[test]
    fn read_wast() {
        let input = include_bytes!("../corpus/tee_local.wast").as_ref();
        let mut parser = atoms::Parser::new(&input);
        let mut i = 0;
        loop {
            match parser.read::<String>() {
                Ok(v) => println!("{:?} {:?}", i, v),
                Err(e) => { println!("done {:?}", e); break }
            }
            i += 1;
        }
    }
    */

    #[test]
    fn read_binary() {
        let input = include_bytes!("../webdsp_c.wasm").as_ref();
        let module = super::section::Module::decode_from(&input);
        println!("{:?}", module);
    }
}

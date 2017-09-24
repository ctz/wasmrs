use function::FunctionBody;
use expr::InitExpr;
use ty::{ValueType, ElementType};
use error::CodecError;
use codec;

use untrusted;

#[derive(Debug)]
enum ResizableLimits {
    Initial(u32),
    InitialMax(u32, u32),
}

impl ResizableLimits {
    pub fn decode(rd: &mut untrusted::Reader) -> Result<ResizableLimits, CodecError> {
        let flags = codec::read_varu1(rd)?;
        let initial = codec::read_varu32(rd)?;
        if flags == 1 {
            let max = codec::read_varu32(rd)?;
            Ok(ResizableLimits::InitialMax(initial, max))
        } else {
            Ok(ResizableLimits::Initial(initial))
        }
    }
}

#[derive(Debug)]
struct CustomSection<'a> {
    name: &'a str,
    payload: &'a [u8],
}

#[derive(Debug)]
struct FunctionType {
    params: Vec<ValueType>,
    ret: Option<ValueType>,
}

impl FunctionType {
    pub fn decode(rd: &mut untrusted::Reader) -> Result<FunctionType, CodecError> {
        let form = codec::read_varu7(rd)?;

        let param_count = codec::read_varu32(rd)?;
        let mut params = vec![];
        for _ in 0..param_count {
            params.push(ValueType::decode(rd)?);
        }

        let return_count = codec::read_varu1(rd)?;
        let ret = if return_count == 1 {
            Some(ValueType::decode(rd)?)
        } else {
            None
        };

        Ok(FunctionType { params, ret })
    }
}

#[derive(Debug)]
struct TableType {
    element_ty: ElementType,
    limits: ResizableLimits,
}

impl TableType {
    pub fn decode(rd: &mut untrusted::Reader) -> Result<TableType, CodecError> {
        let element_ty = ElementType::decode(rd)?;
        let limits = ResizableLimits::decode(rd)?;
        Ok(TableType { element_ty, limits })
    }
}

#[derive(Debug)]
struct MemoryType {
    limits: ResizableLimits,
}

impl MemoryType {
    pub fn decode(rd: &mut untrusted::Reader) -> Result<MemoryType, CodecError> {
        ResizableLimits::decode(rd)
            .map(|limits| MemoryType { limits })
    }
}

#[derive(Debug)]
struct GlobalType {
    content: ValueType,
    mutable: bool,
}

impl GlobalType {
    pub fn decode(rd: &mut untrusted::Reader) -> Result<GlobalType, CodecError> {
        let content = ValueType::decode(rd)?;
        let mutable = if codec::read_varu1(rd)? == 1 { true } else { false };
        Ok(GlobalType { content, mutable })
    }
}

#[derive(Debug)]
enum ImportKind {
    Function(u32),
    Table(TableType),
    Memory(MemoryType),
    Global(GlobalType),
}

impl ImportKind {
    pub fn decode(rd: &mut untrusted::Reader) -> Result<ImportKind, CodecError> {
        let kind = ExternalKind::decode(rd)?;

        match kind {
            ExternalKind::Function => {
                Ok(ImportKind::Function(codec::read_varu32(rd)?))
            }
            ExternalKind::Table => {
                Ok(ImportKind::Table(TableType::decode(rd)?))
            }
            ExternalKind::Memory => {
                Ok(ImportKind::Memory(MemoryType::decode(rd)?))
            }
            ExternalKind::Global => {
                Ok(ImportKind::Global(GlobalType::decode(rd)?))
            }
        }
    }
}

#[derive(Debug)]
struct ImportEntry<'a> {
    module: &'a str,
    field: &'a str,
    kind: ImportKind,
}

impl<'a> ImportEntry<'a> {
    pub fn decode(rd: &mut untrusted::Reader<'a>) -> Result<ImportEntry<'a>, CodecError> {
        let mod_len = codec::read_varu32(rd)?;
        let module = codec::read_utf8(rd, mod_len as usize)?;
        let field_len = codec::read_varu32(rd)?;
        let field = codec::read_utf8(rd, field_len as usize)?;
        let kind = ImportKind::decode(rd)?;

        Ok(ImportEntry { module, field, kind })
    }
}

#[derive(Debug)]
struct GlobalVariable {
    ty: GlobalType,
    init: InitExpr,
}

impl GlobalVariable {
    pub fn decode(rd: &mut untrusted::Reader) -> Result<GlobalVariable, CodecError> {
        let ty = GlobalType::decode(rd)?;
        let init = InitExpr::decode(rd)?;
        Ok(GlobalVariable { ty, init })
    }
}

#[derive(Debug)]
enum ExternalKind {
    Function,
    Table,
    Memory,
    Global,
}

impl ExternalKind {
    pub fn decode(rd: &mut untrusted::Reader) -> Result<ExternalKind, CodecError> {
        let k = codec::read_u8(rd)?;

        match k {
            0 => Ok(ExternalKind::Function),
            1 => Ok(ExternalKind::Table),
            2 => Ok(ExternalKind::Memory),
            3 => Ok(ExternalKind::Global),
            _ => Err(CodecError::BadType),
        }
    }
}

#[derive(Debug)]
struct ExportEntry<'a> {
    field: &'a str,
    kind: ExternalKind,
    index: u32,
}

impl<'a> ExportEntry<'a> {
    pub fn decode(rd: &mut untrusted::Reader<'a>) -> Result<ExportEntry<'a>, CodecError> {
        let field_len = codec::read_varu32(rd)?;
        let field = codec::read_utf8(rd, field_len as usize)?;
        let kind = ExternalKind::decode(rd)?;
        let index = codec::read_varu32(rd)?;

        Ok(ExportEntry { field, kind, index })
    }
}

#[derive(Debug)]
struct ElementSegment {
    index: u32,
    offset: InitExpr,
    elems: Vec<u32>,
}

impl ElementSegment {
    pub fn decode(rd: &mut untrusted::Reader) -> Result<ElementSegment, CodecError> {
        let index = codec::read_varu32(rd)?;
        let offset = InitExpr::decode(rd)?;
        let count = codec::read_varu32(rd)?;

        let mut elems = vec![];
        for _ in 0..count {
            elems.push(codec::read_varu32(rd)?);
        }
        Ok(ElementSegment { index, offset, elems })
    }
}

#[derive(Debug)]
struct DataSegment<'a> {
    index: u32,
    init: InitExpr,
    data: &'a [u8],
}

impl<'a> DataSegment<'a> {
    pub fn decode(rd: &mut untrusted::Reader<'a>) -> Result<DataSegment<'a>, CodecError> {
        let index = codec::read_varu32(rd)?;
        let init = InitExpr::decode(rd)?;

        let size = codec::read_varu32(rd)?;
        let data = rd.skip_and_get_input(size as usize)
            .map_err(|_| CodecError::Truncated)
            .map(|inp| inp.as_slice_less_safe())?;

        Ok(DataSegment { index, init, data })
    }
}

#[derive(Debug)]
enum Section<'a> {
    Custom(CustomSection<'a>),
    Type(Vec<FunctionType>),
    Import(Vec<ImportEntry<'a>>),
    Function(Vec<u32>),
    Table(Vec<TableType>),
    Memory(Vec<MemoryType>),
    Global(Vec<GlobalVariable>),
    Export(Vec<ExportEntry<'a>>),
    Start(u32),
    Element(Vec<ElementSegment>),
    Code(Vec<FunctionBody>),
    Data(Vec<DataSegment<'a>>),
}

const SECTIONID_CUSTOM: u8 = 0;
const SECTIONID_TYPE: u8 = 1;
const SECTIONID_IMPORT: u8 = 2;
const SECTIONID_FUNCTION: u8 = 3;
const SECTIONID_TABLE: u8 = 4;
const SECTIONID_MEMORY: u8 = 5;
const SECTIONID_GLOBAL: u8 = 6;
const SECTIONID_EXPORT: u8 = 7;
const SECTIONID_START: u8 = 8;
const SECTIONID_ELEMENT: u8 = 9;
const SECTIONID_CODE: u8 = 10;
const SECTIONID_DATA: u8 = 11;

impl<'a> Section<'a> {
    pub fn decode(rd: &mut untrusted::Reader<'a>) -> Result<Section<'a>, CodecError> {
        let id = codec::read_varu7(rd)?;
        let len = codec::read_varu32(rd)?;
        let payload = rd.skip_and_get_input(len as usize)
            .map_err(|_| CodecError::Truncated)?;
        let mut prd = untrusted::Reader::new(payload);

        let section = match id {
            SECTIONID_CUSTOM => {
                let namelen = codec::read_varu32(&mut prd)?;
                let name = codec::read_utf8(&mut prd, namelen as usize)?;
                let payload = prd.skip_to_end().as_slice_less_safe();

                Ok(Section::Custom(CustomSection { name, payload }))
            }
            SECTIONID_TYPE => {
                let count = codec::read_varu32(&mut prd)?;
                let mut func_types = vec![];
                for _ in 0..count {
                    func_types.push(FunctionType::decode(&mut prd)?);
                }
                Ok(Section::Type(func_types))
            }
            SECTIONID_IMPORT => {
                let count = codec::read_varu32(&mut prd)?;
                let mut imports = vec![];
                for _ in 0..count {
                    imports.push(ImportEntry::decode(&mut prd)?);
                }
                Ok(Section::Import(imports))
            }
            SECTIONID_FUNCTION => {
                let count = codec::read_varu32(&mut prd)?;
                let mut funcs = vec![];
                for _ in 0..count {
                    funcs.push(codec::read_varu32(&mut prd)?);
                }
                Ok(Section::Function(funcs))
            }
            SECTIONID_TABLE => {
                let count = codec::read_varu32(&mut prd)?;
                let mut tables = vec![];
                for _ in 0..count {
                    tables.push(TableType::decode(&mut prd)?);
                }
                Ok(Section::Table(tables))
            }
            SECTIONID_MEMORY => {
                let count = codec::read_varu32(&mut prd)?;
                let mut memories = vec![];
                for _ in 0..count {
                    memories.push(MemoryType::decode(&mut prd)?);
                }
                Ok(Section::Memory(memories))
            }
            SECTIONID_GLOBAL => {
                let count = codec::read_varu32(&mut prd)?;
                let mut globals = vec![];
                for _  in 0..count {
                    globals.push(GlobalVariable::decode(&mut prd)?);
                }
                Ok(Section::Global(globals))
            }
            SECTIONID_EXPORT => {
                let count = codec::read_varu32(&mut prd)?;
                let mut exports = vec![];
                for _  in 0..count {
                    exports.push(ExportEntry::decode(&mut prd)?);
                }
                Ok(Section::Export(exports))
            }
            SECTIONID_START => {
                let index = codec::read_varu32(&mut prd)?;
                Ok(Section::Start(index))
            }
            SECTIONID_ELEMENT => {
                let count = codec::read_varu32(&mut prd)?;
                let mut elements = vec![];
                for _  in 0..count {
                    elements.push(ElementSegment::decode(&mut prd)?);
                }
                Ok(Section::Element(elements))
            }
            SECTIONID_CODE => {
                let count = codec::read_varu32(&mut prd)?;
                let mut funcs = vec![];
                for _  in 0..count {
                    funcs.push(FunctionBody::decode(&mut prd)?);
                }
                Ok(Section::Code(funcs))
            }
            SECTIONID_DATA => {
                let count = codec::read_varu32(&mut prd)?;
                let mut datas = vec![];
                for _  in 0..count {
                    datas.push(DataSegment::decode(&mut prd)?);
                }
                Ok(Section::Data(datas))
            }
            _ => {
                println!("sec type {:?} unimpl", id);
                Err(CodecError::Unimpl)
            }
        };
        section
    }
}

#[derive(Debug)]
pub struct Module<'a> {
    sections: Vec<Section<'a>>
}

impl<'a> Module<'a> {
    pub fn decode(rd: &mut untrusted::Reader<'a>) -> Result<Module<'a>, CodecError> {
        if codec::read_u32(rd)? != 0x6d736100 {
            return Err(CodecError::BadMagic);
        }

        if codec::read_u32(rd)? != 1 {
            return Err(CodecError::BadVersion);
        }

        let mut sections = vec![];

        while !rd.at_end() {
            sections.push(Section::decode(rd)?);
        }

        Ok(Module { sections })
    }

    pub fn decode_from(bytes: &'a [u8]) -> Result<Module<'a>, CodecError> {
        let inp = untrusted::Input::from(bytes);
        inp.read_all(
            CodecError::TrailingData,
            Module::decode
        )
    }
}

use nom::bytes::complete::{take, take_while};
use nom::IResult;
use nom::multi::count;
use nom::number::complete::{f32, i32, u32, u64, u8};
use nom::number::Endianness;
use nom::sequence::tuple;
use crate::parser::Pointer;
use crate::sector::Sector;

#[derive(Debug, PartialEq)]
pub struct Element {
    pub name: String,
    pub element: ElementType
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Transform {
    pub flags: u32,
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale_shear: [[f32; 3]; 3]
}

#[derive(Debug, PartialEq)]
pub enum ElementType {
    /// A list of elements
    Reference(Vec<Element>),
    ArrayOfReferences(Vec<Vec<Element>>),
    /// A pointer to raw data
    /// NOTE: This is currently not supported by this library
    VariantReference,
    /// A string
    String(String),
    /// A real value (aka float 32)
    F32(f32),
    /// An unsigned 8 bit integer
    U8(u8),
    /// A signed 32 bit integer
    I32(i32),
    ///
    Transform(Transform),

    /// Not really an element type and instead it's an array inside the element
    Array(Vec<ElementType>)
}

#[derive(Debug, PartialEq)]
pub struct TypeInfo {
    type_id: u32,
    name_offset: Option<Pointer>,
    children_offset: Option<Pointer>,
    array_size: i32,
}

fn unsigned(is_64bits: bool, endianness: Endianness) -> impl FnMut(&[u8]) -> IResult<&[u8], u64> {
    move |input| {
        if is_64bits {
            u64(endianness)(input)
        } else {
            let (input, val) = u32(endianness)(input)?;
            Ok((input, val as u64))
        }
    }
}

pub fn parse_type_info(endianness: Endianness, type_sector: &Sector, is_64bits: bool, offset: u32) -> impl FnMut(&[u8]) -> IResult<&[u8], TypeInfo> + '_ {
    move |input| {
        let type_id = u32(endianness); // 0
        let name_offset = unsigned(is_64bits, endianness); // 4
        let children_offset = unsigned(is_64bits, endianness); // 8/12
        let array_size = i32(endianness);

        let (input, (type_id, _, _, array_size)) = tuple((type_id, name_offset, children_offset, array_size))(input)?;

        let (input, _) = take(if is_64bits { 20usize } else { 16usize })(input)?;

        let name_offset = type_sector.resolve_pointer((offset + 4) as _);
        let children_offset = type_sector.resolve_pointer((offset + if is_64bits { 12 } else { 8 }) as _);

        Ok((input, TypeInfo {
            type_id,
            name_offset,
            children_offset,
            array_size
        }))
    }
}

fn parse_string(input: &[u8]) -> IResult<&[u8], String> {
    let (input, bytes) = take_while(|n| { n != 0x0 })(input)?;

    Ok((input, std::str::from_utf8(bytes).expect("invalid string").to_string()))
}

pub fn parse_element(endianness: Endianness, is_64bits: bool, sectors: &Vec<Sector>, data_sector_id: u32, type_sector_id: u32, data_offset: u32, type_offset: u32) -> IResult<&[u8], Vec<Element>> {
    let data_sector = &sectors[data_sector_id as usize];
    let type_sector = &sectors[type_sector_id as usize];

    let all_type_data = &*type_sector.data;
    let all_data = &*data_sector.data;

    let mut type_data = &all_type_data[type_offset as usize..];
    let mut data = &all_data[data_offset as usize..];

    let mut elements = Vec::new();

    loop {
        let (next, type_info) = parse_type_info(endianness, type_sector, is_64bits, (all_type_data.len() - type_data.len()) as u32)(type_data)?;
        if type_info.type_id == 0 || type_info.type_id > 22 {
            break
        }

        let name = if let Some(name_offset) = type_info.name_offset {
            let (_, name) = parse_string(&sectors[name_offset.dst_sector as usize].data[name_offset.dst_offset as usize..])?;
            name
        } else {
            "".to_string()
        };

        let element = if type_info.array_size > 0 {
            let mut inners = Vec::new();
            for _ in 0..(if type_info.array_size == 0 { 1 } else { type_info.array_size }) {
                let (next, element_inner) = parse_element_data(endianness, is_64bits, sectors, data_sector_id, all_data, data, &type_info)?;
                data = next;

                inners.push(element_inner);
            }

            Element {
                name,
                element: ElementType::Array(inners)
            }
        } else {
            let (next, element_inner) = parse_element_data(endianness, is_64bits, sectors, data_sector_id, all_data, data, &type_info)?;
            data = next;

            Element {
                name,
                element: element_inner
            }
        };

        elements.push(element);
        type_data = next;
    }

    Ok((data, elements))
}

fn parse_element_data<'a>(endianness: Endianness, is_64bits: bool, sectors: &'a Vec<Sector>, data_sector_id: u32, all_data: &[u8], mut data: &'a [u8], type_info: &TypeInfo) -> IResult<&'a [u8], ElementType> {
    let data_sector = &sectors[data_sector_id as usize];
    match type_info.type_id {
        1 => {
            Ok((data, ElementType::VariantReference))
        }
        2 => {
            let pos = all_data.len() - data.len();
            let (next, _) = unsigned(is_64bits, endianness)(data)?;

            data = next;

            let ptr = data_sector.resolve_pointer(pos);
            let elements = if let Some(ptr) = ptr {
                assert_eq!(ptr.dst_sector, data_sector_id);

                let children_offset = type_info.children_offset.unwrap();

                let (_, elements) = parse_element(
                    endianness,
                    is_64bits,
                    sectors,
                    ptr.dst_sector,
                    children_offset.dst_sector,
                    ptr.dst_offset,
                    children_offset.dst_offset
                )?;

                elements
            } else {
                Vec::new()
            };

            Ok((data, ElementType::Reference(elements)))
        }
        3 => {
            let pos = all_data.len() - data.len() + 4;
            let size = u32(endianness);
            let offset = unsigned(is_64bits, endianness);

            let (next, (size, _)) = tuple((size, offset))(data)?;
            data = next;

            let mut elements = Vec::new();

            let data_ptr = data_sector.resolve_pointer(pos);
            if size > 0 && data_ptr.is_some() {
                if let Some(data_ptr) = data_ptr {
                    let type_ptr = type_info.children_offset.unwrap();

                    let data_sector = &sectors[data_ptr.dst_sector as usize];

                    let mut data_offset = data_ptr.dst_offset;
                    for _ in 0..size {
                        let (left_data, mut e) = parse_element(endianness, is_64bits, sectors, data_ptr.dst_sector, type_ptr.dst_sector, data_offset, type_ptr.dst_offset)?;
                        elements.append(&mut e);

                        data_offset = (data_sector.data.len() - left_data.len()) as u32;
                    }
                }
            }

            Ok((data, ElementType::Reference(elements)))
        }
        4 => {
            let pos = all_data.len() - data.len() + 4;
            let size = u32(endianness);
            let offset = unsigned(is_64bits, endianness);

            let (next, (size, _)) = tuple((size, offset))(data)?;
            data = next;

            let ptr = data_sector.resolve_pointer(pos);

            let mut references = Vec::new();

            if let Some(ptr) = ptr {
                let type_ptr = type_info.children_offset.unwrap();

                let element_data_sector = &sectors[ptr.dst_sector as usize];

                for i in 0..size {
                    let element_ptr = element_data_sector.resolve_pointer((ptr.dst_offset + if is_64bits { 8 * i } else { 4 * i }) as usize).unwrap();

                    let (_, e) = parse_element(endianness, is_64bits, sectors, element_ptr.dst_sector, type_ptr.dst_sector, element_ptr.dst_offset, type_ptr.dst_offset)?;

                    references.push(e);
                }
            }

            Ok((data, ElementType::ArrayOfReferences(references)))
        }
        5 => {
            let offset = unsigned(is_64bits, endianness);
            let data_ptr = unsigned(is_64bits, endianness);

            let (next, (_, _)) = tuple((offset, data_ptr))(data)?;
            data = next;

            Ok((data, ElementType::VariantReference))
        }
        7 => {
            let pos = all_data.len() - data.len();

            let type_ptr = unsigned(is_64bits, endianness);
            let size = u32(endianness);
            let data_ptr = unsigned(is_64bits, endianness);

            let (next, (_, size, _)) = tuple((type_ptr, size, data_ptr))(data)?;
            data = next;

            let type_ptr = data_sector.resolve_pointer(pos).unwrap();
            let data_ptr = data_sector.resolve_pointer(pos + if is_64bits { 8 + 4 } else { 4 + 4 }).unwrap();

            let mut data_offset = data_ptr.dst_offset;

            let mut elements = Vec::new();

            for _ in 0..size {
                let (left_data, e) = parse_element(endianness, is_64bits, sectors, data_ptr.dst_sector, type_ptr.dst_sector, data_offset, type_ptr.dst_offset)?;

                data_offset = (sectors[data_ptr.dst_sector as usize].data.len() - left_data.len()) as _;

                elements.push(e);
            }

            Ok((data, ElementType::ArrayOfReferences(elements)))
        }
        8 => {
            let pos = all_data.len() - data.len();
            let (next, _) = unsigned(is_64bits, endianness)(data)?;

            data = next;

            let ptr = data_sector.resolve_pointer(pos).unwrap();
            let (_, value) = parse_string(&sectors[ptr.dst_sector as usize].data[ptr.dst_offset as usize..])?;

            Ok((data, ElementType::String(value)))
        }
        9 => {
            let flags = u32(endianness);
            let translation = count(f32(endianness), 3);
            let rotation = count(f32(endianness), 4);
            let scale_shear = count(f32(endianness), 3 * 3);

            let (next, (flags, translation, rotation, scale_shear)) = tuple((flags, translation, rotation, scale_shear))(data)?;
            data = next;

            Ok((data, ElementType::Transform(Transform {
                flags,
                translation: [translation[0], translation[1], translation[2]],
                rotation: [rotation[0], rotation[1], rotation[2], rotation[3]],
                scale_shear: [
                    [scale_shear[0], scale_shear[1], scale_shear[2]],
                    [scale_shear[3], scale_shear[4], scale_shear[5]],
                    [scale_shear[6], scale_shear[7], scale_shear[8]],
                ]
            })))
        }
        10 => {
            let (next, val) = f32(endianness)(data)?;
            data = next;

            Ok((data, ElementType::F32(val)))
        },
        12 | 14 => {
            let (next, val) = u8(data)?;
            data = next;

            Ok((data, ElementType::U8(val)))
        }
        19 => {
            let (next, val) = i32(endianness)(data)?;
            data = next;

            Ok((data, ElementType::I32(val)))
        }
        _ => {
            panic!("Unknown element type id {}", type_info.type_id);
        }
    }
}
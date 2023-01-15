use nom::bytes::complete::{take, take_while1};
use nom::character::is_alphanumeric;
use nom::IResult;
use nom::number::complete::{f32, i32, u32, u64};
use nom::number::Endianness;
use nom::sequence::tuple;
use crate::parser::Pointer;
use crate::sector::Sector;

#[derive(Debug, PartialEq)]
pub struct Element {
    name: String,
    element: ElementType
}

#[derive(Debug, PartialEq)]
pub enum ElementType {
    Reference(Vec<Element>),
    String(String),
    F32(f32),
    I32(i32),

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

pub fn parse_type_info(endianness: Endianness, ptr_table: &Vec<Pointer>, is_64bits: bool, offset: u32) -> impl FnMut(&[u8]) -> IResult<&[u8], TypeInfo> + '_ {
    move |input| {
        let unsigned = |input| {
            return if is_64bits {
                u64(endianness)(input)
            } else {
                let (input, val) = u32(endianness)(input)?;
                Ok((input, val as u64))
            }
        };

        let type_id = u32(endianness); // 0
        let name_offset = |i| { unsigned(i) }; // 4
        let children_offset = |i| { unsigned(i) }; // 8, 12
        let array_size = i32(endianness);

        let (input, (type_id, _, _, array_size)) = tuple((type_id, name_offset, children_offset, array_size))(input)?;

        let (input, _) = take(if is_64bits { 20usize } else { 16usize })(input)?;

        let name_offset = ptr_table.iter().find(|ptr| ptr.src_offset == offset + 4).map(|ptr| ptr.clone());
        let children_offset = ptr_table.iter().find(|ptr| ptr.src_offset == offset + if is_64bits { 12 } else { 8 }).map(|ptr| ptr.clone());

        Ok((input, TypeInfo {
            type_id,
            name_offset,
            children_offset,
            array_size
        }))
    }
}

fn parse_string(input: &[u8]) -> IResult<&[u8], String> {
    let (input, bytes) = take_while1(|n| { n != 0x0 })(input)?;

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
        let (next, type_info) = parse_type_info(endianness, &type_sector.pointer_table, is_64bits, (all_type_data.len() - type_data.len()) as u32)(type_data)?;
        if type_info.type_id == 0 || type_info.type_id > 22 {
            break
        }

        println!("{:?}", type_info);

        let name = if let Some(name_offset) = type_info.name_offset {
            let (_, name) = parse_string(&sectors[name_offset.dst_sector as usize].data[name_offset.dst_offset as usize..])?;
            name
        } else {
            "".to_string()
        };

        let element = if type_info.array_size > 0 {
            let mut inners = Vec::new();
            for _ in 0..(if type_info.array_size == 0 { 1 } else { type_info.array_size }) {
                let (next, element_inner) = parse_element_data(endianness, is_64bits, &sectors, data_sector_id, data_sector, all_data, data, &type_info)?;
                data = next;

                inners.push(element_inner);
            }

            Element {
                name,
                element: ElementType::Array(inners)
            }
        } else {
            let (next, element_inner) = parse_element_data(endianness, is_64bits, &sectors, data_sector_id, data_sector, all_data, data, &type_info)?;
            data = next;

            Element {
                name,
                element: element_inner
            }
        };

        println!("{:?}", element);
        println!("===========");

        elements.push(element);
        type_data = next;
    }

    Ok((&*data_sector.data, elements))
}

fn parse_element_data<'a>(endianness: Endianness, is_64bits: bool, sectors: &'a Vec<Sector>, data_sector_id: u32, data_sector: &Sector, all_data: &[u8], mut data: &'a [u8], type_info: &TypeInfo) -> IResult<&'a [u8], ElementType> {
    match type_info.type_id {
        2 => {
            let pos = all_data.len() - data.len();
            let (next, _) = if is_64bits {
                u64(endianness)(data)?
            } else {
                let (data, ptr) = u32(endianness)(data)?;
                (data, ptr as u64)
            };

            data = next;

            let ptr = data_sector.pointer_table.iter().find(|p| p.src_offset as usize == pos).unwrap().clone();
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

            println!("Elements: {:?}", elements);

            Ok((data, ElementType::Reference(elements)))
        },
        8 => {
            let pos = all_data.len() - data.len();
            let (next, _) = if is_64bits {
                u64(endianness)(data)?
            } else {
                let (data, ptr) = u32(endianness)(data)?;
                (data, ptr as u64)
            };

            data = next;

            let ptr = data_sector.pointer_table.iter().find(|p| p.src_offset as usize == pos).unwrap().clone();
            let (_, value) = parse_string(&sectors[ptr.dst_sector as usize].data[ptr.dst_offset as usize..])?;

            Ok((data, ElementType::String(value)))
        }
        10 => {
            let (next, val) = f32(endianness)(data)?;
            data = next;

            Ok((data, ElementType::F32(val)))
        },
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
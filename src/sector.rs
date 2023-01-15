use nom::number::Endianness;
use crate::decompression::decompress_sector;
use crate::parser::{parse_pointer, Pointer, SectorInfo};

#[derive(Debug)]
pub struct Sector {
    pub info: SectorInfo,
    pub data: Vec<u8>,
    pub pointer_table: Vec<Pointer>
}

pub fn load_sector(input: &[u8], endianness: Endianness, info: SectorInfo) -> Sector {
    let data = decompress_sector(input, &info);
    let mut pointer_table = Vec::new();

    let mut fixup_input = &input[info.fixup_offset as usize..];
    for _ in 0..info.fixup_size {
        let res = parse_pointer(endianness)(fixup_input);
        if let Ok((next, pointer)) = res {
            pointer_table.push(pointer);

            fixup_input = next
        } else {
            panic!("Failed to load fixup table for sector");
        }
    }

    Sector {
        info,
        data,
        pointer_table,
    }
}
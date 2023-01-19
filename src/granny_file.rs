use std::fs::File;
use std::io::Read;
use nom::number::Endianness;
use crate::granny_path::GrannyResolve;
use crate::parser::{Element, ElementType, parse_element, parse_file_info, parse_header, parse_sector_info};
use crate::sector::load_sector;

pub struct GrannyFile {
    root_elements: Vec<Element>
}

impl GrannyFile {
    pub fn load_from_file(path: &str) -> Option<GrannyFile> {
        let mut file = File::open(path).ok()?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).ok()?;

        GrannyFile::load_from_bytes(buffer.as_ref())
    }

    pub fn load_from_bytes(bytes: &[u8]) -> Option<GrannyFile> {
        let (data, header) = parse_header(bytes).ok()?;

        let endianness = if header.big_endian {
            Endianness::Big
        } else {
            Endianness::Little
        };

        let (mut data, file_info) = parse_file_info(endianness)(data).ok()?;

        let mut sectors = Vec::new();

        for _ in 0..file_info.sector_count {
            let (next_input, sector) = parse_sector_info(endianness)(data).ok()?;

            sectors.push(load_sector(bytes, endianness, sector));

            data = next_input;
        }

        let (_, root) = parse_element(
            endianness,
            header.bits_64,
            &sectors,
            file_info.root_ref.sector,
            file_info.type_ref.sector,
            file_info.root_ref.position,
            file_info.type_ref.position
        ).ok()?;

        Some(GrannyFile {
            root_elements: root
        })
    }

    pub fn find_element(&self, path: &str) -> Option<&Element> {
        self.root_elements.resolve(path)
    }
}
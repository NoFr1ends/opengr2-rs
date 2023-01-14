use nom::bytes::complete::take;
use nom::IResult;
use nom::number::complete::{i32, u32};
use nom::number::Endianness;
use nom::sequence::tuple;
use super::{Reference, parse_reference};

#[derive(Debug, PartialEq)]
pub struct FileInfo {
    pub format_version: i32,
    pub total_size: u32,
    pub crc32: u32,
    pub file_info_size: u32,
    pub sector_count: u32,
    pub type_ref: Reference,
    pub root_ref: Reference,
    pub tag: u32,
}

pub fn parse_file_info(endianness: Endianness) -> impl FnMut(&[u8]) -> IResult<&[u8], FileInfo> {
    move |input| {
        let format = i32(endianness);
        let total_size = u32(endianness);
        let crc32 = u32(endianness);
        let file_info_size = u32(endianness);
        let sector_count = u32(endianness);
        let type_ref = parse_reference(endianness);
        let root_ref = parse_reference(endianness);
        let tag = u32(endianness);

        let (input, (format_version, total_size, crc32, file_info_size, sector_count, type_ref, root_ref, tag)) = tuple(
            (format, total_size, crc32, file_info_size, sector_count, type_ref, root_ref, tag)
        )(input)?;

        let (input, _) = take((file_info_size - 40) as usize)(input)?;

        Ok((input, FileInfo {
            format_version,
            total_size,
            crc32,
            file_info_size,
            sector_count,
            type_ref,
            root_ref,
            tag
        }))
    }
}

#[cfg(test)]
mod tests {
    mod le_format_7_32bits {
        use nom::number::Endianness;
        use crate::parser::{FileInfo, parse_file_info, Reference};

        #[test]
        fn test_parse_file_info() {
            let bytes = include_bytes!("../../assets/test1.gr2");

            let res = parse_file_info(Endianness::Little)(&bytes[32..]);

            if let Ok((_, info)) = res {
                assert_eq!(info, FileInfo {
                    format_version: 7,
                    total_size: 10900,
                    crc32: 1737925998,
                    file_info_size: 72,
                    sector_count: 8,
                    type_ref: Reference { sector: 6, position: 0 },
                    root_ref: Reference { sector: 0, position: 0 },
                    tag: 2147483648
                })
            } else {
                assert!(false)
            }
        }
    }

    mod le_format_6_32bits {
        use nom::number::Endianness;
        use crate::parser::{FileInfo, parse_file_info, Reference};

        #[test]
        fn test_parse_file_info() {
            let bytes = include_bytes!("../../assets/prova.gr2");

            let res = parse_file_info(Endianness::Little)(&bytes[32..]);

            if let Ok((_, info)) = res {
                assert_eq!(info, FileInfo {
                    format_version: 6,
                    total_size: 18528,
                    crc32: 133629031,
                    file_info_size: 56,
                    sector_count: 8,
                    type_ref: Reference { sector: 6, position: 960 },
                    root_ref: Reference { sector: 0, position: 0 },
                    tag: 2147483669
                })
            } else {
                assert!(false)
            }
        }
    }
}
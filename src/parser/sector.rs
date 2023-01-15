use nom::IResult;
use nom::number::complete::u32;
use nom::number::Endianness;
use nom::sequence::tuple;

#[derive(Debug, PartialEq)]
pub struct SectorInfo {
    pub compression_type: u32,
    pub data_offset: u32,
    pub compressed_length: u32,
    pub decompressed_length: u32,
    pub alignment: u32,
    pub oodle_stop_0: u32,
    pub oodle_stop_1: u32,
    pub fixup_offset: u32,
    pub fixup_size: u32,
    pub marshall_offset: u32,
    pub marshall_size: u32
}

pub fn parse_sector_info(endianness: Endianness) -> impl FnMut(&[u8]) -> IResult<&[u8], SectorInfo> {
    move |input| {
        let (input, (
            compression_type,
            data_offset,
            compressed_length,
            decompressed_length,
            alignment,
            oodle_stop_0,
            oodle_stop_1,
            fixup_offset,
            fixup_size,
            marshall_offset,
            marshall_size
        )) = tuple((
            u32(endianness),
            u32(endianness),
            u32(endianness),
            u32(endianness),
            u32(endianness),
            u32(endianness),
            u32(endianness),
            u32(endianness),
            u32(endianness),
            u32(endianness),
            u32(endianness)
        ))(input)?;

        Ok((input, SectorInfo {
            compression_type,
            data_offset,
            compressed_length,
            decompressed_length,
            alignment,
            oodle_stop_0,
            oodle_stop_1,
            fixup_offset,
            fixup_size,
            marshall_offset,
            marshall_size,
        }))
    }
}

#[cfg(test)]
mod tests {
    use nom::number::Endianness;
    use crate::parser::{parse_sector_info, SectorInfo};

    #[test]
    pub fn test_parse_sector_le_7_32bits() {
        let bytes = include_bytes!("../../assets/test1.gr2");

        let res = parse_sector_info(Endianness::Little)(&bytes[104..148]);
        if let Ok((input, sector)) = res {
            assert_eq!(sector, SectorInfo {
                compression_type: 0,
                data_offset: 456,
                compressed_length: 1824,
                decompressed_length: 1824,
                alignment: 4,
                oodle_stop_0: 200,
                oodle_stop_1: 200,
                fixup_offset: 8296,
                fixup_size: 9,
                marshall_offset: 8404,
                marshall_size: 0
            });
            assert_eq!(input.len(), 0);
        } else {
            assert!(false);
        }
    }
}
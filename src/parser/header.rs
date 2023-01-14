use nom::branch::alt;
use nom::bytes::complete::{tag, take};
use nom::IResult;
use nom::number::complete::u32;
use nom::number::Endianness;
use nom::sequence::tuple;

#[derive(Debug, PartialEq)]
pub struct Header {
    pub big_endian: bool,
    pub extra_16: bool,
    pub bits_64: bool,

    pub size: u32,
    pub format: u32,
}

pub fn parse_header(input: &[u8]) -> IResult<&[u8], Header> {
    let (input, magic) = alt((
        tag([0xB8, 0x67, 0xB0, 0xCA, 0xF8, 0x6D, 0xB1, 0x0F, 0x84, 0x72, 0x8C, 0x7E, 0x5E, 0x19, 0x00, 0x1E]), /* Little Endian 32-bit File Format 6 */
        tag([0xCA, 0xB0, 0x67, 0xB6, 0x0F, 0xB1, 0xDB, 0xF8, 0x7E, 0x8C, 0x72, 0x84, 0x1E, 0x00, 0x19, 0x5E]), /* Big Endian 32-bit File Format 6 */

        tag([0x29, 0xDE, 0x6C, 0xC0, 0xBA, 0xA4, 0x53, 0x2B, 0x25, 0xF5, 0xB7, 0xA5, 0xF6, 0x66, 0xE2, 0xEE]), /* Little Endian 32-bit File Format 7 (Granny 2.9) */
        tag([0xE5, 0x9B, 0x49, 0x5E, 0x6F, 0x63, 0x1F, 0x14, 0x1E, 0x13, 0xEB, 0xA9, 0x90, 0xBE, 0xED, 0xC4]), /* Little Endian 64-bit File Format 7 (Granny 2.9) */
    ))(input)?;

    let big_endian = magic[0] == 0xCA;
    let extra_16 = magic[0] == 0x29 || magic[0] == 0xE5;
    let bits_64 = magic[0] == 0xE5;

    let endianness = if big_endian {
        Endianness::Big
    } else {
        Endianness::Little
    };

    let size_with_sectors = u32(endianness);
    let format = u32(endianness);
    let extra = take(8usize);

    let (input, (size, format, _)) = tuple((size_with_sectors, format, extra))(input)?;

    Ok((input, Header {
        big_endian,
        extra_16,
        bits_64,
        size,
        format,
    }))
}

#[cfg(test)]
mod tests {
    mod le_format_7_32bits {
        use crate::parser::{Header, parse_header};

        #[test]
        fn test_parse_header() {
            let bytes = include_bytes!("../../assets/test1.gr2");

            let res = parse_header(bytes);

            if let Ok((input, header)) = res {
                assert_eq!(header, Header { big_endian: false, extra_16: true, bits_64: false, size: 456, format: 0 });
                assert_eq!(input[0], 0x07);
                assert_eq!(input.len(), bytes.len() - 32);
            } else {
                assert!(false)
            }
        }
    }

    mod le_format_6_32bits {
        use crate::parser::{Header, parse_header};

        #[test]
        fn test_parse_header() {
            let bytes = include_bytes!("../../assets/prova.gr2");

            let res = parse_header(bytes);

            if let Ok((input, header)) = res {
                assert_eq!(header, Header { big_endian: false, extra_16: false, bits_64: false, size: 440, format: 0 });
                assert_eq!(input[0], 0x06);
                assert_eq!(input.len(), bytes.len() - 32);
            } else {
                assert!(false)
            }
        }
    }
}
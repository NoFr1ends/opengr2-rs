use nom::Err;
use nom::number::Endianness;
use opengr2::parser::{parse_file_info, parse_header, parse_sector};

#[test]
fn test_parser_integration_le_7_32bits() {
    let bytes = include_bytes!("../assets/test1.gr2");

    let (input, header) = parse_header(bytes).unwrap();
    println!("{:?}", header);
    let endianness = if header.big_endian {
        Endianness::Big
    } else {
        Endianness::Little
    };

    let (mut input, file_info) = parse_file_info(endianness)(input).unwrap();
    println!("{:?}", file_info);
    assert_eq!(file_info.sector_count, 8);

    for i in 0..file_info.sector_count {
        let (next_input, sector) = parse_sector(endianness)(input).unwrap();

        println!("Sector {}:", i);
        println!("{:?}", sector);

        input = next_input;
    }
}
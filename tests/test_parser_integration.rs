use nom::number::Endianness;
use opengr2::parser::{parse_element, parse_file_info, parse_header, parse_sector_info};
use opengr2::sector::load_sector;

fn parse_data(bytes: &[u8]) {
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

    let mut sectors = Vec::new();

    for i in 0..file_info.sector_count {
        let (next_input, sector) = parse_sector_info(endianness)(input).unwrap();
        println!("Sector {}:", i);
        println!("{:?}", sector);

        sectors.push(load_sector(bytes, endianness, sector));

        input = next_input;
    }

    let (_, root) = parse_element(
        endianness,
        header.bits_64,
        &sectors,
        file_info.root_ref.sector,
        file_info.type_ref.sector,
        file_info.root_ref.position,
        file_info.type_ref.position
    ).unwrap();
}

#[test]
fn test_parser_integration_le_7_32bits() {
    let bytes = include_bytes!("../assets/suzanne_le.gr2");

    parse_data(bytes);
}

#[test]
fn test_parser_integration_le_7_64bits() {
    let bytes = include_bytes!("../assets/suzanne_le64.gr2");

    parse_data(bytes);
}

#[test]
fn test_parser_integration_be_7_32bits() {
    let bytes = include_bytes!("../assets/suzanne_be.gr2");

    parse_data(bytes);
}

#[test]
fn test_parser_integration_be_7_64bits() {
    let bytes = include_bytes!("../assets/suzanne_be64.gr2");

    parse_data(bytes);
}
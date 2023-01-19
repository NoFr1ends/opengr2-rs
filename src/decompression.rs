use crate::parser::SectorInfo;

#[repr(u32)]
enum CompressionType {
    None,
    #[allow(dead_code)]
    Oodle0,
    #[allow(dead_code)]
    Oodle1,
    #[allow(dead_code)]
    Bitknit1,
    #[allow(dead_code)]
    Bitknit2
}

pub fn decompress_sector(input: &[u8], sector: &SectorInfo) -> Vec<u8> {
    let sector_data = &input[sector.data_offset as usize..(sector.data_offset + sector.compressed_length) as usize];

    if sector.compression_type == CompressionType::None as u32 {
        return sector_data.to_vec();
    }

    panic!("Unsupported compression type {}", sector.compression_type);
}
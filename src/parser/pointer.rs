use nom::IResult;
use nom::number::complete::u32;
use nom::number::Endianness;
use nom::sequence::tuple;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Pointer {
    pub src_offset: u32,
    pub dst_sector: u32,
    pub dst_offset: u32,
}

pub fn parse_pointer(endianness: Endianness) -> impl FnMut(&[u8]) -> IResult<&[u8], Pointer> {
    move |input| {
        let src_offset = u32(endianness);
        let dst_sector = u32(endianness);
        let dst_offset = u32(endianness);

        let (input, (src_offset, dst_sector, dst_offset)) = tuple((src_offset, dst_sector, dst_offset))(input)?;

        Ok((input, Pointer {
            src_offset,
            dst_sector,
            dst_offset,
        }))
    }
}
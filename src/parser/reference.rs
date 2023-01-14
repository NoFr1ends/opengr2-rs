use nom::IResult;
use nom::number::complete::u32;
use nom::number::Endianness;
use nom::sequence::tuple;

#[derive(Debug, PartialEq)]
pub struct Reference {
    pub sector: u32,
    pub position: u32,
}

pub fn parse_reference(endianness: Endianness) -> impl FnMut(&[u8]) -> IResult<&[u8], Reference> {
    move |input| {
        let sector = u32(endianness);
        let position = u32(endianness);

        let (input, (sector, position)) = tuple((sector, position))(input)?;

        Ok((input, Reference {
            sector,
            position,
        }))
    }
}
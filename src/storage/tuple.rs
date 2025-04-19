// TODO: create some sort of "schema" type to interpret a raw tuple
pub struct Tuple<'a> {
    data: &'a [u8],
}

impl<'a> Tuple<'a> {
    pub fn from(data: &'a [u8]) -> Self {
        Tuple { data }
    }
}

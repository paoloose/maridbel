// TODO
// - Create some sort of "schema" type to interpret raw tuples

pub struct Tuple<'a> {
    data: &'a [u8],
}

impl<'a> Tuple<'a> {
    pub fn from(data: &'a [u8]) -> Self {
        Tuple { data }
    }
}

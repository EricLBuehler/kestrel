#[derive(Clone, Debug)]
pub struct FileInfo<'a> {
    pub data: &'a [u8],
    pub name: String,
    pub dir: String,
}

#[derive(Clone, Debug)]
pub struct Position {
    pub line: usize,
    pub startcol: usize, //Inclusive
    pub endcol: usize,   //Exclusive
}

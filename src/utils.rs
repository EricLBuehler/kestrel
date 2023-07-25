use std::str::Chars;

#[derive(Clone, Debug)]
pub struct FileInfo<'a> {
    pub data: Chars<'a>,
    pub name: String,
    pub dir: String,
}

#[derive(Clone, Debug)]
pub struct Position {
    pub line: usize,
    pub startcol: usize, //Inclusive
    pub endcol: usize,   //Exclusive
    pub startcol_raw: usize, //Inclusive
    pub endcol_raw: usize,   //Exclusive
}


#[derive(Debug)]
pub enum Number {
    Float64(f64),
    Usize(usize),
}

#[derive(Debug)]
pub enum Data {
    Number(Number),
    Blarg(Box<str>),

}
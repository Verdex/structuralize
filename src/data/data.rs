
#[derive(Debug, PartialEq, Clone)]
pub enum Number {
    Float64(f64),
    Usize(usize),
    // TODO:  Indeterminate that's just a string or something?
}

#[derive(Debug, PartialEq, Clone)]
pub enum Data {
    Number(Number),
    String(String),
    Symbol(String),
    Cons { name: String, params: Vec<Data> },
    Struct { name: String, fields: Vec<(String, Data)> },
    List(Vec<Data>),
}

// TODO: macro that lets you use struct dot notation to access fields from struct item
// TODO: derives needed so that Data can be used in hash?
// TODO: Hash case for Data?

impl From<usize> for Data {
    fn from(item : usize) -> Self {
        Data::Number(Number::Usize(item))
    }
}

impl From<f64> for Data {
    fn from(item : f64) -> Self {
        Data::Number(Number::Float64(item))
    }
}

impl<T> From<Vec<T>> for Data where Data : From<T> {
    fn from(item : Vec<T>) -> Self {
        Data::List(item.into_iter().map(|x| x.into()).collect::<Vec<Data>>())
    }
}
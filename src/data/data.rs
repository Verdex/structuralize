
#[derive(Debug, PartialEq, Clone)]
pub enum Number {
    Float64(f64),
    Usize(usize),
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
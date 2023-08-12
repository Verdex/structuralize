
#[derive(Debug, PartialEq, Clone)]
pub enum Number {
    Float64(f64),
    Usize(usize),
    // TODO:  Indeterminate that's just a string or something?
}

// TODO: Add Data Trait
// Guess is that either there needs to be a lot of is_* methods or a to_data method

#[derive(Debug, PartialEq)]
pub enum Data {
    Number(Number),
    String(Box<str>), 
    Symbol(Box<str>),
    Cons { name: Box<str>, params: Vec<Data> },
    Struct { name: Box<str>, fields: Vec<(Box<str>, Data)> },
    List(Vec<Data>),
}

// TODO Denest operator 


impl From<Box<str>> for Data {
    fn from(item : Box<str>) -> Self {
        Data::String(item.into())
    }
}

impl From<&str> for Data {
    fn from(item : &str) -> Self {
        Data::String(item.into())
    }
}

impl From<String> for Data {
    fn from(item : String) -> Self {
        Data::String(item.into())
    }
}

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
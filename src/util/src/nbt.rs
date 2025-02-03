use std::collections::HashMap;

use nbt::Value;

pub trait NbtExt {
    fn as_byte(&self) -> &i8;

    fn as_short(&self) -> &i16;

    fn as_int(&self) -> &i32;

    fn as_long(&self) -> &i64;

    fn as_float(&self) -> &f32;

    fn as_double(&self) -> &f64;

    fn as_byte_array(&self) -> &Vec<i8>;

    fn as_string(&self) -> &String;

    fn as_list(&self) -> &Vec<Value>;

    fn as_compound(&self) -> &HashMap<String, Value>;

    fn as_int_array(&self) -> &Vec<i32>;

    fn as_long_array(&self) -> &Vec<i64>;
}

impl NbtExt for Value {
    fn as_byte(&self) -> &i8 {
        match self {
            Value::Byte(x) => x,
            _ => panic!("Not a Byte")
        }
    }

    fn as_short(&self) -> &i16 {
        match self {
            Value::Short(x) => x,
            _ => panic!("Not a Short")
        }
    }

    fn as_int(&self) -> &i32 {
        match self {
            Value::Int(x) => x,
            _ => panic!("Not a Int")
        }
    }

    fn as_long(&self) -> &i64 {
        match self {
            Value::Long(x) => x,
            _ => panic!("Not a Long")
        }
    }

    fn as_float(&self) -> &f32 {
        match self {
            Value::Float(x) => x,
            _ => panic!("Not a Float")
        }
    }

    fn as_double(&self) -> &f64 {
        match self {
            Value::Double(x) => x,
            _ => panic!("Not a Double")
        }
    }

    fn as_byte_array(&self) -> &Vec<i8> {
        match self {
            Value::ByteArray(x) => x,
            _ => panic!("Not a ByteArray")
        }
    }

    fn as_string(&self) -> &String {
        match self {
            Value::String(x) => x,
            _ => panic!("Not a String")
        }
    }

    fn as_list(&self) -> &Vec<Value> {
        match self {
            Value::List(x) => x,
            _ => panic!("Not a List")
        }
    }

    fn as_compound(&self) -> &HashMap<String, Value> {
        match self {
            Value::Compound(x) => x,
            _ => panic!("Not a Compound")
        }
    }

    fn as_int_array(&self) -> &Vec<i32> {
        match self {
            Value::IntArray(x) => x,
            _ => panic!("Not an IntArray")
        }
    }

    fn as_long_array(&self) -> &Vec<i64> {
        match self {
            Value::LongArray(x) => x,
            _ => panic!("Not a LongArray")
        }
    }
}

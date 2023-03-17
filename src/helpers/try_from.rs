use super::wrapper::W;
use surrealdb::sql::{Array, Object, Value};

use crate::error::AppError;

impl TryFrom<W<Value>> for Object {
    type Error = AppError;

    fn try_from(value: W<Value>) -> Result<Object, AppError> {
        match value.0 {
            Value::Object(obj) => Ok(obj),
            _ => Err(AppError::TypeError("Object")),
        }
    }
}

impl TryFrom<W<Value>> for Array {
    type Error = AppError;

    fn try_from(value: W<Value>) -> Result<Array, AppError> {
        match value.0 {
            Value::Array(obj) => Ok(obj),
            _ => Err(AppError::TypeError("Array")),
        }
    }
}

impl TryFrom<W<Value>> for i64 {
    type Error = AppError;

    fn try_from(value: W<Value>) -> Result<i64, AppError> {
        match value.0 {
            Value::Number(obj) => Ok(obj.as_int()),
            _ => Err(AppError::TypeError("i64")),
        }
    }
}

impl TryFrom<W<Value>> for bool {
    type Error = AppError;

    fn try_from(value: W<Value>) -> Result<bool, AppError> {
        match value.0 {
            Value::True => Ok(true),
            Value::False => Ok(false),
            _ => Err(AppError::TypeError("bool")),
        }
    }
}

impl TryFrom<W<Value>> for String {
    type Error = AppError;

    fn try_from(value: W<Value>) -> Result<String, AppError> {
        match value.0 {
            Value::Strand(strand) => Ok(strand.as_string()),
            Value::Thing(thing) => Ok(thing.to_string()),
            _ => Err(AppError::TypeError("String")),
        }
    }
}

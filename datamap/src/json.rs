use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

pub fn parse_json_value<T: DeserializeOwned>(value: Value) -> T {
    serde_json::from_value(value).unwrap()
}

pub fn to_json_value<T: Serialize>(value: T) -> Value {
    serde_json::to_value(value).unwrap()
}

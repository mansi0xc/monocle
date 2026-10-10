//! Strict bounded first pass. A plain serde_json::Value would discard duplicate keys.
use crate::{Code, Diagnostic};
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Number, Value};
use std::fmt;

struct Seed<'a> {
    depth: usize,
    nodes: &'a mut usize,
}
impl<'de> DeserializeSeed<'de> for Seed<'_> {
    type Value = Value;
    fn deserialize<D: de::Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
        if self.depth > 32 || *self.nodes >= 100_000 {
            return Err(de::Error::custom(
                "MONOCLE_STRUCTURE: depth/node limit exceeded",
            ));
        }
        *self.nodes += 1;
        deserializer.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for Seed<'_> {
    type Value = Value;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("bounded JSON")
    }
    fn visit_bool<E: de::Error>(self, v: bool) -> Result<Value, E> {
        Ok(Value::Bool(v))
    }
    fn visit_unit<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_i64<E: de::Error>(self, v: i64) -> Result<Value, E> {
        Ok(v.into())
    }
    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Value, E> {
        Ok(v.into())
    }
    fn visit_f64<E: de::Error>(self, v: f64) -> Result<Value, E> {
        Number::from_f64(v)
            .map(Value::Number)
            .ok_or_else(|| E::custom("non-finite number"))
    }
    fn visit_str<E: de::Error>(self, v: &str) -> Result<Value, E> {
        self.visit_string(v.to_owned())
    }
    fn visit_string<E: de::Error>(self, v: String) -> Result<Value, E> {
        if v.len() > crate::MAX_BUNDLE_BYTES {
            return Err(E::custom("MONOCLE_STRUCTURE: string too large"));
        }
        Ok(Value::String(v))
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
        let mut values = Vec::new();
        while let Some(value) = seq.next_element_seed(Seed {
            depth: self.depth + 1,
            nodes: self.nodes,
        })? {
            if values.len() == 4096 {
                return Err(de::Error::custom("MONOCLE_STRUCTURE: array too large"));
            }
            values.push(value);
        }
        Ok(Value::Array(values))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        let mut values = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if key.len() > 128 || values.len() == 64 {
                return Err(de::Error::custom("MONOCLE_STRUCTURE: object/key too large"));
            }
            if values.contains_key(&key) {
                return Err(de::Error::custom(
                    "MONOCLE_DUPLICATE: repeated JSON object key",
                ));
            }
            let value = map.next_value_seed(Seed {
                depth: self.depth + 1,
                nodes: self.nodes,
            })?;
            values.insert(key, value);
        }
        Ok(Value::Object(values))
    }
}

pub(crate) fn strict_value(bytes: &[u8], max: usize) -> Result<Value, Diagnostic> {
    if bytes.len() > max {
        return Err(Diagnostic::new(
            Code::InputTooLarge,
            "$",
            format!("Input exceeds {max} bytes"),
        ));
    }
    let mut de = serde_json::Deserializer::from_slice(bytes);
    let value = Seed {
        depth: 0,
        nodes: &mut 0,
    }
    .deserialize(&mut de)
    .map_err(|e| Diagnostic::from_serde(&e.to_string(), "$", Code::MalformedJson))?;
    de.end()
        .map_err(|e| Diagnostic::from_serde(&e.to_string(), "$", Code::MalformedJson))?;
    Ok(value)
}

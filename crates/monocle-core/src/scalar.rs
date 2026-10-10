use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};
use sha2::{Digest, Sha256};

/// Exact JSON decimal-string transport, never a floating-point number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecimalU64(u64);
impl DecimalU64 {
    pub fn new(value: u64) -> Self {
        Self(value)
    }
    pub fn get(self) -> u64 {
        self.0
    }
}
impl Serialize for DecimalU64 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_string())
    }
}
impl<'de> Deserialize<'de> for DecimalU64 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)
            .map_err(|_| D::Error::custom("MONOCLE_INTEGER: expected a decimal string"))?;
        if value.is_empty()
            || value.len() > 20
            || !value.bytes().all(|b| b.is_ascii_digit())
            || (value.len() > 1 && value.starts_with('0'))
        {
            return Err(D::Error::custom(
                "MONOCLE_INTEGER: expected canonical u64 decimal string",
            ));
        }
        value
            .parse()
            .map(Self)
            .map_err(|_| D::Error::custom("MONOCLE_INTEGER: u64 overflow"))
    }
}

/// Lowercase, even-length hex on the JSON boundary; bounded decoded bytes internally.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Bytes<const MIN: usize, const MAX: usize>(Vec<u8>);
impl<const MIN: usize, const MAX: usize> Bytes<MIN, MAX> {
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}
impl<const MIN: usize, const MAX: usize> Serialize for Bytes<MIN, MAX> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&hex::encode(&self.0))
    }
}
impl<'de, const MIN: usize, const MAX: usize> Deserialize<'de> for Bytes<MIN, MAX> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)
            .map_err(|_| D::Error::custom("MONOCLE_HEX: expected hex string"))?;
        if value.len() < MIN * 2
            || value.len() > MAX * 2
            || value.len() % 2 != 0
            || !value
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(D::Error::custom(
                "MONOCLE_HEX: invalid length or noncanonical lowercase hex",
            ));
        }
        hex::decode(value).map(Self).map_err(D::Error::custom)
    }
}

/// SHA-256 of exact source bytes. An identity, not producer authentication.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ContentDigest([u8; 32]);
impl ContentDigest {
    pub fn of(bytes: &[u8]) -> Self {
        Self(Sha256::digest(bytes).into())
    }
}
impl Serialize for ContentDigest {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format!("sha256:{}", hex::encode(self.0)))
    }
}
impl<'de> Deserialize<'de> for ContentDigest {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)
            .map_err(|_| D::Error::custom("MONOCLE_HEX: expected sha256 digest"))?;
        let Some(value) = value.strip_prefix("sha256:") else {
            return Err(D::Error::custom("MONOCLE_HEX: expected sha256 prefix"));
        };
        if value.len() != 64
            || !value
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(D::Error::custom(
                "MONOCLE_HEX: expected 64 lowercase digest digits",
            ));
        }
        let mut bytes = [0; 32];
        hex::decode_to_slice(value, &mut bytes).map_err(D::Error::custom)?;
        Ok(Self(bytes))
    }
}

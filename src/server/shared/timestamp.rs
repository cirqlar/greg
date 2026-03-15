use serde::{de, ser};
use time::{OffsetDateTime, format_description};

pub fn deserialize_timestamp<'de, D>(deserializer: D) -> Result<OffsetDateTime, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: String = de::Deserialize::deserialize(deserializer)?;
    serde_json::from_str(&s).map_err(de::Error::custom)
}

pub fn serialize_timestamp<S>(timestamp: &OffsetDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: ser::Serializer,
{
    let s = timestamp
        .format(&format_description::well_known::Rfc2822)
        .map_err(ser::Error::custom)?;
    // let s = serde_json::to_string(timestamp).map_err(ser::Error::custom)?;
    serializer.serialize_str(&s)
}

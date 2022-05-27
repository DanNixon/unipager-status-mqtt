use anyhow::anyhow;
use json::{self, JsonValue};
use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub(crate) struct PocsagMessage {
    destination: u32,
    text: String,
}

impl TryFrom<&Vec<JsonValue>> for PocsagMessage {
    type Error = anyhow::Error;

    fn try_from(value: &Vec<JsonValue>) -> std::result::Result<Self, Self::Error> {
        lazy_static! {
            static ref RE: Regex =
                Regex::new("Received Message.+addr:\\s+(\\d+).+data:\\s+\"(.*)\".+").unwrap();
        }

        let text = &value[1];
        if text.is_string() {
            match RE.captures(
                text.as_str()
                    .ok_or_else(|| anyhow!("Failed to parse log as string"))?,
            ) {
                Some(c) => Ok(Self {
                    destination: c[1].parse()?,
                    text: c[2].to_string(),
                }),
                None => Err(anyhow!("Failed to parse log line format")),
            }
        } else {
            Err(anyhow!("Failed to parse log text"))
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub(crate) enum StatusMessage {
    Timeslot(u8),
    Transmitting(bool),
    QueueLength(usize),
    NewMessage(PocsagMessage),
}

impl TryFrom<JsonValue> for StatusMessage {
    type Error = anyhow::Error;

    fn try_from(value: JsonValue) -> std::result::Result<Self, Self::Error> {
        if let JsonValue::Array(v) = &value["StatusUpdate"] {
            match v[0].as_str() {
                Some("timeslot") => Ok(Self::Timeslot(
                    v[1].as_u8()
                        .ok_or_else(|| anyhow!("Failed to parse timeslot"))?,
                )),
                Some("queue") => Ok(Self::QueueLength(
                    v[1].as_usize()
                        .ok_or_else(|| anyhow!("Failed to parse timeslot"))?,
                )),
                Some("transmitting") => {
                    Ok(Self::Transmitting(v[1].as_bool().ok_or_else(|| {
                        anyhow!("Failed to parse transmitting status")
                    })?))
                }
                _ => Err(anyhow!("Unknown status update type")),
            }
        } else if let JsonValue::Array(v) = &value["Log"] {
            Ok(Self::NewMessage(PocsagMessage::try_from(v)?))
        } else {
            Err(anyhow!("Failed to match any known API response"))
        }
    }
}

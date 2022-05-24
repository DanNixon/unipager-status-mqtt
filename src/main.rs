mod config;
mod types;

use crate::{config::Config, types::StatusMessage};
use anyhow::Result;
use clap::Parser;
use futures_util::StreamExt;
use json::{self};
use paho_mqtt::{AsyncClient, ConnectOptionsBuilder, CreateOptionsBuilder, Message};
use std::env;
use std::path::PathBuf;

/// A tool to publish UniPager status via MQTT.
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Path to configuration file
    #[clap(long = "config", short, default_value = "./config.toml")]
    config_file: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Cli::parse();
    let config = Config::from_file(&args.config_file)?;

    let mqtt = AsyncClient::new(
        CreateOptionsBuilder::new()
            .server_uri(config.mqtt.broker)
            .client_id(&config.mqtt.client_id)
            .persistence(env::temp_dir())
            .finalize(),
    )?;

    mqtt.connect(
        ConnectOptionsBuilder::new()
            .user_name(&config.mqtt.username)
            .password(&config.mqtt.password)
            .will_message(Message::new(
                config.topics.availability.as_str(),
                "offline",
                0,
            ))
            .finalize(),
    )
    .wait()?;

    log::info!("Connected to MQTT broker");

    let (ws, _) = tokio_tungstenite::connect_async(config.unipager.api).await?;
    let (_, rx) = ws.split();

    log::info!("Connected to Unipager API");

    mqtt.try_publish(Message::new(
        config.topics.availability.as_str(),
        "online",
        0,
    ))?;

    rx.filter_map(|m| async { m.ok() })
        .filter_map(|m| async { m.into_text().ok() })
        .filter_map(|m| async move { json::parse(&m).ok() })
        .filter_map(|m| async move { StatusMessage::try_from(m).ok() })
        .filter_map(|m| {
            let topics = config.topics.clone();
            async move {
                let topic = match &m {
                    StatusMessage::Timeslot(_) => topics.timeslot,
                    StatusMessage::QueueLength(_) => topics.queue_length,
                    StatusMessage::Transmitting(_) => topics.transmitting,
                    StatusMessage::NewMessage(_) => topics.new_message,
                };
                match serde_json::to_string(&m) {
                    Ok(m) => {
                        log::debug!("Message: {}", m);
                        Some(Message::new(topic, m, 0))
                    }
                    Err(_) => None,
                }
            }
        })
        .for_each(|m| async {
            if let Err(e) = mqtt.try_publish(m) {
                log::error!("Failed to publish MQTT message: {}", e);
            }
        })
        .await;

    Ok(())
}

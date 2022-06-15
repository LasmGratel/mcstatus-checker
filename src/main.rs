#[macro_use]
extern crate serde;

#[macro_use]
extern crate rocket;

use std::future::Future;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use async_minecraft_ping::{ConnectionConfig, ServerDescription, ServerPlayer, ServerPlayers, ServerVersion, StatusConnection, StatusResponse};
use rocket::{Build, Rocket};
use rocket::http::{ContentType, Header, Status};
use rocket::serde::json::Json;
use serde::{Serialize, Serializer};
use tokio::fs::read_to_string;
use thiserror::Error;
use tokio::time::Timeout;
use serde_with::{serde_as, SerializeAs};

type StdError = Box<dyn std::error::Error>;

/// Contains information about the server version.
#[derive(Debug, Serialize)]
#[serde(remote = "ServerVersion")]
pub struct ServerVersionDef {
    /// The server's Minecraft version, i.e. "1.15.2".
    pub name: String,

    /// The server's ServerListPing protocol version.
    pub protocol: u32,
}

/// Contains information about a player.
#[derive(Debug, Serialize)]
#[serde_as]
#[serde(remote = "ServerPlayer")]
pub struct ServerPlayerDef {
    /// The player's in-game name.
    pub name: String,

    /// The player's UUID.
    pub id: String,
}

/// Contains information about the currently online
/// players.
#[serde_as]
#[serde(remote = "ServerPlayers")]
#[derive(Debug, Serialize)]
pub struct ServerPlayersDef {
    /// The configured maximum number of players for the
    /// server.
    pub max: u32,

    /// The number of players currently online.
    pub online: u32,

    /// An optional list of player information for
    /// currently online players.
    #[serde_as(as = "Option<Vec<ServerPlayerDef>>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample: Option<Vec<ServerPlayer>>,
}

impl SerializeAs<ServerPlayer> for ServerPlayerDef {
    fn serialize_as<S>(source: &ServerPlayer, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        ServerPlayerDef::serialize(source, serializer)
    }
}

/// Contains the server's MOTD.
#[derive(Debug, Serialize)]
#[serde(untagged)]
#[serde(remote = "ServerDescription")]
pub enum ServerDescriptionDef {
    Plain(String),
    Object { text: String },
}

/// The decoded JSON response from a status query over
/// ServerListPing.
#[derive(Debug, Serialize)]
#[serde(remote = "StatusResponse")]
pub struct StatusResponseDef {
    /// Information about the server's version.
    #[serde(with = "ServerVersionDef")]
    pub version: ServerVersion,

    /// Information about currently online players.
    #[serde(with = "ServerPlayersDef")]
    pub players: ServerPlayers,

    /// Single-field struct containing the server's MOTD.
    #[serde(with = "ServerDescriptionDef")]
    pub description: ServerDescription,

    /// Optional field containing a path to the server's
    /// favicon.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,
}

#[serde_as]
#[derive(Debug, Serialize)]
pub struct Response {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub err: Option<StatusError>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<StatusResponseDef>")]
    pub result: Option<StatusResponse>,
}

impl SerializeAs<StatusResponse> for StatusResponseDef {
    fn serialize_as<S>(source: &StatusResponse, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        StatusResponseDef::serialize(source, serializer)
    }
}

#[derive(Error, Debug, Serialize)]
pub enum StatusError {
    #[error("Protocol error")]
    ProtocolError,

    #[error("Invalid input string")]
    InvalidInput,

    #[error("Timed out")]
    Timeout,
}

#[get("/<address>")]
async fn status(address: &str) -> (Status, &'static str) {
    let mut split = address.split(":");
    let result: Result<StatusResponse, StatusError> = async {
        let host = split.next().ok_or(StatusError::InvalidInput)?;
        let port = split.next().and_then(|x| x.parse::<u16>().ok()).unwrap_or(25565);
        match tokio::time::timeout(Duration::from_secs(2), ping(host, port)).await {
            Ok(x) => {
                match x {
                    Ok(y) => {
                        Ok(y)
                    }
                    Err(_) => {
                        Err(StatusError::ProtocolError)
                    }
                }
            }
            Err(_) => {
                Err(StatusError::Timeout)
            }
        }
    }.await;


    match result {
        Ok(response) => {
            (Status::Ok, "Online")
        }
        Err(e) => {
            (Status::ServiceUnavailable, "Offline")
        }
    }
}

#[get("/<address>/json")]
async fn status_json(address: &str) -> Json<Response> {
    let mut split = address.split(":");
    let result: Result<StatusResponse, StatusError> = async {
        let host = split.next().ok_or(StatusError::InvalidInput)?;
        let port = split.next().and_then(|x| x.parse::<u16>().ok()).unwrap_or(25565);
        match tokio::time::timeout(Duration::from_secs(5), ping(host, port)).await {
            Ok(x) => {
                match x {
                    Ok(y) => {
                        Ok(y)
                    }
                    Err(_) => {
                        Err(StatusError::ProtocolError)
                    }
                }
            }
            Err(_) => {
                Err(StatusError::Timeout)
            }
        }
    }.await;


    Json(match result {
        Ok(response) => {
            Response {
                result: Some(response),
                err: None
            }
        }
        Err(e) => {
            Response {
                result: None,
                err: Some(e)
            }
        }
    })
}

async fn ping(host: &str, port: u16) -> Result<StatusResponse, StdError> {
    let mut connection_config = ConnectionConfig::build(host).with_port(port);
    let status = connection_config.connect().await?.status().await?;
    Ok(status.status)
}

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build().mount("/", routes![status, status_json])
}
/*
#[tokio::main]
async fn main() -> Result<(), StdError> {
    tokio::time::timeout(Duration::from_secs(2), ping("www.baidu.com", 25565)).await?;
    println!("Hello, world!");
    Ok(())
}
*/
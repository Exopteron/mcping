use serde::{Deserialize, Serialize};



#[derive(Serialize, Deserialize, Debug)]
pub struct PingVersion {
    pub name: String,
    pub protocol: u32
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PingPlayer {
    pub name: String,
    pub id: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PingPlayerInfo {
    pub max: u32,
    pub online: u32,
    pub sample: Vec<PingPlayer>
}
#[derive(Serialize, Deserialize, Debug)]
pub struct PingMod {
    pub modid: String,
    pub version: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PingModInfo {
    #[serde(rename = "type")]
    pub ty: String,
    #[serde(rename = "modList")]
    pub mod_list: Vec<PingMod>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PingResponse {
    pub version: PingVersion,
    pub players: Option<PingPlayerInfo>,
    pub description: serde_json::Value,
    pub favicon: String,
    pub enforces_secure_chat: Option<bool>,
    pub previews_chat: Option<bool>,
    #[serde(rename = "modinfo")]
    pub mods: Option<PingModInfo>
}
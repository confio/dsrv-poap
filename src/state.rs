use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Map;

pub const EVENTS: Map<&str, EventData> = Map::new("events");
// (event name, attendee address)
pub const ATTENDEES: Map<(&str, &Addr), BadgeData> = Map::new("attendees");
// (attendee address, event name)
pub const BADGES: Map<(&Addr, &str), BadgeData> = Map::new("badges");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct EventData {
    pub owner: Addr,
    pub name: String,
    pub image: String,
    pub description: String,
    pub start_time: u64,
    pub end_time: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BadgeData {
    was_late: bool,
}

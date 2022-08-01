use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    RegisterEvent {
        name: String,
        image: String,
        description: String,
        start_time: u64,
        end_time: u64,
    },
    MintBadge {
        event: String,
        attendee: String,
        was_late: bool,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetEvent { name: String },
    // FIXME: add pagination
    ListAllEvents {},
    // FIXME: add pagination
    ListAttendees { name: String },
    ListMyBadges { attendee: String },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetEventResponse {
    pub owner: String,
    pub name: String,
    pub image: String,
    pub description: String,
    pub start_time: u64,
    pub end_time: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ListAllEventsResponse {
    pub events: Vec<GetEventResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ListAttendeesResponse {
    pub attendees: Vec<Attendee>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Attendee {
    pub attendee: String,
    pub was_late: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ListMyBadgesResponse {
    pub badges: Vec<Badge>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Badge {
    pub event: String,
    pub was_late: bool,
}

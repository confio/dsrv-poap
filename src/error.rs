use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Event name was already registered")]
    EventAlreadyRegistered,

    #[error("Event name less than 2 characters")]
    NameTooShort,

    #[error("Event name more than 100 characters")]
    NameTooLong,

    #[error("Image URL must be https://, was {0}")]
    InvalidImageURL(String),

    // #[error("Image URL must be https://, was {url}")]
    // InvalidImageURL{url: String},
    #[error("Event start time before end time")]
    StartBeforeEnd,

    #[error("Cannot register an event in the past")]
    EventAlreadyOver,
}

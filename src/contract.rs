#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, Event, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    Attendee, Badge, ExecuteMsg, GetEventResponse, InstantiateMsg, ListAllEventsResponse,
    ListAttendeesResponse, ListMyBadgesResponse, QueryMsg,
};
use crate::state::{BadgeData, EventData, ATTENDEES, BADGES, EVENTS};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:dsrv-poap";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::RegisterEvent {
            name,
            image,
            description,
            start_time,
            end_time,
        } => execute_register_event(
            deps,
            env,
            info,
            name,
            image,
            description,
            start_time,
            end_time,
        ),
        ExecuteMsg::MintBadge {
            event,
            attendee,
            was_late,
        } => execute_mint_badge(deps, env, info, event, attendee, was_late),
    }
}

pub fn execute_register_event(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    name: String,
    image: String,
    description: String,
    start_time: u64,
    end_time: u64,
) -> Result<Response, ContractError> {
    if EVENTS.may_load(deps.storage, &name)?.is_some() {
        return Err(ContractError::EventAlreadyRegistered);
    }
    let event = build_event(
        &env,
        &info,
        name.clone(),
        image,
        description,
        start_time,
        end_time,
    )?;
    EVENTS.save(deps.storage, &name, &event)?;

    Ok(Response::new().add_attribute("register_event", name))
}

// validate
fn build_event(
    env: &Env,
    info: &MessageInfo,
    name: String,
    image: String,
    description: String,
    start_time: u64,
    end_time: u64,
) -> Result<EventData, ContractError> {
    if name.len() < 2 {
        return Err(ContractError::NameTooShort);
    }
    if name.len() > 100 {
        return Err(ContractError::NameTooLong);
    }
    if !image.starts_with("https://") {
        return Err(ContractError::InvalidImageURL(image));
    }
    if start_time >= end_time {
        return Err(ContractError::StartBeforeEnd);
    }
    if end_time < env.block.time.seconds() {
        return Err(ContractError::EventAlreadyOver);
        // return Err(StdError::generic_err("event already over").into());
    }

    let event = EventData {
        owner: info.sender.clone(),
        name,
        image,
        description,
        start_time,
        end_time,
    };
    Ok(event)
}

pub fn execute_mint_badge(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    event: String,
    attendee: String,
    was_late: bool,
) -> Result<Response, ContractError> {
    let data = EVENTS.load(deps.storage, &event)?;
    if info.sender != data.owner {
        return Err(ContractError::Unauthorized {});
    }
    if env.block.time.seconds() < data.start_time {
        return Err(ContractError::EventNotStarted);
    }
    if env.block.time.seconds() > data.end_time {
        return Err(ContractError::EventAlreadyOver);
    }

    let attendee = deps.api.addr_validate(&attendee)?;
    if ATTENDEES
        .may_load(deps.storage, (&event, &attendee))?
        .is_some()
    {
        return Err(ContractError::BadgeAlreadyIssued);
    }

    let badge = BadgeData { was_late };
    ATTENDEES.save(deps.storage, (&event, &attendee), &badge)?;
    BADGES.save(deps.storage, (&attendee, &event), &badge)?;

    let ev = Event::new("mint-badge")
        .add_attribute("event", event)
        .add_attribute("attendee", attendee);
    Ok(Response::new().add_event(ev))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetEvent { name } => to_binary(&query_get_event(deps, name)?),
        QueryMsg::ListAllEvents {} => to_binary(&list_all_events(deps)?),
        QueryMsg::ListAttendees { name } => to_binary(&list_attendees(deps, name)?),
        QueryMsg::ListMyBadges { attendee } => to_binary(&list_my_badges(deps, attendee)?),
    }
}

fn query_get_event(deps: Deps, name: String) -> StdResult<GetEventResponse> {
    let evt = EVENTS.load(deps.storage, &name)?;
    Ok(evt.into())
}

fn list_all_events(deps: Deps) -> StdResult<ListAllEventsResponse> {
    // let events = EVENTS
    //     .range(deps.storage, None, None, Order::Ascending)
    //     .limit(30)
    //     .map(|item| {
    //         let (_name, data) = item?;
    //         Ok(data.into())
    //     })
    //     .collect::<StdResult<Vec<_>>>()?;

    let mut events = vec![];
    for evt in EVENTS.range(deps.storage, None, None, Order::Ascending) {
        let (_name, data) = evt?;
        events.push(data.into());
    }
    Ok(ListAllEventsResponse { events })
}

fn list_attendees(deps: Deps, name: String) -> StdResult<ListAttendeesResponse> {
    // let attendees = ATTENDEES
    //     .prefix(&name)
    //     .range(deps.storage, None, None, Order::Ascending)
    //     .map(|item| {
    //         let (attendee, badge) = item?;
    //         Ok(Attendee {
    //             attendee: attendee.into(),
    //             was_late: badge.was_late,
    //         })
    //     })
    //     .collect::<StdResult<_>>()?;

    let mut attendees = vec![];
    for item in ATTENDEES
        .prefix(&name)
        .range(deps.storage, None, None, Order::Ascending)
    {
        let (attendee, badge) = item?;
        attendees.push(Attendee {
            attendee: attendee.into(),
            was_late: badge.was_late,
        })
    }
    Ok(ListAttendeesResponse { attendees })
}

fn list_my_badges(deps: Deps, attendee: String) -> StdResult<ListMyBadgesResponse> {
    let attendee = deps.api.addr_validate(&attendee)?;
    let badges = BADGES
        .prefix(&attendee)
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            let (event, badge) = item?;
            Ok(Badge {
                event,
                was_late: badge.was_late,
            })
        })
        .collect::<StdResult<_>>()?;
    Ok(ListMyBadgesResponse { badges })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    #[test]
    fn simple_test() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // create an event
        let start_time = mock_env().block.time.seconds() - 200;
        let end_time = mock_env().block.time.seconds() + 3000;
        let info = mock_info("ethan", &[]);
        let err = execute_register_event(
            deps.as_mut(),
            mock_env(),
            info,
            "DSRV Hacker House".to_string(),
            "http://foo.bar".to_string(),
            "Fun times hacking".to_string(),
            start_time,
            end_time,
        )
        .unwrap_err();
        assert_eq!(
            err,
            ContractError::InvalidImageURL("http://foo.bar".to_string())
        );

        let name = "DSRV Hacker House";
        let info = mock_info("ethan", &[]);
        execute_register_event(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            name.to_string(),
            "https://dsrv.kr/logo.png".to_string(),
            "Fun times hacking".to_string(),
            start_time,
            end_time,
        )
        .unwrap();

        let attendee = "moog";
        execute_mint_badge(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            name.to_string(),
            attendee.to_string(),
            false,
        )
        .unwrap();

        let late = "bart";
        execute_mint_badge(
            deps.as_mut(),
            mock_env(),
            info,
            name.to_string(),
            late.to_string(),
            true,
        )
        .unwrap();

        // find all attendees
        let res = list_attendees(deps.as_ref(), name.to_string()).unwrap();
        assert_eq!(res.attendees.len(), 2);
        assert_eq!(
            res.attendees[0],
            Attendee {
                attendee: late.to_string(),
                was_late: true
            }
        );
        assert_eq!(
            res.attendees[1],
            Attendee {
                attendee: attendee.to_string(),
                was_late: false
            }
        );
    }
}

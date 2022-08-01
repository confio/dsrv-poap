#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, Event, MessageInfo, Response, StdError, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetCountResponse, InstantiateMsg, QueryMsg};
use crate::state::{BadgeData, EventData, State, ATTENDEES, BADGES, EVENTS, STATE};

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
    if !image.startswith("https://") {
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
        QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
    }
}

fn query_count(deps: Deps) -> StdResult<GetCountResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(GetCountResponse { count: state.count })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_binary(&res).unwrap();
        assert_eq!(17, value.count);
    }

    #[test]
    fn increment() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Increment {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_binary(&res).unwrap();
        assert_eq!(18, value.count);
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let unauth_info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        // should now be 5
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_binary(&res).unwrap();
        assert_eq!(5, value.count);
    }
}

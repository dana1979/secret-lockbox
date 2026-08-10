use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, CosmosMsg, BankMsg, Coin, to_binary, Addr
};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, LockResponse};
use crate::state::{LockBox, LOCK_BOX};

const ONE_YEAR_IN_SECONDS: u64 = 31_536_000;
const FAKE_USDT_AMOUNT: u128 = 739_000_000; // 739.00 USDT الثابتة للتمويه

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    // تم التبسيط لإلغاء فحص addr_validate المعقد واستخدام المعاينة المباشرة
    let recipient_addr = Addr::unchecked(&msg.recipient);
    let creation_time = _env.block.time.seconds();
    let exact_unlock_time = creation_time + ONE_YEAR_IN_SECONDS;

    let lock_box = LockBox {
        sender: info.sender,
        recipient: recipient_addr,
        actual_amount: msg.actual_amount,
        fake_amount: FAKE_USDT_AMOUNT,
        unlock_time: exact_unlock_time,
        is_unlocked: false,
    };

    LOCK_BOX.save(deps.storage, &lock_box)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("unlock_time", exact_unlock_time.to_string()))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Withdraw {} => try_withdraw(deps, env, info),
    }
}

pub fn try_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let mut lock_box = LOCK_BOX.load(deps.storage)?;

    if info.sender == lock_box.sender || info.sender != lock_box.recipient {
        return Err(ContractError::Unauthorized {});
    }

    if env.block.time.seconds() < lock_box.unlock_time {
        return Err(ContractError::Unauthorized {});
    }

    lock_box.is_unlocked = true;
    LOCK_BOX.save(deps.storage, &lock_box)?;

    let withdrawal_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: lock_box.recipient.to_string(),
        amount: vec![Coin {
            denom: "uscrt".to_string(),
            amount: lock_box.actual_amount.into(),
        }],
    });

    Ok(Response::new()
        .add_message(withdrawal_msg)
        .add_attribute("action", "withdraw_successful"))
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetStatus { recipient_addr } => to_binary(&query_lock_status(deps, env, recipient_addr)?),
    }
}

pub fn query_lock_status(
    deps: Deps,
    env: Env,
    recipient_addr: String,
) -> StdResult<LockResponse> {
    let lock_box = LOCK_BOX.load(deps.storage)?;
    
    if recipient_addr != lock_box.recipient && recipient_addr != lock_box.sender {
        return Err(StdError::generic_err("غير مصرح لك بالاطلاع"));
    }

    let time_has_passed = env.block.time.seconds() >= lock_box.unlock_time;

    if lock_box.is_unlocked || time_has_passed {
        Ok(LockResponse {
            amount: lock_box.actual_amount,
            asset: "SCRT".to_string(),
            status: "Unlocked".to_string(),
        })
    } else {
        Ok(LockResponse {
            amount: lock_box.fake_amount,
            asset: "USDT".to_string(),
            status: "Locked".to_string(),
        })
    }
}

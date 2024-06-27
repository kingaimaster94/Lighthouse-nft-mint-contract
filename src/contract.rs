#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary,
    Addr,
    Binary,
    Deps,
    DepsMut,
    Empty,
    Env,
    MessageInfo,
    Order,
    Reply,
    Response,
    StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use cw_utils::{ maybe_addr, parse_reply_instantiate_data };
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::execute::{
    mint_native,
    register_collection,
    update_collection,
    update_config,
    unfreeze_collection,
    reveal_collection_metadata,
    update_reveal_collection_metadata, update_admin, renounce_collection,
};
use crate::helpers::{ create_group_key, create_min_log_key };
use crate::msg::{ ExecuteMsg, InstantiateMsg, QueryMsg };
use crate::state::{
    Collection,
    Config,
    MintInfo,
    COLLECTIONS,
    CONFIG,
    INSTANTIATE_INFO,
    MINT_INFO,
    MINT_LOG, RENOUNCE_INFO,
};
use crate::structs::{ CollectionResponseMinimal, CollectionsResponse };

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:lighthouse";
const CONTRACT_VERSION: &str = "0.3.8";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        admin: info.sender,
        fee: msg.fee,
        registeration_open: msg.registeration_open,
        next_reply_id: 0,
        denom: msg.denom,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::RegisterCollection {
            cw721_code,
            name,
            symbol,
            supply,
            token_uri,
            royalty_percent,
            royalty_wallet,
            mint_groups,
            iterated_uri,
            start_order,
            frozen,
            hidden_metadata,
            placeholder_token_uri,
        } =>
            register_collection(
                deps,
                env,
                info,
                cw721_code,
                name,
                symbol,
                supply,
                token_uri,
                royalty_percent,
                royalty_wallet,
                mint_groups,
                iterated_uri,
                start_order,
                frozen,
                hidden_metadata,
                placeholder_token_uri,
            ),
        ExecuteMsg::MintNative { collection, group, recipient, merkle_proof, hashed_address } =>
            mint_native(
                deps,
                env,
                info,
                collection,
                group,
                recipient,
                merkle_proof,
                hashed_address
            ),
        ExecuteMsg::UpdateCollection {
            collection,
            name,
            symbol,
            supply,
            token_uri,
            royalty_percent,
            royalty_wallet,
            mint_groups,
            iterated_uri,
            start_order,
        } =>
            update_collection(
                deps,
                env,
                info,
                collection,
                name,
                symbol,
                supply,
                token_uri,
                royalty_percent,
                royalty_wallet,
                mint_groups,
                iterated_uri,
                start_order
            ),
        ExecuteMsg::UpdateConfig { fee, registeration_open } =>
            update_config(deps, env, info, fee, registeration_open),
        ExecuteMsg::UnfreezeCollection { collection } =>
            unfreeze_collection(deps, env, info, collection),
        ExecuteMsg::RevealCollectionMetadata { collection } =>
            reveal_collection_metadata(deps, env, info, collection),
        ExecuteMsg::UpdateRevealCollectionMetadata { collection, placeholder_token_uri } =>
            update_reveal_collection_metadata(deps, env, info, collection, placeholder_token_uri),
        ExecuteMsg::UpdateAdmin { collection, admin } =>
            update_admin(deps, env, info, collection, admin),
        ExecuteMsg::RenounceCollection { collection } =>
            renounce_collection(deps, env, info, collection),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    let instantiate_info = INSTANTIATE_INFO.load(deps.storage, msg.id.clone())?;
    //let config: Config = CONFIG.load(deps.storage)?;

    let reply = parse_reply_instantiate_data(msg.clone()).unwrap();

    let mut collection = instantiate_info.clone();
    collection.cw721_address = Some(Addr::unchecked(reply.contract_address).into());

    if COLLECTIONS.has(deps.storage, collection.cw721_address.clone().unwrap().to_string()) {
        return Err(ContractError::CollectionExists {});
    }

    COLLECTIONS.save(
        deps.storage,
        collection.cw721_address.clone().unwrap().to_string(),
        &collection
    )?;
    INSTANTIATE_INFO.remove(deps.storage, msg.id);

    Ok(
        Response::new()
            .add_attribute("register_collection", "success")
            .add_attribute("collection", collection.cw721_address.unwrap().to_string())
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&query_config(deps)?),
        QueryMsg::GetCollection { collection } => {
            let collection: Collection = COLLECTIONS.load(deps.storage, collection)?;
            to_binary(&collection)
        }
        QueryMsg::BalanceOf { address, collection } => {
            let collection_info: Collection = COLLECTIONS.load(deps.storage, collection.clone())?;
            let mut mints = Vec::new();
            for group in collection_info.mint_groups {
                let key = create_group_key(&address, &collection, &group.name);
                let info = MINT_INFO.load(deps.storage, key.clone()).unwrap_or(MintInfo {
                    mints: Vec::new(),
                });
                mints.extend(info.mints);
            }
            to_binary(&(MintInfo { mints }))
        }
        QueryMsg::GetCollections { start_after, limit, result_type } => {
            let default_limit = 10;
            let max_limit = 30;
            let limit = limit.unwrap_or(default_limit).min(max_limit) as usize;
            let start = maybe_addr(deps.api, start_after)?;

            if result_type.is_some() && result_type.clone().unwrap() == "full" {
                let collections: Vec<_> = COLLECTIONS.range(
                    deps.storage,
                    start.map(Bound::exclusive),
                    None,
                    Order::Ascending
                )
                    .take(limit)
                    .map(|item| {
                        let (_key, value) = item?;
                        return Ok(value);
                    })
                    .collect::<StdResult<_>>()?;

                to_binary(&(CollectionsResponse { collections }))
            } else {
                let collections: Vec<_> = COLLECTIONS.range(
                    deps.storage,
                    start.map(Bound::exclusive),
                    None,
                    Order::Ascending
                )
                    .take(limit)
                    .map(|item| {
                        let (_key, value) = item?;

                        Ok(CollectionResponseMinimal {
                            cw721_address: value.cw721_address,
                        })
                    })
                    .collect::<StdResult<_>>()?;

                to_binary(&(CollectionsResponse { collections }))
            }
        }
        QueryMsg::GetMinterOf { collection, token_id } => {
            let key = create_min_log_key(&collection, &token_id);

            let minter = MINT_LOG.load(deps.storage, key)?;

            to_binary(&minter)
        }
        QueryMsg::IsCollectionRenounced { collection } => {
            let renounce_info = RENOUNCE_INFO.load(deps.storage, collection)?;
            to_binary(&renounce_info)
        }
    }
}

fn query_config(deps: Deps) -> StdResult<Config> {
    let config: Config = CONFIG.load(deps.storage)?;
    Ok(config)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: Empty) -> Result<Response, ContractError> {
    //let config: Config = CONFIG.load(deps.storage)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new())
}

#[cfg(test)]
mod tests {}

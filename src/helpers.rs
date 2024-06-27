use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};

use cosmwasm_std::{to_binary, Addr, CosmosMsg, StdResult, WasmMsg};

use crate::msg::ExecuteMsg;

/// CwTemplateContract is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct CwTemplateContract(pub Addr);

impl CwTemplateContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T) -> StdResult<CosmosMsg> {
        let msg = to_binary(&msg.into())?;
        Ok((WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![],
        })
        .into())
    }
}

pub fn create_group_key(addr: &Addr, collection_addr: &str, group_name: &str) -> String {
    format!("{}_{}_{}", addr, collection_addr, group_name)
}

pub fn create_min_log_key(collection_addr: &str, token_id: &str) -> String {
    format!("{}_{}", collection_addr, token_id)
}

pub fn create_token_uri(token_uri: &str, token_id: &str, iterated_uri: &bool) -> String {
    if !iterated_uri {
        format!("{}/{}", token_uri, token_id)
    } else {
        token_uri.to_string()
    }
}

pub fn validate_merkle_proof(proof: Vec<Vec<u8>>, root: Vec<u8>, leaf: Vec<u8>) -> bool {
    let mut hash = leaf;
    for proof_hash in proof.into_iter() {
        let mut hasher = Keccak256::new();

        // Hash current hash and proof hash together
        // The smaller one goes first
        if hash < proof_hash {
            hasher.update(&hash);
            hasher.update(&proof_hash);
        } else {
            hasher.update(&proof_hash);
            hasher.update(&hash);
        }

        hash = hasher.finalize().to_vec();
    }

    hash == root
}

pub fn hash(address: &str) -> Vec<u8> {
    let mut hasher = Keccak256::new();
    hasher.update(address.as_bytes());
    hasher.finalize().to_vec()
}

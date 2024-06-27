use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Registeration is close")]
    RegisterationClose {},

    #[error("Collection already exists")]
    CollectionExists {},

    #[error("Collection Already Instantiated")]
    CollectionAlreadyInstantiated {},

    #[error("Sold out")]
    SoldOut {},

    #[error("Invalid Mint Group")]
    InvalidMintGroup {},

    #[error("Group Not Open to Mint")]
    GroupNotOpenToMint {},

    #[error("Invalid Funds")]
    InvalidFunds {},

    #[error("Max Tokens Minted")]
    MaxTokensMinted {},

    #[error("Supply cannot be lower than already minted token count")]
    SupplyLowerThanMinted {},

    #[error("Invalid Merkle Root")]
    InvalidMerkleRoot {},

    #[error("Invalid Merkle Proof")]
    InvalidMerkleProof  {},

    #[error("Invalid Sender")]
    InvalidSender {},

    #[error("Invalid Reply ID")]
    InvalidReplyId {},

    #[error("Share percentage sum must equal to 100")]
    InvalidShares {},

    #[error("Collection is renounced and cannot be updated")]
    Renounced {},
}

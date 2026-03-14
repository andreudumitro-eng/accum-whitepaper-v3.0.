use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid block")]
    InvalidBlock,
    #[error("invalid timestamp")]
    InvalidTimestamp,
    #[error("invalid previous hash")]
    InvalidPrevHash,
    #[error("invalid merkle root")]
    InvalidMerkleRoot,
    #[error("invalid proof of work")]
    InvalidProofOfWork,
    #[error("invalid coinbase")]
    InvalidCoinbase,
    #[error("invalid share")]
    InvalidShare,
    #[error("insufficient input")]
    InsufficientInput,
    #[error("fee too low")]
    FeeTooLow,
    #[error("dust output")]
    DustOutput,
    #[error("double spend")]
    DoubleSpend,
    #[error("invalid script")]
    InvalidScript,
    #[error("invalid epoch")]
    InvalidEpoch,
    #[error("p2p error")]
    P2p,
    #[error("storage error")]
    Storage,
}
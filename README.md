ACCUM v3.2 — Fair Proof‑of‑Contribution Blockchain Protocol
Whitepaper & Technical Specification
Author: Andrii Dumitro
Version: 3.2 (node-ready)
Date: March 2026

1. Introduction
ACCUM is a next‑generation blockchain protocol built on a novel consensus mechanism called Fair Proof‑of‑Contribution (F‑PoC). The protocol is designed to address the structural weaknesses of classical Proof‑of‑Work (PoW), while preserving its strongest properties: decentralization, permissionlessness, and objective security.

Traditional PoW systems suffer from:

• ASIC dominance
• mining pool centralization
• unpredictable, lottery‑style rewards
• lack of miner loyalty
• economic unfairness
• high entry barriers

ACCUM solves these issues by redefining how mining rewards are distributed. Instead of rewarding only the miner who finds a block, ACCUM distributes rewards across three independent axes of contribution:

Shares — computational work

Loyalty — long‑term participation

Bond — economic identity and stake

This creates a mining environment where:

• CPU miners remain competitive
• rewards are proportional and predictable
• long‑term contributors earn more
• Sybil attacks become economically unviable
• mining pools lose structural advantage
• the network remains decentralized by design

ACCUM is built for fairness, sustainability, and long‑term stability.

2. Monetary Model
• Base unit: ACM (Accum)
• Minimal unit: Lyator (LYT)
• Exchange rate: 1 ACM = 10,000,000 LYT
• All protocol values (balances, rewards, fees, bond) — uint64 in LYT
• Maximum supply: 150,000,000 ACM
• Block reward: 500,000 LYT (0.05 ACM)
• Epoch reward: 720,000,000 LYT (72 ACM)
• Block time: 60 seconds
• Epoch length: 1440 blocks (86400 seconds)

3. Motivation and Problems of Classical PoW
Classical PoW (Bitcoin‑style) has several structural flaws:

3.1 ASIC dominance
Specialized hardware outcompetes CPUs and GPUs, centralizing mining.

3.2 Pool centralization
Most miners join pools, giving a few entities control over block production.

3.3 Lottery‑style rewards
A miner may work for months and receive nothing.

3.4 No loyalty
Miners can switch chains instantly, destabilizing smaller networks.

3.5 Economies of scale
Large farms gain disproportionate advantage.

ACCUM solves these issues through epoch‑based mining and PoCI.

4. Cryptographic Parameters
• Proof-of-work function: Argon2id
• Argon2id parameters:
• memory: 256 MiB (268,435,456 bytes)
• iterations: 2
• parallelism: 4
• version: 0x13
• type: Argon2id

5. Block Header Structure (120 bytes, little-endian)
Field	Type	Size (bytes)	Description
version	uint32	4	Block version
prev_hash	[32]byte	32	Previous block hash
merkle_root	[32]byte	32	Merkle root of transactions
timestamp	uint64	8	Unix timestamp
difficulty	[32]byte	32	Compact target
nonce	uint64	8	Proof-of-Work nonce
epoch_index	uint32	4	Current epoch number
Total: 120 bytes

6. Valid Block
text
hash = Argon2id(complete header, parameters above)
condition: hash < target_block (target_block derived from difficulty)
7. Valid Share
text
hash = Argon2id(complete header, parameters above)
condition: hash < target_share (target_share < target_block, defined by protocol)
8. Share Packet Format (P2P)
Field	Type	Size (bytes)	Description
miner_id	[20]byte	20	RIPEMD160(SHA256(compressed secp256k1 pubkey))
header	[120]byte	120	Complete block header
nonce	uint64	8	Nonce producing valid share
hash	[32]byte	32	Computed Argon2id hash
Total: 180 bytes

9. Miner Identity
• miner_id = RIPEMD160(SHA256(compressed secp256k1 public key))
• All PoCI accruals are strictly tied to miner_id

10. Bond
• Minimum bond for PoCI inclusion: 10,000,000 LYT (1 ACM)
• Default lock-up period: 20,160 blocks (≈14 days at 60 sec/block)
• Slashing (100% bond burn):
• equivocation (two different signatures for same block/epoch)
• publishing invalid share with >50% share in epoch
• proven 51% attack (to be implemented in future)

11. Loyalty
• Initial value: 0
• Participation in epoch (≥ 1 valid share): loyalty += 1
• Missed epoch: loyalty = loyalty // 2 (integer division)

12. PoCI (Proof-of-Contribution Index)
text
PoCI_i = 0.6 × norm_shares_i + 0.2 × norm_loyalty_i + 0.2 × norm_bond_i
Normalization:

text
norm_X_i = X_i / max_X
where max_X is the maximum value among all miners in the epoch.

Shares normalization (chosen by protocol constant):

sqrt(shares_i) / max_sqrt_shares

or log2(1 + shares_i) / max_log2_shares

13. Reward Distribution
text
reward_i (in LYT) = (PoCI_i / ΣPoCI) × 720,000,000
Payment occurs in the coinbase transaction of the first block of the next epoch (special output to miner_id address).

14. Difficulty Adjustment
• Interval: every 1440 blocks
• Formula: new_target = old_target × (actual_time / 86400)
• Constraint: change no more than ±25% per adjustment

15. Transactions (Minimal Set)
Transaction Structure
Field	Type	Description
version	uint32	Transaction version (1)
inputs	[]TxIn	List of inputs
outputs	[]TxOut	List of outputs
locktime	uint32	Block height or timestamp lock
TxIn
Field	Type	Description
prev_txid	[32]byte	Previous transaction hash
prev_index	uint32	Output index
scriptSig	VarBytes	Unlocking script
sequence	uint32	For relative locktime
TxOut
Field	Type	Description
value	uint64	Amount in LYT
scriptPubKey	VarBytes	Locking script
Supported Scripts
• P2PKH (OP_DUP OP_HASH160 <20> OP_EQUALVERIFY OP_CHECKSIG)
• P2PK
• 1-of-n multisig
• OP_CHECKLOCKTIMEVERIFY
• OP_CHECKSEQUENCEVERIFY

Validation Rules
• ∑ inputs ≥ ∑ outputs + fee
• fee ≥ 50 LYT
• dust limit: 100 LYT per output
• ECDSA secp256k1, sighash ALL
• no double-spend within block

16. Genesis
• Output: 500,000,000 LYT
• scriptPubKey: 76a91462e907b15cbf27d5425399ebf6f0fb50ebb88f18ac

17. P2P Messages (Minimum Required)
Message	Description
version	Handshake, protocol version
verack	Acknowledgment of version
inv	Inventory (block, tx, share)
getdata	Request data by inv
block	Block transmission
tx	Transaction transmission
share	Share packet transmission
ping / pong	Connection keep-alive
epoch_commit	Merkle root of all shares in epoch
18. Epoch‑Based Mining
Mining in ACCUM is structured into epochs of 1440 blocks.

During an epoch:

• miners submit shares
• loyalty is accumulated
• bond weight is applied
• PoCI is calculated at epoch end
• rewards are distributed in the first block of the next epoch

This eliminates the "lottery effect" of classical PoW and ensures predictable, proportional rewards.

19. Lyator (LYT)
Lyator is the minimal unit of ACCUM.

All protocol‑level values are stored in LYT:

• balances
• fees
• block rewards
• epoch rewards
• bond amounts

This ensures precision and avoids floating‑point errors.

20. Security Model
ACCUM provides:

• ASIC resistance (Argon2id, 256 MB memory)
• anti‑pool decentralization (epoch-based rewards)
• Sybil resistance via Bond (minimum 1 ACM)
• anti‑burst mining via Loyalty (decay on miss)
• predictable inflation (0.0175% annually)
• stable long‑term economics (150M supply cap)

21. Implementation Notes
All amounts stored as uint64 LYT.

Constants (for implementation)
text
LYATORS_PER_ACM      = 10,000,000
BLOCK_REWARD_LYT     = 500,000
EPOCH_BLOCKS         = 1,440
EPOCH_REWARD_LYT     = 720,000,000
MINIMUM_FEE_LYT      = 50
DUST_LIMIT_LYT       = 100
MINIMUM_BOND_LYT     = 10,000,000
BOND_LOCKUP_BLOCKS   = 20,160
ARGON2_MEMORY        = 268,435,456  // 256 MiB
ARGON2_ITERATIONS    = 2
ARGON2_PARALLELISM   = 4
22. Conclusion
ACCUM v3.2 defines a complete, fair, CPU‑friendly blockchain protocol with:

• Lyator monetary unit
• Argon2id PoW
• Epoch‑based mining
• PoCI reward model (0.6 shares, 0.2 loyalty, 0.2 bond)
• Bond identity and slashing conditions
• Loyalty participation with decay
• Low inflation (0.0175% annually)

This document describes the conceptual and economic foundations of the protocol alongside the complete technical specifications required for node implementation.

Author: Andrii Dumitro
March 2026


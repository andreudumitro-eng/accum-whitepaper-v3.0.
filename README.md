ACCUM v3.2 — Fair Proof‑of‑Contribution Blockchain Protocol
Whitepaper & Technical Specification
Author: Andrii Dumitro
Version: 3.2 (node-ready)
Date: March 2026

1. Introduction
ACCUM is a next‑generation blockchain protocol built on a novel consensus mechanism called Fair Proof‑of‑Contribution (F‑PoC). The protocol is designed to address the structural weaknesses of classical Proof‑of‑Work (PoW), while preserving its strongest properties: decentralization, permissionlessness, and objective security.

Traditional PoW systems suffer from:

ASIC dominance

mining pool centralization

unpredictable, lottery‑style rewards

lack of miner loyalty

economic unfairness

high entry barriers

ACCUM solves these issues by redefining how mining rewards are distributed. Instead of rewarding only the miner who finds a block, ACCUM distributes rewards across three independent axes of contribution:

Shares — computational work

Loyalty — long‑term participation

Bond — economic identity and stake

This creates a mining environment where:

CPU miners remain competitive

rewards are proportional and predictable

long‑term contributors earn more

Sybil attacks become economically unviable

mining pools lose structural advantage

the network remains decentralized by design

ACCUM is built for fairness, sustainability, and long‑term stability.

2. Monetary Model
Parameter	Value
Base unit	ACM (Accum)
Minimal unit	Lyator (LYT)
Exchange rate	1 ACM = 10,000,000 LYT
All protocol values	stored as uint64 in LYT
Maximum supply	150,000,000 ACM
Block reward	500,000 LYT (0.05 ACM)
Epoch reward	720,000,000 LYT (72 ACM)
Block time	60 seconds
Epoch length	1440 blocks (86,400 seconds)
3. Motivation and Problems of Classical PoW
Classical PoW (Bitcoin‑style) has several structural flaws:

3.1 ASIC Dominance
Specialized hardware outcompetes CPUs and GPUs, centralizing mining power in the hands of a few manufacturers and large-scale farms. This creates a high barrier to entry for ordinary users.

3.2 Pool Centralization
To smooth out the variance of lottery-based rewards, individual miners are forced to join mining pools. This concentrates consensus power in the hands of pool operators, creating systemic risk and potential for cartelization.

3.3 Lottery‑Style Rewards
A miner's contribution is binary: either they find a block and receive the full reward, or they find nothing. A miner contributing significant hash power over a long period may receive zero compensation.

3.4 No Loyalty
Miners are economic mercenaries who can switch between chains instantly in response to market fluctuations, causing network instability and security vulnerabilities, especially for smaller chains.

3.5 Economies of Scale
Large farms gain disproportionate advantage through bulk hardware purchases, discounted electricity, and optimized cooling. The system favors those with the largest capital outlay.

ACCUM solves these issues through epoch‑based mining and the Proof-of-Contribution Index (PoCI).

4. Cryptographic Parameters
Proof-of-work function: Argon2id

Argon2id parameters:

memory: 256 MiB (268,435,456 bytes)

iterations: 2

parallelism: 4

version: 0x13

type: Argon2id

Argon2id is selected for its memory-hard properties, which provide strong ASIC resistance. The 256 MB memory requirement makes it economically unviable to design efficient ASICs, leveling the playing field for commodity CPU hardware.

5. Block Header Structure (120 bytes, little-endian)
Field	Type	Size (bytes)	Description
version	uint32	4	Block version number (currently 1)
prev_hash	[32]byte	32	SHA256 hash of previous block header
merkle_root	[32]byte	32	Root hash of transaction Merkle tree
timestamp	uint64	8	Unix timestamp (seconds since 1970-01-01)
difficulty	[32]byte	32	Compact target representation
nonce	uint64	8	Nonce for Proof-of-Work
epoch_index	uint32	4	Current epoch number (starts at 1)
Total: 120 bytes

All fields are serialized in little-endian byte order.

6. Valid Block
text
hash = Argon2id(complete header, parameters above)
condition: hash < target_block
Where target_block is derived from the difficulty field. The block is considered valid if the hash meets the network difficulty target.

Additional validation rules:

Timestamp must be greater than the median timestamp of the last 11 blocks and less than network-adjusted time + 2 hours

Merkle root must correctly represent all transactions in the block

First transaction must be a valid coinbase transaction

All transactions must follow validation rules

Block height must be consistent with prev_hash

7. Valid Share
text
hash = Argon2id(complete header, parameters above)
condition: hash < target_share
A "share" is a unit of work submitted by a miner to prove computational contribution during an epoch. The target_share is a protocol-defined target that is significantly easier (higher numerical value) than target_block, allowing miners to submit many shares per epoch.

Shares are not blocks — they do not advance the chain. They serve as proof of work for the PoCI calculation.

8. Share Packet Format (P2P)
Field	Type	Size (bytes)	Description
miner_id	[20]byte	20	RIPEMD160(SHA256(compressed secp256k1 pubkey))
header	[120]byte	120	Complete block header used for the work
nonce	uint64	8	Nonce that produced the valid share hash
hash	[32]byte	32	Computed Argon2id hash (for quick validation)
Total: 180 bytes

Shares are broadcast on the P2P network and used by other nodes to verify a miner's contribution during epoch reward calculation.

9. Miner Identity
miner_id = RIPEMD160(SHA256(compressed secp256k1 public key))

All PoCI accruals and reward distributions are strictly tied to miner_id

The same 20-byte format is used for standard P2PKH addresses

This provides a consistent identity format across the protocol while maintaining cryptographic security through the underlying public key.

10. Bond
The Bond is an economic commitment that enhances a miner's PoCI and provides Sybil resistance.

10.1 Parameters
Parameter	Value
Minimum bond for PoCI inclusion	10,000,000 LYT (1 ACM)
Lock-up period	20,160 blocks (≈14 days)
Bond must be locked in a special bond output and cannot be spent during the lock-up period.

10.2 Bond Weight in PoCI
Miners with bond ≥ minimum receive additional weight in the PoCI calculation. Bond amount is normalized against the maximum bond in the epoch.

10.3 Slashing (100% Bond Burn)
To penalize malicious behavior, the entire bond is burned (destroyed) upon proof of:

Equivocation: Signing two different messages (blocks or shares) for the same height or epoch

Invalid Share Flooding: Publishing known invalid shares while controlling >50% of shares in an epoch

Proven 51% Attack: Participation in a successfully proven 51% attack (mechanism to be defined in future upgrade)

Slashing provides strong economic disincentives for attacks on the network.

10.4 Economic Rationale
The bond requirement serves as a sybil deterrence mechanism, not a monetary barrier measured in fiat. It ensures that each participating miner_id represents a real economic commitment within the protocol's native economy.

Because the value of 1 ACM is determined solely by market forces and may fluctuate significantly over time, the bond is defined exclusively in LYT — the protocol's native unit. This guarantees that:

The requirement remains deterministic and consensus-safe

No external price oracles are needed

The economic weight of the bond scales naturally with the network's market value

The barrier to entry for honest miners remains purely protocol-native

10.5 How Bond Scales with Network Value
Network Phase	Market Cap (example)	1 ACM Value (example)	Bond in USD (illustrative)	Effect
Launch	$5M	$0.033	$0.033	Low barrier — encourages participation
Growth	$150M	$1.00	$1.00	Meaningful cost for sybil creation
Maturity	$1.5B	$10.00	$10.00	Significant economic commitment
Note: USD values are shown for illustration only. The protocol only enforces the bond in LYT.

This design ensures that:

Early phase: Low dollar-cost encourages miner adoption

Mature phase: High dollar-cost naturally deters large-scale sybil attacks

Always: The bond is denominated in the protocol's native asset, preserving decentralization and eliminating oracle dependency

11. Loyalty
Loyalty rewards miners for consistent, long-term participation in the network.

Initial value: 0

Participation in an epoch (≥ 1 valid share): loyalty += 1

Missed epoch (no valid shares): loyalty = loyalty // 2 (integer division)

The loyalty factor increases linearly with continuous participation but decays exponentially (through integer halving) when a miner misses an epoch. This prevents hoarding of loyalty after long absences and ensures that loyalty reflects recent participation.

12. PoCI (Proof-of-Contribution Index)
The PoCI is the mathematical expression of the F-PoC principle, calculated for each miner at the end of every epoch.

12.1 Formula
text
PoCI_i = 0.6 × norm_shares_i + 0.2 × norm_loyalty_i + 0.2 × norm_bond_i
The weights are constants defined by the protocol:

0.6 for shares: Computational work is the primary contribution

0.2 for loyalty: Provides network stability and retention

0.2 for bond: Aligns economic incentives and prevents Sybil attacks

12.2 Normalization
text
norm_X_i = X_i / max_X
Where max_X is the maximum value among all miners in the epoch for that component.

For bond, if a miner's bond is below MINIMUM_BOND_LYT, norm_bond_i = 0.

12.3 Shares Normalization Options
To prevent a single miner with slightly more hashrate from dominating the shares component, a sub-linear normalization function is used. The protocol constant defines which function is active:

Option A: Square Root

text
norm_shares_i = sqrt(shares_i) / max_sqrt_shares
Option B: Logarithmic

text
norm_shares_i = log2(1 + shares_i) / max_log2_shares
Square root provides a balanced reduction of advantage for large miners. Logarithmic provides even stronger compression.

12.4 Reward Calculation
text
reward_i (in LYT) = (PoCI_i / ΣPoCI) × EPOCH_REWARD_LYT
Where ΣPoCI is the sum of PoCI values for all participating miners, and EPOCH_REWARD_LYT = 720,000,000.

13. Reward Distribution
13.1 Accumulation
Throughout the epoch, the network tracks shares, loyalty, and bond for each miner_id.

13.2 Finalization
Upon mining the 1440th block of the epoch, the PoCI for all participants is finalized. The protocol calculates:

Normalized values for all components

Individual PoCI scores

Total PoCI sum

Individual rewards

13.3 Payout
The epoch reward is distributed in a special coinbase transaction included in the first block of the next epoch. This transaction contains multiple outputs, each sending the calculated reward_i to the corresponding miner_id address (P2PKH format).

This mechanism ensures that:

Rewards are proportional to contribution

Distribution is transparent and verifiable

No central party controls payout

14. Difficulty Adjustment
Adjustment interval: every 1440 blocks (each epoch)

Target epoch time: 86,400 seconds (24 hours)

14.1 Formula
text
new_target = old_target × (actual_time_span / 86,400)
Where actual_time_span is the actual time taken to mine the last 1440 blocks.

14.2 Bounds
To prevent extreme volatility from time warp attacks or network hiccups, the adjustment is clamped:

Maximum increase per epoch: +25% (target becomes easier)

Maximum decrease per epoch: -25% (target becomes harder)

14.3 Target Representation
Difficulty is stored as a 32-byte compact target in the block header, following the same format as Bitcoin: a 256-bit number where lower values represent higher difficulty.

15. Transactions (Minimal Set)
15.1 Transaction Structure
Field	Type	Description
version	uint32	Transaction version (currently 1)
inputs	Vec<TxIn>	List of transaction inputs
outputs	Vec<TxOut>	List of transaction outputs
locktime	uint32	Block height or Unix timestamp lock
15.2 TxIn Structure
Field	Type	Description
prev_txid	[32]byte	SHA256 hash of previous transaction
prev_index	uint32	Index of the output in previous transaction
scriptSig	VarBytes	Unlocking script (signature + pubkey)
sequence	uint32	For relative locktime (BIP68)
15.3 TxOut Structure
Field	Type	Description
value	uint64	Amount in LYT
scriptPubKey	VarBytes	Locking script (recipient conditions)
15.4 Supported Script Opcodes
Standard P2PKH:

text
OP_DUP OP_HASH160 <20-byte hash> OP_EQUALVERIFY OP_CHECKSIG
Pay-to-Pubkey (P2PK):

text
<public key> OP_CHECKSIG
1-of-n Multisig:

text
OP_1 <pubkey1> ... <pubkeyn> OP_n OP_CHECKMULTISIG
Timelocks:

OP_CHECKLOCKTIMEVERIFY for absolute timelocks

OP_CHECKSEQUENCEVERIFY for relative timelocks

15.5 Validation Rules
Balance rule: ∑ inputs ≥ ∑ outputs + fee (no inflation)

Minimum fee: fee ≥ 50 LYT

Dust limit: Minimum output value of 100 LYT (outputs below this are considered non-standard and may be rejected)

Signature: ECDSA secp256k1 with SIGHASH_ALL

No double-spend: Same input cannot appear twice in the same block

Script execution: scriptSig must only push data (no opcodes) for standard transactions

16. Genesis Block
The genesis block establishes the initial state of the ledger, hard-coded into every node implementation.

16.1 Genesis Output
Value: 500,000,000 LYT (50 ACM)

scriptPubKey: 76a91462e907b15cbf27d5425399ebf6f0fb50ebb88f18ac

This scriptPubKey corresponds to a standard P2PKH output:

text
OP_DUP OP_HASH160 62e907b15cbf27d5425399ebf6f0fb50ebb88f18 OP_EQUALVERIFY OP_CHECKSIG
16.2 Genesis Properties
The private key for this address is considered destroyed

No one can spend these coins

This symbolizes the fair launch of the network with no pre-mine

16.3 Genesis Header
The genesis block header has:

prev_hash = [0; 32] (all zeros)

timestamp set to the Unix epoch of the launch

difficulty set to the initial network difficulty

nonce set to the value that produces a valid genesis hash

epoch_index = 1

17. P2P Messages (Minimum Required)
All nodes MUST implement the following protocol messages for interoperability:

Message	Direction	Description
version	bidirectional	Handshake: protocol version, capabilities, timestamp
verack	bidirectional	Acknowledgment of version message
inv	one-way	Inventory vector: announces known objects (blocks, tx, shares)
getdata	one-way	Requests specific objects based on inv
block	one-way	Transmits a full block
tx	one-way	Transmits a transaction
share	one-way	Transmits a share packet (see Section 8)
ping	bidirectional	Keep-alive and latency check
pong	bidirectional	Response to ping
epoch_commit	broadcast	Merkle root of all shares in just-concluded epoch
17.1 Message Flow
Nodes establish connection with version/verack

Nodes exchange inv to announce new objects

Peers request unknown objects via getdata

Objects are transmitted (block, tx, share)

ping/pong maintain connection health

At epoch boundaries, epoch_commit is broadcast for validation

18. Epoch-Based Mining
Mining in ACCUM is fundamentally different from classical PoW due to the epoch structure.

18.1 Epoch Lifecycle
During an epoch (blocks N to N+1439):

Miners continuously perform Proof-of-Work

When a miner finds a hash below target_block, they broadcast a block and claim the block reward (0.05 ACM)

When a miner finds a hash below target_share, they broadcast a share packet

All valid shares are recorded for the current epoch

Loyalty counters are active (if miner submits at least one share)

Bond weights are applied based on locked amounts

At epoch boundary (after block N+1439):

The network stops accepting shares for the concluded epoch

PoCI is calculated for all miners who submitted shares

Total reward pool = 720,000,000 LYT + all transaction fees from the epoch

First block of next epoch (block N+1440):

Contains special coinbase transaction with multiple outputs

Each output sends calculated reward to corresponding miner_id

New epoch begins

18.2 Benefits of Epoch-Based Mining
Eliminates lottery effect: Rewards are proportional, not winner-take-all

Predictable income: Miners can estimate returns based on their share of network PoCI

Pool disincentive: Low variance makes solo mining viable

Fairness: Small miners receive their fair share, not zero

19. Lyator (LYT)
Lyator is the minimal, indivisible unit of the ACCUM protocol.

19.1 Purpose
All protocol-level values are stored and processed in LYT to:

Eliminate floating-point errors

Ensure deterministic calculations across all implementations

Simplify integer arithmetic

Prevent rounding discrepancies in consensus

19.2 Usage
LYT is used for:

Account balances

Transaction fees

Block rewards

Epoch rewards

Bond amounts

Dust limits

19.3 Conversion
text
1 ACM = 10,000,000 LYT
User interfaces may display values in ACM for readability, but all consensus code operates exclusively on LYT as uint64.

20. Security Model
ACCUM provides multiple layers of security through its design.

20.1 ASIC Resistance
Argon2id with 256 MB memory requirement

Memory-hard function makes ASIC development economically prohibitive

CPU mining remains competitive

Geographic and demographic decentralization preserved

20.2 Anti-Pool Decentralization
Epoch-based rewards eliminate the variance that forces miners into pools

Proportional rewards make solo mining economically viable

No advantage to pooling hash power

Natural disincentive for pool formation

20.3 Sybil Resistance
Minimum bond requirement (1 ACM) for full PoCI participation

Creating many identities requires locking many coins

Economic barrier to Sybil attacks

20.4 Anti-Burst Mining
Loyalty mechanism rewards long-term participation

Loyalty decays on missed epochs

"Hit-and-run" miners receive lower effective rewards

Network stability through committed miners

20.5 51% Attack Resistance
Attacker needs 51% of shares AND proportional loyalty AND bond

Multi-dimensional requirement dramatically increases attack cost

Bond slashing provides additional disincentive

20.6 Predictable Economics
Fixed maximum supply: 150,000,000 ACM

Low inflation: 0.0175% annually

Transparent emission schedule

No unexpected monetary policy changes

21. Implementation Notes
21.1 Data Types
All monetary amounts MUST be stored as 64-bit unsigned integers (uint64_t) representing LYT.

21.2 Constants
text
// Monetary
LYATORS_PER_ACM      = 10,000,000
MAX_SUPPLY_ACM       = 150,000,000
BLOCK_REWARD_LYT     = 500,000
EPOCH_REWARD_LYT     = 720,000,000

// Time
TARGET_BLOCK_TIME    = 60 seconds
EPOCH_BLOCKS         = 1,440
EPOCH_DURATION       = 86,400 seconds

// Fees & Dust
MINIMUM_FEE_LYT      = 50
DUST_LIMIT_LYT       = 100

// Bond
MINIMUM_BOND_LYT     = 10,000,000
BOND_LOCKUP_BLOCKS   = 20,160

// Proof-of-Work
ARGON2_MEMORY        = 268,435,456  // 256 MiB in bytes
ARGON2_ITERATIONS    = 2
ARGON2_PARALLELISM   = 4
ARGON2_VERSION       = 0x13

// PoCI Weights
POCI_WEIGHT_SHARES   = 0.6
POCI_WEIGHT_LOYALTY  = 0.2
POCI_WEIGHT_BOND     = 0.2
21.3 Serialization
All integers are serialized in little-endian byte order

Variable-length data (scripts) use Bitcoin-style VarInt prefix

Hashes are serialized as raw bytes

21.4 Error Handling
Implementations should use checked arithmetic to prevent overflow/underflow. All monetary operations must validate against maximum supply constraints.

22. Conclusion
ACCUM v3.2 defines a complete, fair, CPU-friendly blockchain protocol that addresses the fundamental inequities of classical Proof-of-Work while preserving its core strengths of decentralization and permissionless security.

22.1 Key Innovations
Fair Proof-of-Contribution (F-PoC)
A multi-dimensional reward mechanism that evaluates miners based on:

Computational work (shares) — 60% weight

Network loyalty — 20% weight

Economic commitment (bond) — 20% weight

Epoch-Based Distribution

1440-block epochs (24 hours)

Proportional rewards based on PoCI

Elimination of lottery-style variance

ASIC Resistance

Argon2id with 256 MB memory

CPU-friendly mining

Level playing field for commodity hardware

Economic Alignment

Bond requirements with slashing conditions

Loyalty accumulation and decay

Low inflation (0.0175% annually)

Fixed supply (150,000,000 ACM)

22.2 Technical Completeness
This document provides everything necessary for a complete, interoperable node implementation:

Full monetary model with precise constants

Cryptographic parameters and hash function specifications

Complete data structures (block header, share packet, transactions)

Consensus rules and validation logic

P2P protocol message definitions

Genesis block specification

22.3 Vision
ACCUM establishes a new standard for fairness in decentralized networks. By rewarding not just computational power, but also long-term commitment and economic alignment, the protocol creates a more equitable mining environment where:

Individual CPU miners can compete effectively

Long-term contributors are appropriately rewarded

Attacks become economically prohibitive

The network remains truly decentralized

Value accrues to those who build and maintain the network

This document serves as both a conceptual whitepaper explaining the philosophy behind ACCUM and a complete technical specification for node developers. The protocol is ready for implementation and launch, with all parameters finalized and no unresolved design questions.

23. Governance
23.1 Governance Philosophy
ACCUM is built on the principle of gradual decentralization. The protocol recognizes that in the early stages of network existence, rapid decision-making is critical for security and stability. Therefore, ACCUM's governance model is evolutionary — it develops alongside the network, transferring control to the community as it grows and matures.

23.2 Phase 1: Foundational Governance (Blocks 0 — 200,000)
Duration: ~1 year (at 60 sec/block)

In this phase, the protocol founders retain the right to make strategic decisions. This is necessary for:

Rapid response to critical vulnerabilities

Parameter adjustments in response to real network conditions

Preventing attacks in the early stage when the community is still small

Limitations on Founder Power:

All changes must be published as ACCUM Improvement Proposals (AIPs) at least 14 days before activation. Each change must be accompanied by public justification and technical audit.

23.3 Phase 2: Hybrid Governance (Blocks 200,001 — 500,000)
Duration: ~1.5 years

In this phase, the first elements of decentralized governance are introduced:

Advisory Votes

The community gains the right to non-binding votes on key parameters:

Changes to PoCI weights (0.6/0.2/0.2)

Adjustments to minimum bond

Changes to epoch length

Although the results are not binding, founders commit to publicly explaining any decision that contradicts the community's will.

Validator Council

An advisory body of 7 validators elected by the community is created. The council receives veto power over changes affecting network security, with the ability to delay changes for 30 days.

23.4 Phase 3: Full DAO Decentralization (Block 500,001+)
After reaching block height 500,001 (~2.5 years after launch), governance fully transitions to the community according to the following model:

Governance Actors
Role	Composition	Authority
Validators	Node operators	Technical upgrades, consensus parameters
Token Holders	All ACM owners	Economic parameters, treasury
Security Council	9 elected experts (6/9 multisig)	Emergency actions, freeze during attacks
Voting Mechanism
Vote Weight:

For economic matters: 1 ACM = 1 vote (quadratic weighting to prevent whale dominance)

For technical matters: validator votes weighted by their share of PoCI

For security matters: Security Council only, with mandatory public report

Procedure:

Proposal published as AIP on forum (minimum 7 days discussion)

Author deposits 1000 ACM (returned if successful, burned if malicious)

Voting lasts 7 days

Approval requires >50% votes with >30% participation

7-day timelock before activation

Voteable Parameters
Category	Examples	Change Frequency
PoCI weights	0.6/0.2/0.2	No more than once per year
Bond	Minimum bond	No more than once per 6 months
Epoch length	1440 blocks	No more than once per year
Fee market	Minimum fee	No more than once per 3 months
Treasury	Fund allocation	Anytime
23.5 Economic Security of Governance
To prevent governance attacks:

Preventing Vote Buying:

Quadratic voting: vote cost increases exponentially with number of votes

Minimum token holding period for voting participation: 7 days

Protection Against Malicious Proposals:

Author's proposal deposit burned if attempting destructive changes

Security Council has veto power over proposals threatening network integrity (with mandatory public justification)

23.6 Transparency and Accountability
All votes, proposals, and decisions are recorded on the blockchain. Historical records of governance decisions are available for audit by any network participant. The Security Council is required to publish quarterly reports on its activities.

23.7 Fork Mechanism
In case of disagreement with governance decisions, any participant may create a fork of the protocol. ACCUM does not prevent forks and recognizes the community's right to separate, provided basic consensus rules are followed.

Author: Andrii Dumitro
March 2026


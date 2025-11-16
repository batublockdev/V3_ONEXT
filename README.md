# BettingContract - Soroban Smart Contract
**Decentralized Peer-to-Peer Betting Platform on Stellar**  
`#![no_std]` • Soroban SDK • Stellar Network  

![Stellar](https://img.shields.io/badge/Stellar-Soroban-00D2FF?style=flat-square&logo=stellar)
![Rust](https://img.shields.io/badge/Rust-no_std-orange?style=flat-square&logo=rust)
![License](https://img.shields.io/badge/License-MIT-blue.svg)
![Version](https://img.shields.io/badge/Version-2.0-brightgreen)
![Updated](https://img.shields.io/badge/Updated-November%2016%2C%202025-yellow)

## Overview

**BettingContract** is a fully decentralized, oracle-free, peer-to-peer betting protocol built on **Stellar** using **Soroban** smart contracts.

No centralized oracles. No trusted third parties.  
Pure on-chain game theory with economic incentives for honesty.

## Key Features

| Feature                        | Description                                                                 |
|-------------------------------|-----------------------------------------------------------------------------|
| Private Betting Rooms         | Invite-only groups with fixed bet amounts                                   |
| Dual Collateral System        | USD + TRUST token (10%) or USD + extra 20% USD collateral                   |
| Honesty Points Leaderboard    | Users earn/lose points → Top 2 become Supreme Court judges                 |
| Result Submission & Voting    | Any participant submits result → others vote within 5 hours                |
| Supreme Court Dispute Layer   | Admin + External Trusted + Top 2 Honest Users + Community vote             |
| Automatic Payouts             | Winners share loser pool + fines, protocol takes 5%                         |
| Full Refund Mechanisms        | Inactive rooms or abandoned games → refund + honesty bonus                 |

## How It Works (Flow)

graph TD
    A[Admin signs & sets game] --> B[Users create private rooms]
    B --> C[Invite friends]
    C --> D[Everyone places bet before kickoff]
    D --> E[Game ends]
    E --> F[Anyone submits result]
    F --> G[5-hour voting window]
    G -->|Consensus| H[Auto payout executed]
    G -->|Dispute| I[Supreme Court activated]
    I --> J[24-hour final decision]
    J --> H
    H --> K[Users claim winnings] 


## 📌 Overview
The **BettingContract** allows users to participate in **private, invite‑only betting rooms** for predefined games. Bets are placed in USD tokens with optional collateral (USD or TRUST). Results are submitted by participants and validated through decentralized voting.

If all bettors agree → winnings are distributed automatically.
If disagreement occurs → the case escalates to the **Supreme Court**, composed of:
- The protocol admin
- A trusted external member
- Top users by honesty points

The system uses **collateral + honesty points** to discourage dishonest behavior.

---

## 📦 Features
- 🔒 Private betting rooms (invite-only)
- 🧮 Fixed betting amounts per room
- 🏦 Collateral requirement (USD or TRUST)
- ⏱ Time windows: betting, result submission, dispute, abandonment
- ⚖ Consensus-based validation (approve/reject)
- 👑 Supreme Court dispute system
- 💵 Automated distributions & fee system
- 💰 Refunds for inactive rooms or abandoned games
- 🏆 Honesty points + leaderboard
- 🧾 Event emissions for off-chain indexing

---

## 🏗 Contract Architecture
```
BettingContract
├── constants.rs
├── errors.rs
├── events.rs
├── storage.rs
├── types.rs
└── lib.rs (contract implementation)
```

### Key Modules
- **constants.rs** – Percentages, point values, time windows.
- **errors.rs** – Typed errors for precise failure handling.
- **events.rs** – All emitted Soroban events.
- **storage.rs** – Persistent state (bets, rooms, games, points, pools).
- **types.rs** – Structs and enums for core logic.

---

## 🧱 Data Structures
### `Game`
Represents a real-world event.
- id
- startTime / endTime
- active

### `Bet`
Represents a user bet.
- id
- gameid
- setting
- bet (BetKey)
- amount_bet
- collateralUsd

### `PrivateBet` (Betting Room)
- id
- gameid
- amount_bet_min
- users_invated
- active
- settingAdmin

### `ResultGame`
User submitted result.
- result
- description
- distribution_executed
- pause

### `Assessment`
Voting on results.
- UsersApprove
- UsersReject

### Enums
- **BetKey** – Team_local / Tie / Team_away / Cancel
- **AssessmentKey** – approve / reject
- **ClaimType** – Supreme / Protocol / User

---

## 🔁 Betting Flow
```
1. Admin adds game
2. Room admin creates private betting room
3. Members are invited
4. Users place bets → collateral locked
5. Game ends → bettors submit result
6. Voting window (5 hours)
    • If consensus → payout
    • If not → Supreme Court
7. Distribution executed
8. Users claim winnings or refunds
```

---

## 💳 Fees & Rewards
- **Protocol fee:** 5% of winners' profit
- **Supreme Court fee:** 3% (only in disputes)
- **Fines:** dishonest users lose their collateral
- **Honesty system:** good behavior → points → leaderboard → Supreme Court eligibility

---

## 🛠 Key Functions
### Initialization
- `__constructor`

### Betting
- `set_game`
- `set_private_bet`
- `add_user_privateBet`
- `bet`

### Result Submission
- `summitResult`
- `assessResult`

### Dispute / Supreme Court
- `setResult_supremCourt`
- `AssestResult_supremCourt`

### Distribution & Claims
- `execute_distribution`
- `claim`
- `claim_refund`

### Helpers
- `update_leaderboard`
- `what_kind_user`
- `transfer`

---

## 🗄 Storage
The contract uses Soroban's key-value ledger storage for:
- Games
- Bets
- Rooms
- Honesty points
- Voting data
- Claim flags
- Pools and balances
- Admin + Supreme Court configuration

Optimized for minimal reads/writes.

---

## 📡 Events
Emitted during:
- Game creation
- Room creation / invitations
- Bet placement
- Result submission
- Result voting
- Distributions
- Claims
- Honesty updates

Useful for UI and indexers.

---

## 🔐 Security
This contract follows strict controls:
- `require_auth()` for user actions
- Ed25519 signature validation for games
- No reentrancy (as per Soroban token model)
- Timestamp-based windows (no oracle needed)
- Anti-cheating via collateral + honesty points
- Prevents double-claim with flags
- Immutable deployment (no upgrade path)

---

## 🚀 Deployment
### Build
```
cargo build --target wasm32-unknown-unknown --release
```

### Deploy
```
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/betting_contract.wasm
```

### Initialize
```
soroban contract invoke \
  --id <contract_id> \
  --fn __constructor \
  --arg <admin> \
  --arg <admin_pubkey> \
  --arg <usd_token> \
  --arg <trust_token> \
  --arg <supreme_court>
```

---

## 🧪 Testing
- Use Soroban local test environment.
- Test all flows: bets, refunds, disputes.
- Simulate time via mocked ledger timestamps.

---

## 📄 License
MIT 

---


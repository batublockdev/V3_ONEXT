#![no_std]
use core::{f32::consts::E, panic, result};

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, vec,
    xdr::{ScVal, ToXdr, WriteXdr},
    Address, Bytes, BytesN, Env, IntoVal, String, Symbol, Vec,
};

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Game {
    pub id: i128,
    pub active: bool,
    pub league: i128,
    pub description: String,
    pub team_local: i128,
    pub team_away: i128,
    pub startTime: u32,
    pub endTime: u32,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResultGame {
    pub id: i128,
    pub gameid: i128,
    pub setting: i128,
    pub description: String,
    pub result: BetKey,
    pub pause: bool,
    pub distribution_executed: bool,
}
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CountVotesResult {
    pub id: i128,
    pub gameid: i128,
    pub aproved: i128,
    pub rejected: i128,
}
#[contracttype]
#[derive(Clone)]
pub struct ResultAssessment {
    pub id: i128,
    pub gameid: i128,
    pub setting: i128,
    pub UsersApprove: Vec<Address>,
    pub UsersReject: Vec<Address>,
}
#[contracttype]
#[derive(Clone)]
pub struct ResultAssessmentSupreme {
    pub id: i128,
    pub gameid: i128,
    pub Admin: bool,
    pub ExternalUser: bool,
    pub countHonestyUsers: Vec<Address>,
}
#[contracttype]
#[derive(Clone)]
pub struct PrivateBet {
    pub id: i128,
    pub gameid: i128,
    pub active: bool,
    pub settingAdmin: Address,
    pub description: String,
    pub amount_bet_min: i128,
    pub users_invated: Vec<Address>,
}
#[contracttype]
#[derive(Clone)]
pub struct LastB {
    pub id: i128,
    pub lastBet: BetKey,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bet {
    pub id: i128,
    pub gameid: i128,
    pub Setting: i128,
    pub bet: BetKey,
    pub amount_bet: i128,
    pub collateralUsd: bool,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Game(i128),
    AmountUsers(i128),
    AmountUsersGame(i128),
    AmountUsersVoted(i128),
    AmountUsersVotedGame(i128),
    TotalBet(i128),
    ResultSupreme(i128),
    Result(i128),
    HonestyPoints(Address),
    ClaimWinner(Address),
    ClaimSupreme(i128),
    ClaimProtocol,
    ClaimProtocolTrust,
    StakeMinAmount,
    StakeUserAmount(Address),
    StakeUserGameAmount(Address, i128),
    ResultAssessment(i128),
    ResultAssessmentSupreme(i128),
    votesSupreme(i128),
    GameSummiter(i128),
    listBoard,
    SetPrivateBet(i128),
    Bet(Address, i128),
    PrivateBetList(i128),
    lastBet(i128),
    Result_Local_team(i128),
    Result_Away_team(i128),
    Result_Tie_team(i128),
    Result_Cancel_team(i128),
    ListBetUser(i128),
    NotAssesedYet(i128, BetKey),
    CollateralTrustNotAssesedYet(i128, BetKey),
    CollateralUsdNotAssesedYet(i128, BetKey),

    Rejected(i128, BetKey),
    CollateralUsdRejected(i128, BetKey),
    CollateralTrustRejected(i128, BetKey),
    Approved(i128, BetKey),
    CollateralUsdApproved(i128, BetKey),
    CollateralTrustApproved(i128, BetKey),
    pool(i128),
    Complain(i128),
    winnerPool(i128),
    loserPool(i128),
    UserWithdraw(i128, Address),
    UserWithdrawSupreme(i128, Address),
}
#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub enum BetKey {
    Team_local,
    Team_away,
    Tie,
    Cancel,
}
#[derive(Clone, PartialEq, Eq)]
#[contracttype]
pub enum AssessmentKey {
    approve,
    reject,
}

#[derive(Clone, PartialEq, Eq)]
#[contracttype]
pub enum ClaimType {
    Supreme,
    Protocol,
    User,
}
#[contracttype]
#[derive(Clone)]
struct Summiter {
    user: Address,
    stakeAmount: i128,
    gameId: i128,
}

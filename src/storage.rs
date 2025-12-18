use crate::types::{
    AssessmentKey, Bet, BetKey, ClaimType, CountVotesResult, DataKey, Game, LastB, PrivateBet,
    ResultAssessment, ResultAssessmentSupreme, ResultGame,
};
use soroban_sdk::{symbol_short, Address, BytesN, Env, String, Symbol, Vec};
const ADMIN_KEY: Symbol = Symbol::short("ADMIN");
const ADMIN_PUB_KEY: Symbol = Symbol::short("Adm_key");

const SUPREME_KEY: Symbol = Symbol::short("SUPREME");
const SUPREME_H: Symbol = Symbol::short("SUPREME_H");

const TOKEN_USD_KEY: Symbol = Symbol::short("TOKEN_USD");
const TOKEN_TRUST_KEY: Symbol = Symbol::short("TK_TRUST");
const LEADERBOARD: Symbol = symbol_short!("LB");
const SUMITTERS_HISTORY: Symbol = symbol_short!("H_S");
const COUNTER: Symbol = symbol_short!("COUNTER");
const x: Symbol = symbol_short!("x");
pub fn get_dummyusser(env: &Env) -> Address {
    Address::from_string(&String::from_str(
        env,
        "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
    ))
}

pub fn has_init(env: &Env) -> bool {
    env.storage().instance().has(&ADMIN_KEY)
}
pub fn init(
    env: Env,
    admin: Address,
    admin_pubkey: BytesN<32>,
    token_usd: Address,
    token_trust: Address,
    supreme_court: Address,
) {
    // save the admin
    env.storage().instance().set(&ADMIN_KEY, &admin);
    env.storage().instance().set(&ADMIN_PUB_KEY, &admin_pubkey);

    // save the token addresses
    env.storage().instance().set(&TOKEN_USD_KEY, &token_usd);
    env.storage().instance().set(&TOKEN_TRUST_KEY, &token_trust);
    env.storage().instance().set(&SUPREME_KEY, &supreme_court);
}
pub fn save_supreme_honesty(env: Env, Users: Vec<Address>) {
    env.storage().instance().set(&SUPREME_H, &Users);
}
pub fn get_supreme_honesty(env: Env) -> Vec<Address> {
    env.storage()
        .instance()
        .get(&SUPREME_H)
        .unwrap_or(Vec::new(&env))
}
pub fn get_supreme(env: Env) -> Address {
    env.storage()
        .instance()
        .get(&SUPREME_KEY)
        .unwrap_or_else(|| panic!("contract not initialized"))
}
pub fn get_usd(env: Env) -> Address {
    env.storage()
        .instance()
        .get(&TOKEN_USD_KEY)
        .unwrap_or_else(|| panic!("contract not initialized"))
}
pub fn get_trust(env: Env) -> Address {
    env.storage()
        .instance()
        .get(&TOKEN_TRUST_KEY)
        .unwrap_or_else(|| panic!("contract not initialized"))
}
pub fn get_admin(env: Env) -> Address {
    env.storage()
        .instance()
        .get(&ADMIN_KEY)
        .unwrap_or_else(|| panic!("contract not initialized"))
}
pub fn get_admin_pubkey(env: Env) -> BytesN<32> {
    env.storage()
        .instance()
        .get(&ADMIN_PUB_KEY)
        .unwrap_or_else(|| panic!("contract not initialized"))
}
pub fn is_summiter(env: Env, setting: i128, user: Address) -> bool {
    let summiter = env
        .storage()
        .persistent()
        .get(&DataKey::GameSummiter(setting))
        .unwrap_or(get_dummyusser(&env));
    if summiter == user {
        true
    } else {
        false
    }
}
pub fn set_summiter(env: Env, user: Address, setting: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::GameSummiter(setting), &user);
}
pub fn get_leaderboard(env: Env) -> Vec<(Address, i128)> {
    env.storage()
        .persistent()
        .get(&DataKey::listBoard)
        .unwrap_or(Vec::new(&env))
}
pub fn set_leaderboard(env: Env, leaderboard: Vec<(Address, i128)>) -> bool {
    env.storage()
        .persistent()
        .set(&DataKey::listBoard, &leaderboard);
    true
}
pub fn set_stakeAmount_user(env: Env, user: Address, stakeAmount: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::StakeUserAmount(user), &stakeAmount);
}

pub fn set_stakeAmount_user_game(env: Env, user: Address, game: i128) {
    let amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::StakeUserAmount(user.clone()))
        .unwrap_or(0);
    env.storage()
        .persistent()
        .set(&DataKey::StakeUserGameAmount(user, game), &amount);
}
pub fn get_stakeAmount_user_game(env: Env, user: Address, game: i128) -> i128 {
    let amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::StakeUserGameAmount(user, game))
        .unwrap_or(0);
    amount
}
pub fn set_Min_stakeAmount(env: Env, stakeAmount: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::StakeMinAmount, &stakeAmount);
}
pub fn get_Min_stakeAmount(env: Env) -> i128 {
    let amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::StakeMinAmount)
        .unwrap_or(0);
    amount
}

pub fn add_privateSettingList(env: Env, gameId: i128, setting: i128) {
    let mut listedBet: Vec<(i128)> = env
        .storage()
        .persistent()
        .get(&DataKey::PrivateBetList(gameId))
        .unwrap_or(Vec::new(&env));
    listedBet.push_back(setting);
    env.storage()
        .persistent()
        .set(&DataKey::PrivateBetList(gameId), &listedBet);
}

pub fn get_privateSettingList(env: Env, gameId: i128) -> Vec<(i128)> {
    let list: Vec<(i128)> = env
        .storage()
        .persistent()
        .get(&DataKey::PrivateBetList(gameId))
        .unwrap_or(Vec::new(&env));
    list
}

pub fn add_bet(env: Env, user: Address, bet: Bet) {
    env.storage()
        .persistent()
        .set(&DataKey::Bet(user.clone(), bet.clone().Setting), &bet);
}
pub fn get_bet(env: Env, user: Address, setting: i128) -> Bet {
    let bet: Bet = env
        .storage()
        .persistent()
        .get(&DataKey::Bet(user.clone(), setting))
        .unwrap_or_else(|| panic!("No bet found for this user"));
    bet
}
pub fn does_bet_active(env: Env, bet: Bet) -> bool {
    let lastBet: LastB = env
        .storage()
        .persistent()
        .get(&DataKey::lastBet(bet.clone().Setting))
        .unwrap_or(LastB {
            id: 0,
            lastBet: BetKey::Team_local,
        });
    if lastBet.clone().id == 0 {
        // it means this is the fisrt bet for this setting
        env.storage().persistent().set(
            &DataKey::lastBet(bet.clone().Setting),
            &LastB {
                id: bet.clone().Setting,
                lastBet: bet.clone().bet,
            },
        );
        return false;
    }
    if lastBet.lastBet != bet.clone().bet {
        return true;
    } else {
        return false;
    }
}
pub fn active_private_setting(env: Env, setting: i128, active: bool) {
    env.storage().persistent().update(
        &DataKey::SetPrivateBet(setting),
        |old: Option<PrivateBet>| {
            let mut res = old.unwrap_or(PrivateBet {
                id: 0,
                gameid: 0,
                active: false,
                settingAdmin: Address::from_string(&String::from_str(
                    &env,
                    "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
                )),
                description: String::from_slice(&env, "No private bet found"),
                amount_bet_min: 0,
                users_invated: Vec::new(&env),
            });
            res.active = active;
            res
        },
    );
}

pub fn get_PrivateBet(env: Env, setting: i128) -> PrivateBet {
    env.storage()
        .persistent()
        .get(&DataKey::SetPrivateBet(setting))
        .unwrap_or(PrivateBet {
            id: 0,
            gameid: 0,
            active: false,
            settingAdmin: Address::from_string(&String::from_str(
                &env,
                "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
            )),
            description: String::from_slice(&env, "No private bet found"),
            amount_bet_min: 0,
            users_invated: Vec::new(&env),
        })
}

pub fn set_game(env: Env, game: Game) {
    let gameReceive = env
        .storage()
        .persistent()
        .get(&DataKey::Game(game.id))
        .unwrap_or(Game {
            id: 0,
            active: false,
            league: 0,
            description: String::from_slice(&env, "No game found"),
            team_local: 0,
            team_away: 0,
            startTime: 0,
            endTime: 0,
        });
    if gameReceive.id != 0 {
        panic!("Game with this ID already exists");
    }
    env.storage()
        .persistent()
        .set(&DataKey::Game(game.id), &game);
}
pub fn set_privateSetting(env: Env, privateBet: PrivateBet) {
    self::verifySettingId(env.clone(), privateBet.id);
    env.storage()
        .persistent()
        .set(&DataKey::SetPrivateBet(privateBet.id), &privateBet);
}
pub fn add_HonestyPoints(env: Env, user: Address, points: i128) {
    let honesty: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::HonestyPoints(user.clone()))
        .unwrap_or(0);
    let total_honesty = honesty + points;
    env.storage()
        .persistent()
        .set(&DataKey::HonestyPoints(user.clone()), &total_honesty);
}
pub fn get_HonestyPoints(env: Env, user: Address) -> i128 {
    let honesty: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::HonestyPoints(user.clone()))
        .unwrap_or(0);
    honesty
}
pub fn add_UsersAmount(env: Env, game_setting: i128, plus: i128) {
    let amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::AmountUsers(game_setting))
        .unwrap_or(0);
    let total = amount + plus;
    env.storage()
        .persistent()
        .set(&DataKey::AmountUsers(game_setting), &total);
}
pub fn add_UsersAmountGame(env: Env, game_setting: i128, plus: i128) {
    let amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::AmountUsersGame(game_setting))
        .unwrap_or(0);
    let total = amount + plus;
    env.storage()
        .persistent()
        .set(&DataKey::AmountUsersGame(game_setting), &total);
}

pub fn UsersAmount(env: Env, game_setting: i128) -> bool {
    let amountX: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::AmountUsersVoted(game_setting))
        .unwrap_or(0);
    let amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::AmountUsers(game_setting))
        .unwrap_or(0);
    if amountX == amount {
        true
    } else {
        false
    }
}
pub fn UsersAmountGame(env: Env, game_setting: i128) -> bool {
    let amountX: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::AmountUsersVotedGame(game_setting))
        .unwrap_or(0);
    let amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::AmountUsersGame(game_setting))
        .unwrap_or(0);
    if amountX == amount {
        true
    } else {
        false
    }
}
pub fn add_UsersAmountVotedGame(env: Env, game_setting: i128) {
    let amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::AmountUsersVotedGame(game_setting))
        .unwrap_or(0);
    let total = amount + 1;
    env.storage()
        .persistent()
        .set(&DataKey::AmountUsersVotedGame(game_setting), &total);
}
pub fn add_UsersAmountVoted(env: Env, game_setting: i128) {
    let amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::AmountUsersVoted(game_setting))
        .unwrap_or(0);
    let total = amount + 1;
    env.storage()
        .persistent()
        .set(&DataKey::AmountUsersVoted(game_setting), &total);
}
pub fn verifySettingId(env: Env, SettingId: i128) {
    let privateBet = env
        .storage()
        .persistent()
        .get(&DataKey::SetPrivateBet(SettingId))
        .unwrap_or(PrivateBet {
            id: 0,
            gameid: 0,
            active: false,
            settingAdmin: Address::from_string(&String::from_str(
                &env,
                "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
            )),
            description: String::from_slice(&env, "No private bet found"),
            amount_bet_min: 0,
            users_invated: Vec::new(&env),
        });
    if privateBet.id != 0 {
        panic!("Private setting with this ID already exists");
    }
}

pub fn get_game(env: Env, game_id: i128) -> Game {
    let game = env
        .storage()
        .persistent()
        .get(&DataKey::Game(game_id))
        .unwrap_or(Game {
            id: 0,
            active: false,
            league: 0,
            description: String::from_slice(&env, "No game found"),
            team_local: 0,
            team_away: 0,
            startTime: 0,
            endTime: 0,
        });
    game
}
pub fn set_result_team_count(env: Env, gameid: i128, result: BetKey) {
    match result {
        BetKey::Team_local => {
            let count: i128 = env
                .storage()
                .persistent()
                .get(&DataKey::Result_Local_team(gameid))
                .unwrap_or(0);
            let total_count = count + 1;
            env.storage()
                .persistent()
                .set(&DataKey::Result_Local_team(gameid), &total_count);
        }
        BetKey::Team_away => {
            let count: i128 = env
                .storage()
                .persistent()
                .get(&DataKey::Result_Away_team(gameid))
                .unwrap_or(0);
            let total_count = count + 1;
            env.storage()
                .persistent()
                .set(&DataKey::Result_Away_team(gameid), &total_count);
        }
        BetKey::Tie => {
            let count: i128 = env
                .storage()
                .persistent()
                .get(&DataKey::Result_Tie_team(gameid))
                .unwrap_or(0);
            let total_count = count + 1;
            env.storage()
                .persistent()
                .set(&DataKey::Result_Tie_team(gameid), &total_count);
        }
        BetKey::Cancel => {
            let count: i128 = env
                .storage()
                .persistent()
                .get(&DataKey::Result_Cancel_team(gameid))
                .unwrap_or(0);
            let total_count = count + 1;
            env.storage()
                .persistent()
                .set(&DataKey::Result_Cancel_team(gameid), &total_count);
        }
    }
}
pub fn overall_result(env: Env, gameid: i128) -> i128 {
    let local: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::Result_Local_team(gameid))
        .unwrap_or(0);
    let away: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::Result_Away_team(gameid))
        .unwrap_or(0);
    let cancel: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::Result_Cancel_team(gameid))
        .unwrap_or(0);
    let tie: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::Result_Tie_team(gameid))
        .unwrap_or(0);
    if local == away && local == tie && local == cancel {
        return 0; // no clear result
    }
    if local > away && local > tie && local > cancel {
        1 // local win
    } else if away > local && away > tie && away > cancel {
        2 // away win
    } else if cancel > local && cancel > tie && cancel > away {
        3 // cancel win
    } else {
        4 // tie win
    }
}
pub fn set_ResultGame(env: Env, result: ResultGame) {
    env.storage()
        .persistent()
        .set(&DataKey::Result(result.clone().setting), &result);
}
pub fn set_ResultGameSupreme(env: Env, result: ResultGame) {
    env.storage()
        .persistent()
        .set(&DataKey::ResultSupreme(result.clone().gameid), &result);
}
pub fn get_ResultGame(env: Env, setting: i128) -> ResultGame {
    let result = env
        .storage()
        .persistent()
        .get(&DataKey::Result(setting))
        .unwrap_or(ResultGame {
            id: 0,
            gameid: 0,
            setting: 0,
            description: String::from_slice(&env, "No result found"),
            result: BetKey::Team_local,
            pause: false,
            distribution_executed: false,
        });
    result
}
pub fn get_ResultGameSupreme(env: Env, gameid: i128) -> ResultGame {
    let result = env
        .storage()
        .persistent()
        .get(&DataKey::ResultSupreme(gameid))
        .unwrap_or(ResultGame {
            id: 0,
            gameid: 0,
            setting: 0,
            description: String::from_slice(&env, "No result found"),
            result: BetKey::Team_local,
            pause: false,
            distribution_executed: false,
        });
    result
}
pub fn puase_ResultGame(env: Env, setting: i128, pause: bool) {
    env.storage()
        .persistent()
        .update(&DataKey::Result(setting), |old: Option<ResultGame>| {
            let mut res = old.unwrap_or(ResultGame {
                id: 0,
                gameid: 0,
                setting: 0,
                description: String::from_str(&env, ""),
                result: BetKey::Team_local,
                pause: false,
                distribution_executed: false,
            });
            res.pause = pause;
            res
        });
}
pub fn distribution_ResultGame(env: Env, setting: i128) {
    env.storage()
        .persistent()
        .update(&DataKey::Result(setting), |old: Option<ResultGame>| {
            let mut res = old.unwrap_or(ResultGame {
                id: 0,
                gameid: 0,
                setting: 0,
                description: String::from_str(&env, ""),
                result: BetKey::Team_local,
                pause: false,
                distribution_executed: false,
            });
            res.distribution_executed = true;
            res
        });
}

pub fn get_ListBetUser(env: Env, gameid: i128) -> Vec<(Address)> {
    env.storage()
        .persistent()
        .get(&DataKey::ListBetUser(gameid))
        .unwrap_or(Vec::new(&env))
}
pub fn get_Bet(env: Env, user: Address, setting: i128) -> Bet {
    env.storage()
        .persistent()
        .get(&DataKey::Bet(user.clone(), setting))
        .unwrap_or(Bet {
            id: 0,
            gameid: 0,
            Setting: 0,
            bet: BetKey::Team_local,
            amount_bet: 0,
            collateralUsd: false,
        })
}

// summitter
pub fn get_ClaimSupreme(env: Env, gameid: i128) -> i128 {
    let amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::ClaimSupreme(gameid))
        .unwrap_or(0);
    amount
}

pub fn add_ClaimSupreme(env: Env, gameid: i128, newAmount: i128) {
    let money: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::ClaimSupreme(gameid))
        .unwrap_or(0);
    let total_money = money + newAmount;
    env.storage()
        .persistent()
        .set(&DataKey::ClaimSupreme(gameid), &total_money);
}
/// protocol trust
pub fn get_ClaimProtocolTrust(env: Env) -> i128 {
    let amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::ClaimProtocolTrust)
        .unwrap_or(0);
    amount
}

pub fn add_ClaimProtocolTrust(env: Env, newAmount: i128) {
    let mut currentAmount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::ClaimProtocolTrust)
        .unwrap_or(0);

    currentAmount += newAmount;
    env.storage()
        .persistent()
        .set(&DataKey::ClaimProtocolTrust, &currentAmount);
}
// protocol
///
pub fn get_ClaimProtocol(env: Env) -> i128 {
    let amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::ClaimProtocol)
        .unwrap_or(0);
    amount
}
pub fn zero_ClaimProtocol(env: Env) {
    let zero: i128 = 0;
    env.storage()
        .persistent()
        .set(&DataKey::ClaimProtocol, &zero);
    env.storage()
        .persistent()
        .set(&DataKey::ClaimProtocolTrust, &zero);
}
pub fn add_ClaimProtocol(env: Env, newAmount: i128) {
    let mut currentAmount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::ClaimProtocol)
        .unwrap_or(0);

    currentAmount += newAmount;
    env.storage()
        .persistent()
        .set(&DataKey::ClaimProtocol, &currentAmount);
}
pub fn get_ResultAssessment(env: Env, setting: i128) -> ResultAssessment {
    env.storage()
        .persistent()
        .get(&DataKey::ResultAssessment(setting))
        .unwrap_or(ResultAssessment {
            id: 0,
            gameid: 0,
            setting: 0,
            UsersApprove: Vec::new(&env),
            UsersReject: Vec::new(&env),
        })
}
pub fn set_ResultAssessment(env: Env, setting: i128, data: ResultAssessment) {
    env.storage()
        .persistent()
        .set(&DataKey::ResultAssessment(setting), &data);
}
pub fn get_ResultAssessmentSupreme(env: Env, gameid: i128) -> ResultAssessmentSupreme {
    env.storage()
        .persistent()
        .get(&DataKey::ResultAssessmentSupreme(gameid))
        .unwrap_or(ResultAssessmentSupreme {
            id: 0,
            gameid: 0,
            Admin: false,
            ExternalUser: false,
            countHonestyUsers: Vec::new(&env),
        })
}
pub fn set_ResultAssessmentSupreme(env: Env, gameid: i128, data: ResultAssessmentSupreme) {
    env.storage()
        .persistent()
        .set(&DataKey::ResultAssessmentSupreme(gameid), &data);
}
pub fn get_CountVoteSupreme(env: Env, gameid: i128) -> CountVotesResult {
    env.storage()
        .persistent()
        .get(&DataKey::votesSupreme(gameid))
        .unwrap_or(CountVotesResult {
            id: 0,
            gameid: 0,
            aproved: 0,
            rejected: 0,
        })
}
pub fn set_CountVoteSupreme(env: Env, gameid: i128, data: CountVotesResult) {
    env.storage()
        .persistent()
        .set(&DataKey::votesSupreme(gameid), &data);
}
pub fn existBet(env: Env, game_id: i128) -> (bool, u32, u32, bool) {
    let mut check: bool = false;
    let receiveGame = env
        .storage()
        .persistent()
        .get(&DataKey::Game(game_id))
        .unwrap_or(Game {
            id: 0,
            active: false,
            league: 0,
            description: String::from_slice(&env, "No game found"),
            team_local: 0,
            team_away: 0,
            startTime: 0,
            endTime: 0,
        });

    if receiveGame.id == 0 {
        check = false;
    } else {
        check = true;
    }
    (
        check,
        receiveGame.startTime,
        receiveGame.endTime,
        receiveGame.active,
    )
}
pub fn add_total_bet(env: Env, game_id: i128, Amount: i128) {
    let total_amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::TotalBet(game_id))
        .unwrap_or(0);
    let Amountx = total_amount + Amount;
    env.storage()
        .persistent()
        .set(&DataKey::TotalBet(game_id), &Amountx);
}

pub fn get_total_bet(env: Env, game_id: i128) -> i128 {
    let total_amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::TotalBet(game_id))
        .unwrap_or(0);
    total_amount
}
pub fn add_not_assesed_yet(
    env: Env,
    game_id: i128,
    Amount: i128,
    bet: BetKey,
    collateralusd: bool,
    collateral: i128,
) {
    let total_amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::NotAssesedYet(game_id, bet.clone()))
        .unwrap_or(0);
    let Amountx = total_amount + Amount;
    env.storage()
        .persistent()
        .set(&DataKey::NotAssesedYet(game_id, bet.clone()), &Amountx);
    if collateralusd {
        let total_amount_Collateral_usd: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::CollateralUsdNotAssesedYet(game_id, bet.clone()))
            .unwrap_or(0);
        let AmountCollateral = total_amount_Collateral_usd + collateral;
        env.storage().persistent().set(
            &DataKey::CollateralUsdNotAssesedYet(game_id, bet.clone()),
            &AmountCollateral,
        );
    } else {
        let total_amount_Collateral_Trust: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::CollateralTrustNotAssesedYet(game_id, bet.clone()))
            .unwrap_or(0);
        let AmountCollateral = total_amount_Collateral_Trust + collateral;
        env.storage().persistent().set(
            &DataKey::CollateralTrustNotAssesedYet(game_id, bet.clone()),
            &AmountCollateral,
        );
    }
}
pub fn delete_not_assesed_yet(
    env: Env,
    game_id: i128,
    Amount: i128,
    bet: BetKey,
    collateralusd: bool,
    collateral: i128,
) {
    let total_amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::NotAssesedYet(game_id, bet.clone()))
        .unwrap_or(0);
    let Amountx = total_amount - Amount;
    env.storage()
        .persistent()
        .set(&DataKey::NotAssesedYet(game_id, bet.clone()), &Amountx);
    if collateralusd {
        let total_amount_Collateral_usd: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::CollateralUsdNotAssesedYet(game_id, bet.clone()))
            .unwrap_or(0);
        let AmountCollateral = total_amount_Collateral_usd - collateral;
        env.storage().persistent().set(
            &DataKey::CollateralUsdNotAssesedYet(game_id, bet.clone()),
            &AmountCollateral,
        );
    } else {
        let total_amount_Collateral_Trust: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::CollateralTrustNotAssesedYet(game_id, bet.clone()))
            .unwrap_or(0);
        let AmountCollateral = total_amount_Collateral_Trust - collateral;
        env.storage().persistent().set(
            &DataKey::CollateralTrustNotAssesedYet(game_id, bet.clone()),
            &AmountCollateral,
        );
    }
}
pub fn get_not_assesed_yet(env: Env, game_id: i128, bet: BetKey) -> i128 {
    let total_amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::NotAssesedYet(game_id, bet.clone()))
        .unwrap_or(0);
    total_amount
}
pub fn get_not_usdCollateral_assesed_yet(env: Env, game_id: i128, bet: BetKey) -> i128 {
    let total_amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::CollateralUsdNotAssesedYet(game_id, bet.clone()))
        .unwrap_or(0);
    total_amount
}
pub fn get_not_TrustCollateral_assesed_yet(env: Env, game_id: i128, bet: BetKey) -> i128 {
    let total_amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::CollateralTrustNotAssesedYet(game_id, bet.clone()))
        .unwrap_or(0);
    total_amount
}
pub fn add_approve_total(
    env: Env,
    game_id: i128,
    Amount: i128,
    bet: BetKey,
    collateralusd: bool,
    collateral: i128,
) {
    let total_amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::Approved(game_id, bet.clone()))
        .unwrap_or(0);
    let Amountx = total_amount + Amount;
    env.storage()
        .persistent()
        .set(&DataKey::Approved(game_id, bet.clone()), &Amountx);
    if collateralusd {
        let total_amount_Collateral_usd: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::CollateralUsdApproved(game_id, bet.clone()))
            .unwrap_or(0);
        let AmountCollateral = total_amount_Collateral_usd + collateral;
        env.storage().persistent().set(
            &DataKey::CollateralUsdApproved(game_id, bet.clone()),
            &AmountCollateral,
        );
    } else {
        let total_amount_Collateral_Trust: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::CollateralTrustApproved(game_id, bet.clone()))
            .unwrap_or(0);
        let AmountCollateral = total_amount_Collateral_Trust + collateral;
        env.storage().persistent().set(
            &DataKey::CollateralTrustApproved(game_id, bet.clone()),
            &AmountCollateral,
        );
    }
}
pub fn get_approve_total(env: Env, game_id: i128, bet: BetKey) -> i128 {
    let total_amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::Approved(game_id, bet.clone()))
        .unwrap_or(0);
    total_amount
}
pub fn get_CollateralTrust_approve_total(env: Env, game_id: i128, bet: BetKey) -> i128 {
    let total_amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::CollateralTrustApproved(game_id, bet.clone()))
        .unwrap_or(0);
    total_amount
}
pub fn get_CollateralUsd_approve_total(env: Env, game_id: i128, bet: BetKey) -> i128 {
    let total_amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::CollateralUsdApproved(game_id, bet.clone()))
        .unwrap_or(0);
    total_amount
}

pub fn add_reject_total(
    env: Env,
    game_id: i128,
    Amount: i128,
    bet: BetKey,
    collateralusd: bool,
    collateral: i128,
) {
    let total_amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::Rejected(game_id, bet.clone()))
        .unwrap_or(0);
    let Amountx = total_amount + Amount;
    env.storage()
        .persistent()
        .set(&DataKey::Rejected(game_id, bet.clone()), &Amountx);
    if collateralusd {
        let total_amount_Collateral_usd: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::CollateralUsdRejected(game_id, bet.clone()))
            .unwrap_or(0);
        let AmountCollateral = total_amount_Collateral_usd + collateral;
        env.storage().persistent().set(
            &DataKey::CollateralUsdRejected(game_id, bet.clone()),
            &AmountCollateral,
        );
    } else {
        let total_amount_Collateral_Trust: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::CollateralTrustRejected(game_id, bet.clone()))
            .unwrap_or(0);
        let AmountCollateral = total_amount_Collateral_Trust + collateral;
        env.storage().persistent().set(
            &DataKey::CollateralTrustRejected(game_id, bet.clone()),
            &AmountCollateral,
        );
    }
}
pub fn get_reject_total(env: Env, game_id: i128, bet: BetKey) -> i128 {
    let total_amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::Rejected(game_id, bet.clone()))
        .unwrap_or(0);
    total_amount
}
pub fn get_CollateralTrust_reject_total(env: Env, game_id: i128, bet: BetKey) -> i128 {
    let total_amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::CollateralTrustRejected(game_id, bet.clone()))
        .unwrap_or(0);
    total_amount
}
pub fn get_CollateralUsd_reject_total(env: Env, game_id: i128, bet: BetKey) -> i128 {
    let total_amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::CollateralUsdRejected(game_id, bet.clone()))
        .unwrap_or(0);
    total_amount
}
pub fn set_pool_total(env: Env, game_id: i128, amount: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::pool(game_id), &amount);
}
pub fn get_pool_total(env: Env, game_id: i128) -> i128 {
    let amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::pool(game_id))
        .unwrap_or(0);
    amount
}

pub fn save_complain(env: Env, game_id: i128, complain: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::Complain(game_id), &complain);
}
pub fn get_complain(env: Env, game_id: i128) -> i128 {
    let complain: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::Complain(game_id))
        .unwrap_or(0);
    complain
}
pub fn save_winnerPool(env: Env, game_id: i128, amount: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::winnerPool(game_id), &amount);
}
pub fn get_winnerPool(env: Env, game_id: i128) -> i128 {
    let amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::winnerPool(game_id))
        .unwrap_or(0);
    amount
}
pub fn save_loserPool(env: Env, game_id: i128, amount: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::loserPool(game_id), &amount);
}
pub fn get_loserPool(env: Env, game_id: i128) -> i128 {
    let amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::loserPool(game_id))
        .unwrap_or(0);
    amount
}
pub fn set_didUserWithdraw(env: Env, user: Address, game_id: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::UserWithdraw(game_id, user), &true);
}
pub fn get_didUserWithdraw(env: Env, user: Address, game_id: i128) -> bool {
    let didWithdraw: bool = env
        .storage()
        .persistent()
        .get(&DataKey::UserWithdraw(game_id, user))
        .unwrap_or(false);
    didWithdraw
}
pub fn set_didUserWithdrawSupreme(env: Env, user: Address, game_id: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::UserWithdrawSupreme(game_id, user), &true);
}
pub fn get_didUserWithdrawSupreme(env: Env, user: Address, game_id: i128) -> bool {
    let didWithdraw: bool = env
        .storage()
        .persistent()
        .get(&DataKey::UserWithdrawSupreme(game_id, user))
        .unwrap_or(false);
    didWithdraw
}

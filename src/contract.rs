#![no_std]

use crate::{
    bettingTrait::betting,
    errors::BettingError,
    events::BettingEvents,
    storage,
    types::{
        AssessmentKey, Bet, BetKey, ClaimType, CountVotesResult, DataKey, Game, LastB, PrivateBet,
        ResultAssessment, ResultAssessmentSupreme, ResultGame,
    },
    Constants::{
        FIFTY_PERCENT, FIFTY_POINTS, FIVE_PERCENT, HUNDRED_POINTS, LESS_HUNDRED_POINTS,
        MINUS_TWENTY_POINTS, ONE_HOUR_SECONDS, SCORE_HISTORY_WEIGHT, TEN_PERCENT, THREE_PERCENT,
        TRUST_TOKEN_PERCENTAGE, TWENTY_PERCENT, TWENTY_POINTS, VOTE_HISTORY_WEIGHT,
    },
};
use soroban_sdk::{
    contract, contractimpl, panic_with_error, symbol_short, token, vec,
    xdr::{ScVal, ToXdr, WriteXdr},
    Address, Bytes, BytesN, Env, IntoVal, String, Symbol, Vec,
};

#[contract]
pub struct BettingContract;

#[contractimpl]
impl betting for BettingContract {
    /*
       @dev This is the constructor of the contract
       @param env Environment
       @param admin Address The address of the admin
       @param admin_pubkey BytesN<32> The public key of the admin
       @param token_usd Address The address of the USD token
       @param token_trust Address The address of the TRUST token
       @param supreme_court Address The address of the supreme court, which is the external users for the system
       @dev This funtion can be called only once, allowing to set the admin, tokens and supreme court address

    */
    fn __constructor(
        env: Env,
        admin: Address,
        admin_pubkey: BytesN<32>,
        token_usd: Address,
        token_trust: Address,
        supreme_court: Address,
    ) {
        admin.require_auth();

        // check if already initialized
        if storage::has_init(&env) {
            panic_with_error!(&env, BettingError::AlreadyInitializedError);
        }
        // Save data
        storage::init(
            env,
            admin,
            admin_pubkey,
            token_usd,
            token_trust,
            supreme_court,
        );
    }

    /*
       @dev this funtion bet on a game
       @param env Environment
       @param user Address The address of the user
       @param bet Bet The bet data
    */
    fn bet(env: Env, user: Address, bet: Bet) -> bool {
        user.require_auth();
        let contract_address = env.current_contract_address();
        let usd = storage::get_usd(env.clone());
        let trust: Address = storage::get_trust(env.clone());
        let betResult: Bet = storage::get_Bet(env.clone(), user.clone(), bet.clone().Setting);
        if betResult.id != 0 {
            panic_with_error!(&env, BettingError::BetAlready);
        }

        if bet.clone().amount_bet <= 0 {
            panic_with_error!(&env, BettingError::InvalidInputError);
        }
        if bet.clone().id == 0 || bet.clone().Setting == 0 || bet.clone().gameid == 0 {
            panic_with_error!(&env, BettingError::InvalidInputError);
        }
        let (exist, startTime, endTime, active) =
            storage::existBet(env.clone(), bet.clone().gameid);
        if !exist {
            panic_with_error!(&env, BettingError::GameDoesNotExist);
        }
        if startTime < env.ledger().timestamp() as u32 {
            panic_with_error!(&env, BettingError::GameHasAlreadyStarted);
        }

        let privateBet: PrivateBet = storage::get_PrivateBet(env.clone(), bet.clone().Setting);

        if privateBet.clone().id == 0 {
            panic_with_error!(&env, BettingError::SettingBetDoesNotExist);
        }
        if !privateBet.clone().users_invated.contains(&user) {
            panic_with_error!(&env, BettingError::PrivateBet_NotAllowToBet);
        }
        if bet.clone().amount_bet != privateBet.clone().amount_bet_min {
            panic_with_error!(&env, BettingError::PrivateBet_NotEnoughToBet);
        }
        storage::add_bet(env.clone(), user.clone(), bet.clone());

        if !privateBet.active {
            if storage::does_bet_active(env.clone(), bet.clone()) {
                storage::active_private_setting(env.clone(), bet.clone().Setting, true);
                let useramount = privateBet.clone().users_invated.len() as i128;
                storage::add_UsersAmount(env.clone(), bet.clone().gameid, useramount);

                BettingEvents::active_setting(&env, privateBet.gameid, privateBet.id);
            }
        }

        storage::add_UsersAmount(env.clone(), bet.clone().Setting, 1);

        storage::add_total_bet(env.clone(), bet.clone().gameid, bet.clone().amount_bet);
        storage::add_HonestyPoints(env.clone(), user.clone(), MINUS_TWENTY_POINTS);
        let points = storage::get_HonestyPoints(env.clone(), user.clone());
        let position: u32 = Self::adduser_board(&env, user.clone(), points);
        BettingEvents::user_honesty_points(&env, user.clone(), points, position);

        if bet.clone().collateralUsd {
            storage::add_not_assesed_yet(
                env.clone(),
                bet.clone().Setting,
                bet.clone().amount_bet,
                bet.clone().bet,
                bet.clone().collateralUsd,
                bet.clone().amount_bet * TWENTY_PERCENT / 100,
            );
            Self::moveToken(
                &env,
                &usd,
                &user,
                &contract_address,
                &bet.clone().amount_bet,
            );
            Self::moveToken(
                &env,
                &usd,
                &user,
                &contract_address,
                &((bet.clone().amount_bet * TWENTY_PERCENT) / 100),
            );
            true
        } else {
            storage::add_not_assesed_yet(
                env.clone(),
                bet.clone().Setting,
                bet.clone().amount_bet,
                bet.clone().bet,
                bet.clone().collateralUsd,
                bet.clone().amount_bet * TEN_PERCENT / 100,
            );
            Self::moveToken(
                &env,
                &usd,
                &user,
                &contract_address,
                &bet.clone().amount_bet,
            );
            Self::moveToken(
                &env,
                &trust,
                &user,
                &contract_address,
                &((bet.clone().amount_bet * TEN_PERCENT) / 100),
            );
            true
        }
    }
    /*
       @dev This function claim the refund in two conditions
       1. the bet setting has not been activated ( all users have bet on the same )
        2. the game has finished and no result has been summited after 3 hours of the end time
       @param env Environment
       @param user Address The address of the user
       @param setting i128 The id of the setting
    */
    fn claim_refund(env: Env, user: Address, setting: i128) -> (i128, i128) {
        user.require_auth();
        let contract_address = env.current_contract_address();
        let usd = storage::get_usd(env.clone());

        let trust: Address = storage::get_trust(env.clone());
        let mut totalBet = 0;
        let betData: Bet = storage::get_bet(env.clone(), user.clone(), setting.clone());
        if betData.id == 0 {
            panic_with_error!(&env, BettingError::NoBetHasBeenFound);
        }
        let mut amountUsd = betData.clone().amount_bet;
        let mut trust_amount = 0;
        let receivedResult: ResultGame =
            storage::get_ResultGame(env.clone(), betData.clone().gameid);
        let (_, startTime, endTime, _) = storage::existBet(env.clone(), betData.clone().gameid);
        //Check if user has claimed before
        let doneBefore = storage::get_didUserWithdraw(env.clone(), user.clone(), setting.clone());
        if doneBefore {
            panic_with_error!(&env, BettingError::UserAlreadyClaimed);
        }
        if startTime < env.ledger().timestamp() as u32 {
            let privateBet: PrivateBet =
                storage::get_PrivateBet(env.clone(), betData.clone().Setting);
            if privateBet.clone().id == 0 {
                panic_with_error!(&env, BettingError::SettingBetDoesNotExist);
            }
            // if the game has started and the setting is not active or the game has started
            // and ended and no result has been summited after 5 hours
            if privateBet.clone().active {
                if endTime + (29 * ONE_HOUR_SECONDS) < env.ledger().timestamp() as u32 {
                    if receivedResult.id != 0 && receivedResult.distribution_executed {
                        panic_with_error!(&env, BettingError::GameIsActive);
                    }
                } else {
                    panic_with_error!(&env, BettingError::GameIsActive);
                }
            }

            Self::moveToken(&env, &usd, &contract_address, &user, &amountUsd);
            if betData.clone().collateralUsd {
                let usd_amount = (betData.clone().amount_bet * TWENTY_PERCENT) / 100;
                amountUsd += usd_amount;
                Self::moveToken(&env, &usd, &contract_address, &user, &usd_amount);
            } else {
                trust_amount = (betData.clone().amount_bet * TEN_PERCENT) / 100;
                Self::moveToken(&env, &trust, &contract_address, &user, &trust_amount);
            }
            storage::add_HonestyPoints(env.clone(), user.clone(), TWENTY_POINTS);
            let points = storage::get_HonestyPoints(env.clone(), user.clone());
            let position: u32 = Self::adduser_board(&env, user.clone(), points);
            BettingEvents::user_honesty_points(&env, user.clone(), points, position);
            storage::set_didUserWithdraw(env.clone(), user.clone(), setting.clone());
        } else {
            panic_with_error!(&env, BettingError::NothingToClaim);
        }
        (amountUsd, trust_amount)
    }
    /*
       @dev This function set a game to be bet on with the admin premission
       @param env Environment
       @param game Game The game data
       @param signature BytesN<64> The signature of the game data
       @param pub_key BytesN<32> The public key of the signer
    */
    fn set_game(env: Env, game: Game, signature: BytesN<64>) -> bool {
        let (exist, startTime, endTime, _) = storage::existBet(env.clone(), game.clone().id);
        if exist {
            panic_with_error!(&env, BettingError::GameHasAlreadySet);
        }
        if game.clone().id == 0
            || game.clone().startTime == 0
            || game.clone().endTime == 0
            || game.clone().startTime >= game.clone().endTime
            || game.active
        {
            panic_with_error!(&env, BettingError::InvalidInputError);
        }
        let encoded = game.clone().to_xdr(&env);
        // Now wrap into Soroban Bytes
        let admin_pubkey = storage::get_admin_pubkey(env.clone());

        env.crypto()
            .ed25519_verify(&admin_pubkey, &encoded, &signature);
        storage::set_game(env.clone(), game.clone());
        BettingEvents::game_set(&env, game.id);
        true
    }
    /*
       @dev This function set a private bet setting for a game
       @param env Environment
       @param user Address The address of the user
       @param privateData PrivateBet The private bet data
       @param game_id i128 The id of the game
    */
    fn set_private_bet(env: Env, user: Address, privateData: PrivateBet, game_id: i128) -> bool {
        user.require_auth();

        let (exist, startTime, endTime, _) = storage::existBet(env.clone(), game_id.clone());
        if !exist {
            panic_with_error!(&env, BettingError::GameDoesNotExist);
        }
        if (privateData.id == 0
            || privateData.gameid != game_id.clone()
            || privateData.amount_bet_min <= 0
            || privateData.users_invated.len() < 2
            || privateData.active
            || privateData.settingAdmin != user.clone())
        {
            panic_with_error!(&env, BettingError::InvalidInputError);
        }

        storage::set_privateSetting(env.clone(), privateData.clone());
        storage::add_privateSettingList(env.clone(), game_id.clone(), privateData.id);
        BettingEvents::private_setting(
            &env,
            game_id,
            privateData.id,
            user,
            privateData.amount_bet_min,
        );
        true
    }
    /*
    @dev This function add a user to a private bet setting
    @param env Environment
    @param setting i128 The id of the setting
    @param game i128 The id of the game
    @param newUser Address The address of the new user to be added
     */
    fn add_user_privateBet(env: Env, setting: i128, game: i128, newUser: Address) -> bool {
        // add fun that allows users to be added in private rooms because of his honesty points
        let mut privateBet: PrivateBet = storage::get_PrivateBet(env.clone(), setting.clone());
        privateBet.settingAdmin.require_auth();
        let (exist, startTime, endTime, _) = storage::existBet(env.clone(), game.clone());
        if !exist {
            panic_with_error!(&env, BettingError::GameDoesNotExist);
        }
        if startTime < env.ledger().timestamp() as u32 {
            panic_with_error!(&env, BettingError::GameHasAlreadyStarted);
        }
        if privateBet.clone().id == 0 || privateBet.clone().gameid != game.clone() {
            panic_with_error!(&env, BettingError::SettingBetDoesNotExist);
        }
        if privateBet.users_invated.contains(&newUser) {
            panic_with_error!(&env, BettingError::InvalidInputError);
        }
        privateBet.users_invated.push_front(newUser.clone());
        storage::set_privateSetting(env.clone(), privateBet.clone());
        BettingEvents::new_user_added_private(&env, game, setting, newUser);
        true
    }
    /*
       @dev This function summit the result of the game, this result can be  summited only by users who bet on the game
       @dev user will have a 5 hours window after the game has finished to summit the result
       @param env Environment
       @param user Address The address of the user
       @param result ResultGame The result of the game
    */
    fn summitResult(env: Env, user: Address, setting: i128, result: ResultGame) -> bool {
        user.require_auth();
        let betResult: Bet = storage::get_Bet(env.clone(), user.clone(), setting.clone());
        let privateBet: PrivateBet = storage::get_PrivateBet(env.clone(), setting.clone());
        if privateBet.clone().id == 0 || !privateBet.active {
            panic_with_error!(&env, BettingError::SettingBetDoesNotExist);
        }
        if betResult.id == 0 {
            panic_with_error!(&env, BettingError::BetNotFound);
        }
        if result.clone().id == 0
            || result.clone().gameid == 0
            || result.clone().setting != setting.clone()
        {
            panic_with_error!(&env, BettingError::InvalidInputError);
        }
        let (exist, startTime, endTime, active) =
            storage::existBet(env.clone(), betResult.clone().gameid);
        let receivedResult: ResultGame = storage::get_ResultGame(env.clone(), setting.clone());
        if receivedResult.id != 0 {
            panic_with_error!(&env, BettingError::GameResultAlreadySet);
        }
        if !exist {
            panic_with_error!(&env, BettingError::GameDoesNotExist);
        }

        if endTime > env.ledger().timestamp() as u32 {
            panic_with_error!(&env, BettingError::GameHasNotFinished);
        }
        if endTime + (5 * ONE_HOUR_SECONDS) < env.ledger().timestamp() as u32 {
            panic_with_error!(&env, BettingError::GameSumitionHasFinished);
        }
        if result.clone().distribution_executed || result.clone().pause {
            panic_with_error!(&env, BettingError::InvalidInputError);
        }
        let mut resultAssessment: ResultAssessment =
            storage::get_ResultAssessment(env.clone(), setting.clone());
        resultAssessment.UsersApprove.push_front(user.clone());
        resultAssessment.id = setting.clone();
        resultAssessment.setting = setting.clone();
        resultAssessment.gameid = betResult.clone().gameid;
        storage::set_ResultAssessment(env.clone(), setting.clone(), resultAssessment.clone());
        storage::add_UsersAmountVoted(env.clone(), setting.clone());
        storage::add_UsersAmountVoted(env.clone(), betResult.clone().gameid);

        if betResult.clone().collateralUsd {
            storage::add_approve_total(
                env.clone(),
                betResult.clone().Setting,
                betResult.clone().amount_bet,
                betResult.clone().bet,
                betResult.clone().collateralUsd,
                betResult.clone().amount_bet * TWENTY_PERCENT / 100,
            );
            storage::delete_not_assesed_yet(
                env.clone(),
                betResult.clone().Setting,
                betResult.clone().amount_bet,
                betResult.clone().bet,
                betResult.clone().collateralUsd,
                betResult.clone().amount_bet * TWENTY_PERCENT / 100,
            );
        } else {
            storage::add_approve_total(
                env.clone(),
                betResult.clone().Setting,
                betResult.clone().amount_bet,
                betResult.clone().bet,
                betResult.clone().collateralUsd,
                betResult.clone().amount_bet * TEN_PERCENT / 100,
            );
            storage::delete_not_assesed_yet(
                env.clone(),
                betResult.clone().Setting,
                betResult.clone().amount_bet,
                betResult.clone().bet,
                betResult.clone().collateralUsd,
                betResult.clone().amount_bet * TEN_PERCENT / 100,
            );
        }

        storage::set_ResultGame(env.clone(), result.clone());
        BettingEvents::game_result(&env, setting.clone(), result.result, result.description);

        true
    }
    /*
       @dev This function assess the result of the game by the users who bet on the game
       @dev user will have a 5 hours window after the game has finished to assess the result
       @param env Environment
       @param user Address The address of the user
       @param bet Bet The bet of the user
       @param desition AssessmentKey The desition of the user (approve or reject)
    */
    fn assessResult(env: Env, user: Address, setting: i128, desition: AssessmentKey) -> bool {
        user.require_auth();
        let betResult: Bet = storage::get_Bet(env.clone(), user.clone(), setting.clone());
        if betResult.id == 0 {
            panic_with_error!(&env, BettingError::BetNotFound);
        }
        let (exist, startTime, endTime, _) =
            storage::existBet(env.clone(), betResult.clone().gameid);
        if !exist {
            panic_with_error!(&env, BettingError::GameDoesNotExist);
        }
        if endTime > env.ledger().timestamp() as u32 {
            panic_with_error!(&env, BettingError::GameHasNotFinished);
        }
        if endTime + (5 * ONE_HOUR_SECONDS) < env.ledger().timestamp() as u32 {
            panic_with_error!(&env, BettingError::GameAssesmentHasFinished);
        }

        let mut results: ResultGame = storage::get_ResultGame(env.clone(), setting.clone());
        if results.id == 0 {
            panic_with_error!(&env, BettingError::GameResultNotFound);
        }
        let mut resultAssessment: ResultAssessment =
            storage::get_ResultAssessment(env.clone(), setting.clone());

        if resultAssessment.UsersApprove.contains(&user)
            || resultAssessment.UsersReject.contains(&user)
        {
            panic_with_error!(&env, BettingError::UserCannotVote);
        }

        if desition == AssessmentKey::approve {
            resultAssessment.UsersApprove.push_front(user.clone());
        } else if desition == AssessmentKey::reject {
            resultAssessment.UsersReject.push_front(user.clone());
            results.pause = true;
        }
        //Check if all the user in the room have voted
        storage::add_UsersAmountVoted(env.clone(), setting.clone());
        if storage::UsersAmount(env.clone(), setting.clone()) {
            BettingEvents::all_voteSetting(&env, setting.clone());
        }

        //Now we check if all the user in the game have voted
        storage::add_UsersAmountVoted(env.clone(), betResult.clone().gameid);
        if storage::UsersAmount(env.clone(), betResult.clone().gameid) {
            BettingEvents::all_voteGame(&env, betResult.clone().gameid);
        }

        storage::set_ResultAssessment(env.clone(), setting.clone(), resultAssessment.clone());
        if betResult.clone().collateralUsd {
            if results.pause {
                BettingEvents::game_result_reject(&env, setting.clone());
                storage::puase_ResultGame(env.clone(), setting.clone(), results.clone().pause);
                storage::add_reject_total(
                    env.clone(),
                    betResult.clone().Setting,
                    betResult.clone().amount_bet,
                    betResult.clone().bet,
                    betResult.clone().collateralUsd,
                    betResult.clone().amount_bet * TWENTY_PERCENT / 100,
                );
            } else {
                storage::add_approve_total(
                    env.clone(),
                    betResult.clone().Setting,
                    betResult.clone().amount_bet,
                    betResult.clone().bet,
                    betResult.clone().collateralUsd,
                    betResult.clone().amount_bet * TWENTY_PERCENT / 100,
                );
            }
            storage::delete_not_assesed_yet(
                env.clone(),
                betResult.clone().Setting,
                betResult.clone().amount_bet,
                betResult.clone().bet,
                betResult.clone().collateralUsd,
                betResult.clone().amount_bet * TWENTY_PERCENT / 100,
            );
            true
        } else {
            if results.pause {
                BettingEvents::game_result_reject(&env, setting.clone());
                storage::puase_ResultGame(env.clone(), setting.clone(), results.clone().pause);
                storage::add_reject_total(
                    env.clone(),
                    betResult.clone().Setting,
                    betResult.clone().amount_bet,
                    betResult.clone().bet,
                    betResult.clone().collateralUsd,
                    betResult.clone().amount_bet * TEN_PERCENT / 100,
                );
            } else {
                storage::add_approve_total(
                    env.clone(),
                    betResult.clone().Setting,
                    betResult.clone().amount_bet,
                    betResult.clone().bet,
                    betResult.clone().collateralUsd,
                    betResult.clone().amount_bet * TEN_PERCENT / 100,
                );
            }

            storage::delete_not_assesed_yet(
                env.clone(),
                betResult.clone().Setting,
                betResult.clone().amount_bet,
                betResult.clone().bet,
                betResult.clone().collateralUsd,
                betResult.clone().amount_bet * TEN_PERCENT / 100,
            );
            true
        }
    }
    /*
       @dev This function claim the money won or the money staked for the summiter and the protocol
       @param env Environment
       @param user Address The address of the user
       @param typeClaim ClaimType The type of claim (summiter, protocol, user)
       @param setting i128 The id of the setting (only for user claim)
    */
    fn claim(env: Env, user: Address, typeClaim: ClaimType, setting: i128) -> (i128, i128) {
        user.require_auth();
        let contract_address = env.current_contract_address();
        let adminAdr: Address = storage::get_admin(env.clone());
        let usd = storage::get_usd(env.clone());
        let trust: Address = storage::get_trust(env.clone());
        let mut amountWithdrew: (i128, i128) = (0, 0);
        match typeClaim {
            ClaimType::Supreme => {
                let doneBefore =
                    storage::get_didUserWithdrawSupreme(env.clone(), user.clone(), setting.clone());
                if doneBefore {
                    panic_with_error!(&env, BettingError::UserAlreadyClaimed);
                }
                let mut assestment =
                    storage::get_ResultAssessmentSupreme(env.clone(), setting.clone());
                if assestment.id == 0 {
                    panic_with_error!(&env, BettingError::NothingToClaim);
                }
                let mut countVotes: CountVotesResult =
                    storage::get_CountVoteSupreme(env.clone(), setting.clone());

                let supreme = storage::get_supreme(env.clone());
                if user != supreme && !assestment.countHonestyUsers.contains(&user) {
                    panic_with_error!(&env, BettingError::NothingToClaim);
                } else {
                    if user == supreme {
                        if !assestment.ExternalUser {
                            panic_with_error!(&env, BettingError::NothingToClaim);
                        }
                    }
                }

                let mut numberVoter = 0;
                if assestment.Admin {
                    numberVoter = countVotes.aproved - 1;
                } else {
                    numberVoter = countVotes.aproved;
                }
                let money: i128 = storage::get_ClaimSupreme(env.clone(), setting.clone());
                let amoutgot = money / numberVoter;
                Self::moveToken(&env, &usd, &contract_address, &user, &amoutgot);
                amountWithdrew = (amoutgot, 0);
                storage::set_didUserWithdrawSupreme(env.clone(), user.clone(), setting.clone());
            }
            ClaimType::Protocol => {
                adminAdr.require_auth();
                let money: i128 = storage::get_ClaimProtocol(env.clone());
                let trustAmount: i128 = storage::get_ClaimProtocolTrust(env.clone());
                Self::moveToken(&env, &usd, &contract_address, &adminAdr, &money);
                Self::moveToken(&env, &trust, &contract_address, &adminAdr, &trustAmount);
                amountWithdrew = (money, trustAmount);
                storage::zero_ClaimProtocol(env.clone());
            }
            ClaimType::User => {
                let doneBefore =
                    storage::get_didUserWithdraw(env.clone(), user.clone(), setting.clone());
                if doneBefore {
                    panic_with_error!(&env, BettingError::UserAlreadyClaimed);
                }
                let (kindofUser, amountBet, usdCollateral) =
                    Self::what_kind_user(env.clone(), user.clone(), setting.clone());
                let winner_pool = storage::get_winnerPool(env.clone(), setting.clone());
                let loser_pool = storage::get_loserPool(env.clone(), setting.clone());
                let amount_share = storage::get_pool_total(env.clone(), setting.clone());
                match kindofUser {
                    6 => {
                        // NO BET
                        panic_with_error!(&env, BettingError::NoBetHasBeenFound);
                    }
                    5 => {
                        // loser who didn't assess the result
                        panic_with_error!(&env, BettingError::NothingToClaim);
                    }
                    4 => {
                        // winner who didn't assess the result
                        // user gets 50% of his bet back and no trust back
                        let bet_50 = (amountBet * 50) / 100;
                        Self::moveToken(&env, &usd, &contract_address, &user, &bet_50);

                        storage::set_didUserWithdraw(env.clone(), user.clone(), setting.clone());
                        amountWithdrew = (bet_50, 0);
                    }
                    3 => {
                        // dishonest user
                        // no money no trust back
                        panic_with_error!(&env, BettingError::NothingToClaim);
                    }
                    2 => {
                        // loser honest
                        // user gets back trust tokens
                        let mut trust_amount = 0;
                        let mut usdwithdraw = 0;
                        if usdCollateral {
                            let usd_amount = (amountBet * TWENTY_PERCENT) / 100;
                            Self::moveToken(&env, &usd, &contract_address, &user, &usd_amount);
                            usdwithdraw = usd_amount;
                        } else {
                            trust_amount = (amountBet * TEN_PERCENT) / 100;
                            Self::moveToken(&env, &trust, &contract_address, &user, &trust_amount);
                            trust_amount = trust_amount;
                        }
                        if winner_pool == 0 {
                            let user_share = (amountBet * 100) / loser_pool;
                            let user_amount = (user_share * amount_share) / 100;
                            Self::moveToken(&env, &usd, &contract_address, &user, &user_amount);
                            usdwithdraw += user_amount;
                        }
                        if storage::is_summiter(env.clone(), setting.clone(), user.clone()) {
                            storage::add_HonestyPoints(env.clone(), user.clone(), 80);
                        } else {
                            storage::add_HonestyPoints(env.clone(), user.clone(), FIFTY_POINTS);
                        }
                        amountWithdrew = (usdwithdraw, trust_amount);

                        let points = storage::get_HonestyPoints(env.clone(), user.clone());
                        let position: u32 = Self::adduser_board(&env, user.clone(), points);
                        BettingEvents::user_honesty_points(&env, user.clone(), points, position);
                        storage::set_didUserWithdraw(env.clone(), user.clone(), setting.clone());
                    }
                    1 => {
                        // winner honest
                        let user_share = (amountBet * 100) / winner_pool;
                        let user_amount = (user_share * amount_share) / 100;
                        let total = amountBet + user_amount;
                        let mut trust_amount = 0;
                        let mut usdWithdraw = 0;

                        Self::moveToken(&env, &usd, &contract_address, &user, &total);
                        usdWithdraw += total;
                        if usdCollateral {
                            let usd_amount = (amountBet * TWENTY_PERCENT) / 100;
                            Self::moveToken(&env, &usd, &contract_address, &user, &usd_amount);
                            usdWithdraw += usd_amount;
                        } else {
                            trust_amount = (amountBet * TEN_PERCENT) / 100;
                            Self::moveToken(&env, &trust, &contract_address, &user, &trust_amount);
                            trust_amount = trust_amount;
                        }
                        storage::add_HonestyPoints(env.clone(), user.clone(), FIFTY_POINTS);
                        if storage::is_summiter(env.clone(), setting.clone(), user.clone()) {
                            storage::add_HonestyPoints(env.clone(), user.clone(), 80);
                        } else {
                            storage::add_HonestyPoints(env.clone(), user.clone(), FIFTY_POINTS);
                        }
                        let points = storage::get_HonestyPoints(env.clone(), user.clone());
                        let position: u32 = Self::adduser_board(&env, user.clone(), points);
                        BettingEvents::user_honesty_points(&env, user.clone(), points, position);
                        storage::set_didUserWithdraw(env.clone(), user.clone(), setting.clone());
                        amountWithdrew = (usdWithdraw, trust_amount);
                    }
                    _ => {
                        panic_with_error!(&env, BettingError::InvalidInputError);
                    }
                }
            }
            _ => {
                // default case
                panic_with_error!(&env, BettingError::InvalidInputError);
            }
        }
        amountWithdrew
    }
    /*
       @dev This function set the result of the game when a complain has been made and the time to summit the result has passed
       @param env Environment
       @param result ResultGame The result of the game
       The supreme court will be composed by a trusted external user, admin , 2 honest users( with highest honesty points)
       and the community overall result
    */
    fn setResult_supremCourt(env: Env, user: Address, result: ResultGame) -> bool {
        user.require_auth();
        let supreme = storage::get_supreme(env.clone());
        let admin: Address = storage::get_admin(env.clone());
        let user_honesty: Vec<Address> = storage::get_supreme_honesty(env.clone());
        //check ....
        let (exist, startTime, endTime, _) = storage::existBet(env.clone(), result.clone().gameid);
        if !exist {
            panic_with_error!(&env, BettingError::GameDoesNotExist);
        }
        if endTime + (5 * ONE_HOUR_SECONDS) > env.ledger().timestamp() as u32 {
            if !storage::UsersAmount(env.clone(), result.clone().gameid) {
                panic_with_error!(&env, BettingError::GameAssesmentHasFinished);
            }
        }
        if endTime + (29 * ONE_HOUR_SECONDS) < env.ledger().timestamp() as u32 {
            panic_with_error!(&env, BettingError::GameAssesmentHasFinished);
        }
        let mut assestment =
            storage::get_ResultAssessmentSupreme(env.clone(), result.clone().gameid);

        assestment.id = result.clone().gameid;
        assestment.gameid = result.clone().gameid;

        let mut countVotes: CountVotesResult =
            storage::get_CountVoteSupreme(env.clone(), result.clone().gameid);

        countVotes.id = result.clone().gameid;
        countVotes.gameid = result.clone().gameid;

        if user == supreme {
            assestment.ExternalUser = true;
        } else if user == admin {
            assestment.Admin = true;
        } else if user_honesty.contains(&user) {
            assestment.countHonestyUsers.push_front(user.clone());
        } else {
            panic_with_error!(&env, BettingError::UnauthorizedError);
        }
        countVotes.aproved += 1;
        storage::set_ResultAssessmentSupreme(
            env.clone(),
            result.clone().gameid,
            assestment.clone(),
        );
        storage::set_CountVoteSupreme(env.clone(), result.clone().gameid, countVotes.clone());
        if storage::get_ResultGameSupreme(env.clone(), result.clone().gameid).id != 0 {
            panic_with_error!(&env, BettingError::GameResultAlreadySet);
        }
        // Check if there are pending summitions
        let mut is_sumition_needed = false;
        let listedPrivateBet: Vec<(i128)> =
            storage::get_privateSettingList(env.clone(), result.clone().gameid);

        if listedPrivateBet.len() != 0 {
            for setting in listedPrivateBet.iter() {
                let xresult: ResultGame = storage::get_ResultGame(env.clone(), setting.clone());
                if xresult.clone().distribution_executed {
                    continue;
                }
                if xresult.pause == true {
                    is_sumition_needed = true;
                    break;
                }
                if xresult.id == 0 {
                    is_sumition_needed = true;
                    break;
                }
            }
        }
        if is_sumition_needed == false {
            panic_with_error!(&env, BettingError::NotAllowToSummitResult);
        }
        if result.clone().distribution_executed
            || result.clone().pause
            || result.clone().id == 0
            || result.clone().gameid == 0
        {
            panic_with_error!(&env, BettingError::InvalidInputError);
        } else {
            storage::set_ResultGameSupreme(env.clone(), result.clone());
            true
        }
    }
    /*
       @dev This function allow the supreme court to assess the result summited and make the final desition
       @param env Environment
       @param result ResultGame The result of the game
       This supreme court will have a 24 hours window after the 3 hours window for summiting
       the result by the users has passed, the result can be submited only by the supreme court members
       if the result is approved the distribution will be made according to the summited result
       if the result is rejected the result summitted willbe removed, so the users will have to summit again the result
    */
    fn AssestResult_supremCourt(
        env: Env,
        user: Address,
        gameid: i128,
        desition: AssessmentKey,
    ) -> bool {
        user.require_auth();
        // 0 correct
        // 1 incorrect.
        let mut complain = 0;
        let (exist, startTime, endTime, _) = storage::existBet(env.clone(), gameid.clone());
        if !exist {
            panic_with_error!(&env, BettingError::GameDoesNotExist);
        }
        //check ....
        if endTime + (5 * ONE_HOUR_SECONDS) > env.ledger().timestamp() as u32 {
            if !storage::UsersAmount(env.clone(), gameid.clone()) {
                panic_with_error!(&env, BettingError::GameAssesmentHasFinished);
            }
        }
        if endTime + (29 * ONE_HOUR_SECONDS) < env.ledger().timestamp() as u32 {
            panic_with_error!(&env, BettingError::GameAssesmentHasFinished);
        }
        let mut is_sumition_needed = false;
        let listedPrivateBet: Vec<(i128)> =
            storage::get_privateSettingList(env.clone(), gameid.clone());

        if listedPrivateBet.len() != 0 {
            for setting in listedPrivateBet.iter() {
                let xresult: ResultGame = storage::get_ResultGame(env.clone(), setting.clone());
                if xresult.clone().distribution_executed {
                    continue;
                }
                if xresult.pause == true {
                    is_sumition_needed = true;
                    break;
                }
                if xresult.id == 0 {
                    is_sumition_needed = true;
                    break;
                }
            }
        }
        if is_sumition_needed == false {
            panic_with_error!(&env, BettingError::GameResultAlreadyExecuted);
        }
        let result: ResultGame = storage::get_ResultGameSupreme(env.clone(), gameid.clone());
        if result.id == 0 {
            panic_with_error!(&env, BettingError::GameResultNotFound);
        }
        // 0 means no clear result
        // 1 approve
        // 2 reject
        let mut community_assessment: i128 = 0;
        match storage::overall_result(env.clone(), gameid.clone()) {
            0 => {
                community_assessment = 0;
            }
            1 => {
                if result.result == BetKey::Team_local {
                    community_assessment = 1;
                } else {
                    community_assessment = 2;
                }
            }
            2 => {
                if result.result == BetKey::Team_away {
                    community_assessment = 1;
                } else {
                    community_assessment = 2;
                }
            }
            3 => {
                if result.result == BetKey::Cancel {
                    community_assessment = 1;
                } else {
                    community_assessment = 2;
                }
            }
            4 => {
                if result.result == BetKey::Tie {
                    community_assessment = 1;
                } else {
                    community_assessment = 2;
                }
            }
            _ => {}
        }
        let supreme = storage::get_supreme(env.clone());
        let admin: Address = storage::get_admin(env.clone());
        let user_honesty: Vec<Address> = storage::get_supreme_honesty(env.clone());
        let mut assestment = storage::get_ResultAssessmentSupreme(env.clone(), gameid.clone());

        let mut countVotes: CountVotesResult =
            storage::get_CountVoteSupreme(env.clone(), gameid.clone());

        if user == supreme {
            if assestment.ExternalUser {
                panic_with_error!(&env, BettingError::UserhasAlreadyVoted);
            }
            assestment.ExternalUser = true;
        } else if user == admin {
            if assestment.Admin {
                panic_with_error!(&env, BettingError::UserhasAlreadyVoted);
            }
            assestment.Admin = true;
        } else if user_honesty.contains(&user) {
            if assestment.countHonestyUsers.contains(&user)
                || assestment.countHonestyUsers.len() >= 2
            {
                panic_with_error!(&env, BettingError::UserhasAlreadyVoted);
            }
            assestment.countHonestyUsers.push_front(user.clone());
        } else {
            panic_with_error!(&env, BettingError::UnauthorizedError);
        }
        storage::set_ResultAssessmentSupreme(env.clone(), gameid.clone(), assestment.clone());
        match desition {
            AssessmentKey::approve => {
                countVotes.aproved += 1;
            }
            AssessmentKey::reject => {
                countVotes.rejected += 1;
            }
        }
        storage::set_CountVoteSupreme(env.clone(), gameid.clone(), countVotes.clone());
        if countVotes.aproved > 2 || (countVotes.aproved == 2 && community_assessment == 1) {
            let listedPrivateBet: Vec<(i128)> =
                storage::get_privateSettingList(env.clone(), result.clone().gameid);

            if listedPrivateBet.len() != 0 {
                for setting in listedPrivateBet.iter() {
                    let xresult: ResultGame = storage::get_ResultGame(env.clone(), setting.clone());
                    if xresult.clone().distribution_executed {
                        continue;
                    }
                    if xresult.id != 0 {
                        if xresult.pause != true {
                            continue;
                        }
                    }

                    if xresult.result != result.result {
                        complain = 0; // The complain made by the users was correct
                    } else {
                        complain = 1; // The complain made by the users was incorrect
                    }
                    let privateBet: PrivateBet =
                        storage::get_PrivateBet(env.clone(), setting.clone());
                    if privateBet.active == false {
                        continue;
                    }

                    if result.result == BetKey::Cancel {
                        storage::active_private_setting(env.clone(), setting.clone(), false);
                    } else {
                        Self::make_distribution(
                            env.clone(),
                            privateBet.clone().gameid,
                            setting.clone(),
                            result.clone().result,
                            complain,
                        );
                    }
                }
            }
            BettingEvents::game_result_supreme(&env, result.gameid, result.result);
            true
        } else if countVotes.rejected > 2 || (countVotes.rejected == 2 && community_assessment == 2)
        {
            // It means the supreme court has rejected the result so this is deleted
            let result_empty: ResultGame = ResultGame {
                id: 0,
                gameid: gameid,
                setting: 0,
                result: BetKey::Cancel,
                description: String::from_slice(&env, ""),
                distribution_executed: false,
                pause: false,
            };
            storage::set_ResultGameSupreme(env.clone(), result_empty.clone());
            let assessment_empty: ResultAssessmentSupreme = ResultAssessmentSupreme {
                id: 0,
                gameid: 0,
                Admin: false,
                ExternalUser: false,
                countHonestyUsers: Vec::new(&env),
            };
            storage::set_ResultAssessmentSupreme(
                env.clone(),
                gameid.clone(),
                assessment_empty.clone(),
            );
            let votes_empty: CountVotesResult = CountVotesResult {
                id: 0,
                gameid: 0,
                aproved: 0,
                rejected: 0,
            };
            storage::set_CountVoteSupreme(env.clone(), gameid.clone(), votes_empty.clone());
            BettingEvents::game_resultbysupremeremoved(&env, result.gameid, result.result);
            true
        } else {
            // still waiting for more votes
            false
        }
    }
    /*
       @dev This function execute the distribution of the pools according to the rules, fines and betting
       @param env Environment
       @param game_id i128 The id of the game
    */
    fn execute_distribution(env: Env, setting: i128) -> bool {
        let complain = 2; // 2 means no complain was made
        let result: ResultGame = storage::get_ResultGame(env.clone(), setting.clone());
        let (exist, startTime, endTime, _) = storage::existBet(env.clone(), result.clone().gameid);
        if endTime + (5 * ONE_HOUR_SECONDS) > env.ledger().timestamp() as u32 {
            if !storage::UsersAmount(env.clone(), setting.clone()) {
                panic_with_error!(&env, BettingError::GameAssesmentHasFinished);
            }
        }
        let privateBet: PrivateBet = storage::get_PrivateBet(env.clone(), setting.clone());
        if privateBet.active == false {
            panic_with_error!(&env, BettingError::SettingNotActive);
        }
        if result.id == 0 {
            panic_with_error!(&env, BettingError::GameNoResult);
        }
        if result.pause == true {
            panic_with_error!(&env, BettingError::GameHasBeenPaused);
        }
        if result.clone().distribution_executed {
            panic_with_error!(&env, BettingError::GameHasAlreadyBeenExecuted);
        }

        if result.result == BetKey::Cancel {
            storage::active_private_setting(env.clone(), setting.clone(), false);
        } else {
            Self::make_distribution(
                env.clone(),
                privateBet.clone().gameid,
                setting.clone(),
                result.clone().result,
                complain,
            );
        }
        storage::set_result_team_count(env.clone(), result.clone().gameid, result.clone().result);

        true
    }
}

impl BettingContract {
    /*
       @dev This  funtion make the distribution of the pools according to the rules, fines and betting
       @param env Environment
       @param game_id i128 The id of the game
       @param setting i128 The id of the setting
       @param resultBet BetKey The result of the game
       @param complain i128 The complain made by the users
       Complain 0 = The complain made by the users was correct
       Complain 1 = The complain made by the users was incorrect
       Complain 2 = No complain was made
    */
    fn make_distribution(
        env: Env,
        game_id: i128,
        setting: i128,
        resultBet: BetKey,
        complain: i128,
    ) {
        let mut amount_gain_pool: i128 = 0;
        let mut trust_collateral_taken_pool: i128 = 0;
        let mut usd_collateral_taken_pool: i128 = 0;
        let mut losers_honest_pool: i128 = 0;
        let mut winner_pool: i128 = 0;
        let mut novote_winner: i128 = 0;
        let admin = storage::get_admin(env.clone());
        let mut result = ResultGame {
            id: setting.clone(),
            gameid: game_id.clone(),
            setting: setting.clone(),
            result: resultBet.clone(),
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };
        let mut resultAssessment: ResultAssessment =
            storage::get_ResultAssessment(env.clone(), game_id.clone());

        let (exist, startTime, endTime, _) = storage::existBet(env.clone(), game_id.clone());
        if !exist {
            panic_with_error!(&env, BettingError::GameDoesNotExist);
        }
        if endTime > env.ledger().timestamp() as u32 {
            panic_with_error!(&env, BettingError::GameHasNotFinished);
        }
        for i in 0..=2 {
            let mut bet_key: BetKey = BetKey::Team_local;
            match i {
                0 => {
                    bet_key = BetKey::Team_local;
                }
                1 => {
                    bet_key = BetKey::Tie;
                }
                2 => {
                    bet_key = BetKey::Team_away;
                }
                _ => {}
            }
            let amountBet =
                storage::get_not_assesed_yet(env.clone(), setting.clone(), bet_key.clone());
            let amountCollateralUsd = storage::get_not_usdCollateral_assesed_yet(
                env.clone(),
                setting.clone(),
                bet_key.clone(),
            );
            let amountCollateralTrust = storage::get_not_TrustCollateral_assesed_yet(
                env.clone(),
                setting.clone(),
                bet_key.clone(),
            );
            trust_collateral_taken_pool += amountCollateralTrust;
            usd_collateral_taken_pool += amountCollateralUsd;
            if resultBet != bet_key {
                amount_gain_pool += amountBet;
            } else {
                novote_winner += amountBet;
            }
        }
        let winner_minus_50 = (novote_winner * 50) / 100;
        amount_gain_pool += winner_minus_50;
        match complain {
            0 => {
                let dishonestTeamLocal =
                    storage::get_approve_total(env.clone(), setting.clone(), BetKey::Team_local);
                let collateralUsdTeamLocal = storage::get_CollateralUsd_approve_total(
                    env.clone(),
                    setting.clone(),
                    BetKey::Team_local,
                );
                let collateralTrustTeamLocal = storage::get_CollateralTrust_approve_total(
                    env.clone(),
                    setting.clone(),
                    BetKey::Team_local,
                );
                let dishonestDraw =
                    storage::get_approve_total(env.clone(), setting.clone(), BetKey::Tie);
                let collateralUsdDraw = storage::get_CollateralUsd_approve_total(
                    env.clone(),
                    setting.clone(),
                    BetKey::Tie,
                );
                let collateralTrustDraw = storage::get_CollateralTrust_approve_total(
                    env.clone(),
                    setting.clone(),
                    BetKey::Tie,
                );
                let dishonestTeamAway =
                    storage::get_approve_total(env.clone(), setting.clone(), BetKey::Team_away);
                let collateralUsdTeamAway = storage::get_CollateralUsd_approve_total(
                    env.clone(),
                    setting.clone(),
                    BetKey::Team_away,
                );
                let collateralTrustTeamAway = storage::get_CollateralTrust_approve_total(
                    env.clone(),
                    setting.clone(),
                    BetKey::Team_away,
                );
                amount_gain_pool += dishonestTeamAway;
                amount_gain_pool += dishonestTeamLocal;
                amount_gain_pool += dishonestDraw;
                trust_collateral_taken_pool += collateralTrustTeamAway;
                trust_collateral_taken_pool += collateralTrustTeamLocal;
                trust_collateral_taken_pool += collateralTrustDraw;
                usd_collateral_taken_pool += collateralUsdTeamAway;
                usd_collateral_taken_pool += collateralUsdTeamLocal;
                usd_collateral_taken_pool += collateralUsdDraw;

                for i in 0..=2 {
                    let mut bet_key: BetKey = BetKey::Team_local;
                    match i {
                        0 => {
                            bet_key = BetKey::Team_local;
                        }
                        1 => {
                            bet_key = BetKey::Tie;
                        }
                        2 => {
                            bet_key = BetKey::Team_away;
                        }
                        _ => {}
                    }
                    let amountBet =
                        storage::get_reject_total(env.clone(), setting.clone(), bet_key.clone());
                    if resultBet != bet_key {
                        amount_gain_pool += amountBet;
                        losers_honest_pool += amountBet;
                    } else {
                        winner_pool += amountBet;
                    }
                }
            }
            1 => {
                let dishonestTeamLocal =
                    storage::get_reject_total(env.clone(), setting.clone(), BetKey::Team_local);
                let collateralUsdTeamLocal = storage::get_CollateralUsd_reject_total(
                    env.clone(),
                    setting.clone(),
                    BetKey::Team_local,
                );
                let collateralTrustTeamLocal = storage::get_CollateralTrust_reject_total(
                    env.clone(),
                    setting.clone(),
                    BetKey::Team_local,
                );
                let dishonestDraw =
                    storage::get_reject_total(env.clone(), setting.clone(), BetKey::Tie);
                let collateralUsdDraw = storage::get_CollateralUsd_reject_total(
                    env.clone(),
                    setting.clone(),
                    BetKey::Tie,
                );
                let collateralTrustDraw = storage::get_CollateralTrust_reject_total(
                    env.clone(),
                    setting.clone(),
                    BetKey::Tie,
                );
                let dishonestTeamAway =
                    storage::get_reject_total(env.clone(), setting.clone(), BetKey::Team_away);
                let collateralUsdTeamAway = storage::get_CollateralUsd_reject_total(
                    env.clone(),
                    setting.clone(),
                    BetKey::Team_away,
                );
                let collateralTrustTeamAway = storage::get_CollateralTrust_reject_total(
                    env.clone(),
                    setting.clone(),
                    BetKey::Team_away,
                );
                amount_gain_pool += dishonestTeamAway;
                amount_gain_pool += dishonestTeamLocal;
                amount_gain_pool += dishonestDraw;
                trust_collateral_taken_pool += collateralTrustTeamAway;
                trust_collateral_taken_pool += collateralTrustTeamLocal;
                trust_collateral_taken_pool += collateralTrustDraw;
                usd_collateral_taken_pool += collateralUsdTeamAway;
                usd_collateral_taken_pool += collateralUsdTeamLocal;
                usd_collateral_taken_pool += collateralUsdDraw;
                for i in 0..=2 {
                    let mut bet_key: BetKey = BetKey::Team_local;
                    match i {
                        0 => {
                            bet_key = BetKey::Team_local;
                        }
                        1 => {
                            bet_key = BetKey::Tie;
                        }
                        2 => {
                            bet_key = BetKey::Team_away;
                        }
                        _ => {}
                    }
                    let amountBet =
                        storage::get_approve_total(env.clone(), setting.clone(), bet_key.clone());
                    if resultBet != bet_key {
                        amount_gain_pool += amountBet;
                        losers_honest_pool += amountBet;
                    } else {
                        winner_pool += amountBet;
                    }
                }
            }
            2 => {
                for i in 0..=2 {
                    let mut bet_key: BetKey = BetKey::Team_local;
                    match i {
                        0 => {
                            bet_key = BetKey::Team_local;
                        }
                        1 => {
                            bet_key = BetKey::Tie;
                        }
                        2 => {
                            bet_key = BetKey::Team_away;
                        }
                        _ => {}
                    }
                    let amountBet =
                        storage::get_approve_total(env.clone(), setting.clone(), bet_key.clone());
                    if resultBet != bet_key {
                        amount_gain_pool += amountBet;
                        losers_honest_pool += amountBet;
                    } else {
                        winner_pool += amountBet;
                    }
                }
            }
            _ => {
                panic_with_error!(&env, BettingError::InvalidInputError);
            }
        }
        amount_gain_pool += usd_collateral_taken_pool;
        let mut supremeCourtReward = 0;

        let mut protocol_retribution = (amount_gain_pool * FIVE_PERCENT) / 100;
        if winner_pool == 0 && losers_honest_pool == 0 {
            //No winner and all users were dishonest
            protocol_retribution = (amount_gain_pool * 100) / 100;
        }
        if complain == 0 || complain == 1 {
            supremeCourtReward = (amount_gain_pool * THREE_PERCENT) / 100;
        }
        amount_gain_pool -= supremeCourtReward;
        amount_gain_pool -= protocol_retribution;
        storage::add_ClaimSupreme(env.clone(), game_id, supremeCourtReward);
        BettingEvents::supreme_reward(&env, game_id, supremeCourtReward);
        storage::add_ClaimProtocol(env.clone(), protocol_retribution);

        storage::add_ClaimProtocolTrust(env.clone(), trust_collateral_taken_pool);

        storage::save_complain(env.clone(), setting.clone(), complain);

        storage::save_winnerPool(env.clone(), setting.clone(), winner_pool);
        storage::save_loserPool(env.clone(), setting.clone(), losers_honest_pool);
        storage::set_pool_total(env.clone(), setting.clone(), amount_gain_pool);
        storage::distribution_ResultGame(env.clone(), setting.clone());
        result.distribution_executed = true;
        storage::set_ResultGame(env.clone(), result.clone());
        BettingEvents::game_setting_distributed(&env, setting);
    }

    /*
       @dev Function to determine the kind of user based on their bet and assessment
       @param env The contract environment
       @param user The address of the user
       @param setting The setting ID of the bet
       @return A tuple containing the kind of user (as an integer) and the amount bet (if applicable)
       1: winner and honest
       2: loser and honest
       3: user dishonest
       4: winner who didn't assess the result
       5: loser who didn't assess the result
       6: no bet
    */
    fn what_kind_user(env: Env, user: Address, setting: i128) -> (i32, i128, bool) {
        let betData: Bet = storage::get_bet(env.clone(), user.clone(), setting.clone());

        if betData.id == 0 {
            return (6, 0, betData.clone().collateralUsd); // no bet
        }
        let (exist, startTime, endTime, _) = storage::existBet(env.clone(), betData.clone().gameid);
        if !exist {
            panic_with_error!(&env, BettingError::GameDoesNotExist);
        }
        let xresult: ResultGame = storage::get_ResultGame(env.clone(), setting.clone());
        if xresult.id == 0 {
            panic_with_error!(&env, BettingError::GameNoResult);
        }
        if xresult.pause == true {
            panic_with_error!(&env, BettingError::GameHasBeenPaused);
        }
        let resultAssessment: ResultAssessment =
            storage::get_ResultAssessment(env.clone(), setting.clone());
        let complain = storage::get_complain(env.clone(), setting.clone());
        match complain {
            0 => {
                if resultAssessment.UsersApprove.contains(&user) {
                    return (3, 0, betData.clone().collateralUsd); // user dishonest
                }
                if resultAssessment.UsersReject.contains(&user) {
                    if betData.bet == xresult.result {
                        return (1, betData.clone().amount_bet, betData.clone().collateralUsd);
                    // winner and honest
                    } else {
                        return (2, betData.clone().amount_bet, betData.clone().collateralUsd);
                        // loser and honest
                    }
                }
                if !resultAssessment.UsersApprove.contains(&user)
                    && !resultAssessment.UsersReject.contains(&user)
                {
                    if betData.bet == xresult.result {
                        return (4, betData.clone().amount_bet, betData.clone().collateralUsd);
                    // winner and honest
                    } else {
                        return (5, 0, betData.clone().collateralUsd); // loser and honest
                    }
                } else {
                    panic_with_error!(&env, BettingError::InvalidInputError);
                }
            }
            1 => {
                if resultAssessment.UsersApprove.contains(&user) {
                    if betData.bet == xresult.result {
                        return (1, betData.clone().amount_bet, betData.clone().collateralUsd);
                    // winner and honest
                    } else {
                        return (2, betData.clone().amount_bet, betData.clone().collateralUsd);
                        // loser and honest
                    }
                }
                if resultAssessment.UsersReject.contains(&user) {
                    return (3, 0, betData.clone().collateralUsd); // user dishonest
                }
                if !resultAssessment.UsersApprove.contains(&user)
                    && !resultAssessment.UsersReject.contains(&user)
                {
                    if betData.bet == xresult.result {
                        return (4, betData.clone().amount_bet, betData.clone().collateralUsd);
                    // winner ?
                    } else {
                        return (5, 0, betData.clone().collateralUsd); // loser ?
                    }
                } else {
                    panic_with_error!(&env, BettingError::InvalidInputError);
                }
            }
            2 => {
                if resultAssessment.UsersApprove.contains(&user) {
                    if betData.bet == xresult.result {
                        return (1, betData.clone().amount_bet, betData.clone().collateralUsd);
                    // winner and honest
                    } else {
                        return (2, betData.clone().amount_bet, betData.clone().collateralUsd);
                        // loser and honest
                    }
                }
                if !resultAssessment.UsersApprove.contains(&user)
                    && !resultAssessment.UsersReject.contains(&user)
                {
                    if betData.bet == xresult.result {
                        return (4, betData.clone().amount_bet, betData.clone().collateralUsd);
                    // winner ?
                    } else {
                        return (5, 0, betData.clone().collateralUsd); // loser ?
                    }
                } else {
                    panic_with_error!(&env, BettingError::InvalidInputError);
                }
            }
            _ => {
                panic_with_error!(&env, BettingError::InvalidInputError);
            }
        }
    }

    /*
    @dev Function to move tokens from one address to another
    @param env The contract environment
    @param token The address of the token contract
    @param from The address to move tokens from
    @param to The address to move tokens to
    @param amount The amount of tokens to move
     */
    fn moveToken(env: &Env, token: &Address, from: &Address, to: &Address, amount: &i128) {
        let token = token::Client::new(env, token);
        token.transfer(from, to, amount);
    }
    /*
        @dev This function add a user to the leaderboard or update his honesty points
        and reorders the leaderboard accordingly returning the new position of the user
        @param env Environment
        @param user Address The address of the user
        @param stakeAmount i128 The amount staked by the user
    */
    fn adduser_board(env: &Env, user: Address, points: i128) -> u32 {
        /*We nee to set a amount to request for the summiter rol */
        let mut honest_top: Vec<(Address)> = Vec::new(&env);
        // ✅ Weighted score calculation
        let new_score = points;

        // ✅ Leaderboard vector
        let mut leaderboard: Vec<(Address, i128)> = storage::get_leaderboard(env.clone());

        // Remove old entry for this user
        let mut i = 0;
        while i < leaderboard.len() {
            let (addr, _) = leaderboard.get(i).unwrap();
            if addr == user {
                leaderboard.remove(i);
                break;
            }
            i += 1;
        }

        // Find position to insert in descending order
        let mut insert_index = leaderboard.len();
        if leaderboard.len() == 0 {
            insert_index = 0;
        } else {
            for idx in 0..leaderboard.len() {
                let (_, score) = leaderboard.get(idx).unwrap();
                if new_score > score || new_score == score {
                    insert_index = idx;
                    break;
                }
            }
        }

        // Insert at correct position
        leaderboard.insert(insert_index, (user.clone(), new_score));

        // Selected the top 2 users
        if leaderboard.len() < 2 {
            let (user1, points) = leaderboard.get(0).unwrap();
            honest_top.push_back(user1.clone());
        } else {
            for j in 0..2 {
                let (user1, points) = leaderboard.get(j).unwrap();
                honest_top.push_back(user1.clone());
            }
        }
        storage::save_supreme_honesty(env.clone(), honest_top);

        // Save leaderboard
        storage::set_leaderboard(env.clone(), leaderboard);
        insert_index
    }
}

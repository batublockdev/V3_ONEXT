#[cfg(test)]
mod tests {

    use std::env;
    use std::string::ToString;

    use super::*;
    use crate::storage;
    use crate::types::{
        AssessmentKey, Bet, BetKey,  ClaimType, Game, PrivateBet,  ResultGame,
    };
    use crate::{BettingContract, BettingContractClient};
    use alloc::vec::Vec;
    use ed25519_dalek::{Keypair, Signer};
    use rand::thread_rng;
    use soroban_sdk::xdr::ToXdr;
    use soroban_sdk::{FromVal, Vec as SorobanVec};
    use soroban_sdk::{symbol_short, token};
    use soroban_sdk::{testutils::Events, vec, Env, IntoVal};
    use soroban_sdk::{
        testutils::{
            budget::Budget, Address as _, AuthorizedFunction, AuthorizedInvocation, BytesN as _,
        },
        testutils::{Address as _, Ledger, LedgerInfo},
        xdr::WriteXdr,
        Address, Bytes, BytesN, BytesN as _, InvokeError, String, Symbol, Symbol as _, TryIntoVal,
        Val,
    };
    use token::Client as TokenClient;
    use token::StellarAssetClient as TokenAdminClient;
    extern crate alloc;

    fn create_token_contract<'a>(
        e: &Env,
        admin: &Address,
    ) -> (TokenClient<'a>, TokenAdminClient<'a>) {
        let sac = e.register_stellar_asset_contract_v2(admin.clone());
        (
            token::Client::new(e, &sac.address()),
            token::StellarAssetClient::new(e, &sac.address()),
        )
    }

    fn events_handler(
        env: Env,
        all_events: std::vec::Vec<
            soroban_sdk::Vec<(
                soroban_sdk::Address,
                soroban_sdk::Vec<soroban_sdk::Val>,
                soroban_sdk::Val,
            )>,
        >,
    ) {
        for event in all_events.iter() {
            for e in event.iter() {
                let (contract_id, topics, value) = e;

                for topic in topics.iter() {
                    let sym: Result<soroban_sdk::Symbol, _> = topic.try_into_val(&env);
                    match sym {
                        Ok(symbol) => {
                            if symbol != soroban_sdk::Symbol::new(&env, "BettingGame")
                                && symbol != soroban_sdk::Symbol::new(&env, "transfer")
                            {
                                std::println!("Topic: {:?}", symbol);

                                // Convert symbol to string for easier matching
                                let symbol_str: std::string::String = symbol.to_string();

                                if symbol_str == "Game_Set" {
                                    let game_id: i128 = value.try_into_val(&env).unwrap();
                                    std::println!("Event GameSet - Game ID: {}", game_id,);
                                } else if symbol_str == "Private_Setting" {
                                    let raw: SorobanVec<Val> = value.try_into_val(&env).unwrap();
                                    let game_id: i128 =
                                        raw.get(1).unwrap().try_into_val(&env).unwrap();
                                    let setting_id: i128 =
                                        raw.get(3).unwrap().try_into_val(&env).unwrap();
                                    let user: Address =
                                      raw.get(0).unwrap().try_into_val(&env).unwrap();
                                    let amount_bet: i128 =
                                     raw.get(2).unwrap().try_into_val(&env).unwrap();
                                    std::println!(
                                "Event PrivateSetting- Game ID: {}, Admin User: {:?}, Setting: {}, Amount Bet: {}",
                                game_id, user,setting_id ,amount_bet 
                            );//Game_allUserHaveVoted
                                } else if symbol_str == "Game_allUserHaveVoted" {
                                    let game_id: i128 = value.try_into_val(&env).unwrap();
                                    std::println!("Event Game all voted setting ID: {}", game_id,);

                                }  else if symbol_str == "Game_Result" {
                                    let raw: SorobanVec<Val> = value.try_into_val(&env).unwrap();
                                    let game_id: i128 =
                                        raw.get(1).unwrap().try_into_val(&env).unwrap();
                                    let result: BetKey =
                                        raw.get(2).unwrap().try_into_val(&env).unwrap();
                                    let description: String =
                                        raw.get(0).unwrap().try_into_val(&env).unwrap();
                                    std::println!(
                                        "Event summit_result - Game ID: {}, Result:{:?}, {} ",
                                        game_id,
                                        result,
                                        description,

                                    );
                                } else if symbol_str == "Seleted_Suimmiters" {
                                    let raw: SorobanVec<Val> = value.try_into_val(&env).unwrap();

                                    let game_id: i128 =
                                        raw.get(0).unwrap().try_into_val(&env).unwrap();
                                    let main: Address =
                                        raw.get(1).unwrap().try_into_val(&env).unwrap();
                                    let summiters: SorobanVec<Address> =
                                        raw.get(2).unwrap().try_into_val(&env).unwrap();

                                    std::println!("Game ID: {}", game_id);
                                    std::println!("Main: {:?}", main);
                                    for s in summiters.iter() {
                                        std::println!("Summiter: {:?}", s);
                                    }
                                }
                                else if symbol_str == "Game_Result_Reject" {
                                    let game_id: i128 = value.try_into_val(&env).unwrap();
                                    std::println!("Event GameSet Reject - Game ID: {}", game_id,);
                                }
                                else if symbol_str == "Game_ResultbySupremeCourt" {
                                    let raw: SorobanVec<Val> = value.try_into_val(&env).unwrap();
                                    let game_id: i128 =
                                        raw.get(0).unwrap().try_into_val(&env).unwrap();
                                    let result: BetKey =
                                        raw.get(1).unwrap().try_into_val(&env).unwrap();
                                    std::println!(
                                        "Event summit_result - Game ID: {}, Result: {:?}",
                                        game_id,
                                        result
                                    );
                                }
                                else if symbol_str == "UserHonestyPoints" {
                                    let raw: SorobanVec<Val> = value.try_into_val(&env).unwrap();
                                    let user: Address =
                                        raw.get(2).unwrap().try_into_val(&env).unwrap();
                                    let points: u32 =
                                        raw.get(1).unwrap().try_into_val(&env).unwrap();
                                        let position: i128 =
                                        raw.get(0).unwrap().try_into_val(&env).unwrap();
                                    std::println!(
                                        "Event Users Points - User: {:?}, Poins: {:?}, Position: {}",
                                        user,
                                        points, position
                                    );
                                }
                            }
                        }
                        Err(_) => std::println!(" "),
                    }
                }
            }
        }
    }
    fn create_test_env() -> (
        Env,
        BettingContractClient<'static>,
        Address,
        Keypair,
        BytesN::<32>,
        Address,
        Address,
        Address,
        TokenClient<'static>,
        TokenClient<'static>,
        TokenAdminClient<'static>,
        TokenAdminClient<'static>,
    ) {
        let env = Env::default();
        env.mock_all_auths(); // Mock all authorizations for testing

        // Create mock accounts
let adminPk = Keypair::generate(&mut thread_rng());
        let public_key = BytesN::<32>::from_array(&env, &adminPk.public.to_bytes());

let admin = Address::generate(&env);
        let user = Address::generate(&env);
        let supreme = Address::generate(&env);

        // Register mock token contracts
        let (token_usd, token_usd_admin) = create_token_contract(&env, &admin);
        let (token_trust, token_trust_admin) = create_token_contract(&env, &admin);
        //pubkey byte



        // Mint initial tokens to user for testing
        token_usd_admin.mint(&user, &100_000_000);
        token_trust_admin.mint(&user, &100_000_000);
        // Register the betting contract
        let contract_id = env.register(
            BettingContract,
            (&admin, public_key.clone(), &token_usd.address, &token_trust.address, &supreme),
        );
        let client = BettingContractClient::new(&env, &contract_id);
        (
            env,
            client,
            admin,
            adminPk,
            public_key.clone(),
            user,
            token_usd.address.clone(),
            token_trust.address.clone(),
            token_usd,
            token_trust,
            token_usd_admin,
            token_trust_admin,
        )
    }

    fn set_ledger_timestamp(env: &Env, timestamp: u32) {
        env.ledger().set(LedgerInfo {
            timestamp: timestamp as u64,
            protocol_version: 23, // Updated to match soroban-sdk 23.0.1
            sequence_number: env.ledger().sequence(),
            base_reserve: 10,
            ..Default::default()
        });
    }


    #[test]
    fn test_bet_private() {
        let (            env,
            client,
            admin,
            key,
            pk,
            user,
            token_usd,
            token_trust,
            token_usd_client,
            token_trust_client,
            adm_usd,
            adm_trust,) =
            create_test_env();
        //client.init(&admin, &token_usd, &token_trust);
                    let mut all_events = Vec::new();

        // Set up a game
        let game_id = 1;

        let game = Game {
            id: game_id,
            startTime: 1000,
            endTime: 2000,
            active: false,
            league: 1,
            description: String::from_slice(&env, "Team A vs Team B"),
            team_local: 33,
            team_away: 44,
        };
        // Encode game to Bytes (variable length)
        let encoded: Vec<u8> = game.clone().to_xdr(&env).iter().collect();



        let signaturex: BytesN<64> =
            BytesN::from_array(&env, &key.sign(encoded.as_slice()).to_bytes());
        client.set_game(&game, &signaturex);

        let user2 = Address::generate(&env);
        let privateSetting = PrivateBet {
            id: 11,
            gameid: game_id,
            active: false,
            settingAdmin: user2.clone(),
            description: String::from_str(&env, "Private Bet 1"),
            amount_bet_min: 500,
            users_invated: vec![&env, user.clone(), user2.clone()],
        };
        client.set_private_bet(&user2, &privateSetting, &game_id);
        
        //let's bet to active the game
        let bet = Bet {
            id: 1,
            Setting: 11,
            bet: BetKey::Team_away,
            amount_bet: 500,
            gameid: game_id,
            collateralUsd: false,
        };

        let initial_usd_balance = token_usd_client.balance(&user);
        let initial_trust_balance = token_trust_client.balance(&user);
        std::println!("User1 honest balance initial {:?}", initial_usd_balance);
        client.bet(&user, &bet);
        all_events.push(env.events().all());

        adm_usd.mint(&user2, &100_000_000);
        adm_trust.mint(&user2, &100_000_000);

        let betx = Bet {
            id: 2,
            Setting: 11,
            bet: BetKey::Team_local,
            amount_bet: 500,
            gameid: game_id,
                        collateralUsd: false,

        };
        let initial_usd_balance = token_usd_client.balance(&user2);
        std::println!("User2 novote balance initial {:?}", initial_usd_balance);

        client.bet(&user2, &betx);
                all_events.push(env.events().all());

        let initial_usd_balance = token_usd_client.balance(&user);
        let initial_trust_balance = token_trust_client.balance(&user);
                events_handler(env.clone(), all_events);


        // Verify token transfers
        //assert_eq!(token_usd_client.balance(&user), initial_usd_balance - 500);
        //assert_eq!(token_trust_client.balance(&user), initial_trust_balance - 150); // 30% of 1000
    }

    //   Error expected = "GameHasAlreadyStarted"
    #[test]
    #[should_panic(expected = "Error(Contract, #207)")]
    fn test_bet_after_game_start() {
                let (            env,
            client,
            admin,
            key,
            pk,
            user,
            token_usd,
            token_trust,
            token_usd_client,
            token_trust_client,
            adm_usd,
            adm_trust,) =
            create_test_env();
        //client.init(&admin, &token_usd, &token_trust);

        // Set up a game
        let game_id = 1;

        let game = Game {
            id: game_id,
            startTime: 1000,
            endTime: 2000,
            active: false,
            league: 1,
            description: String::from_slice(&env, "Team A vs Team B"),
            team_local: 33,
            team_away: 44,
        };
        // Encode game to Bytes (variable length)
        let encoded: Vec<u8> = game.clone().to_xdr(&env).iter().collect();



        let signaturex: BytesN<64> =
            BytesN::from_array(&env, &key.sign(encoded.as_slice()).to_bytes());
        client.set_game(&game, &signaturex);

        let user2 = Address::generate(&env);
        let privateSetting = PrivateBet {
            id: 11,
            gameid: game_id,
            active: false,
            settingAdmin: user2.clone(),
            description: String::from_str(&env, "Private Bet 1"),
            amount_bet_min: 500,
            users_invated: vec![&env, user.clone(), user2.clone()],
        };
        client.set_private_bet(&user2, &privateSetting, &game_id);
        
        //let's bet to active the game
        let bet = Bet {
            id: 1,
            Setting: 11,
            bet: BetKey::Team_away,
            amount_bet: 500,
            gameid: game_id,
                        collateralUsd: false,

        };

        let initial_usd_balance = token_usd_client.balance(&user);
        let initial_trust_balance = token_trust_client.balance(&user);
        std::println!("User1 honest balance initial {:?}", initial_usd_balance);
        client.bet(&user, &bet);

        adm_usd.mint(&user2, &100_000_000);
        adm_trust.mint(&user2, &100_000_000);

        let betx = Bet {
            id: 2,
            Setting: 11,
            bet: BetKey::Team_local,
            amount_bet: 500,
            gameid: game_id,
                        collateralUsd: false,

        };
        let initial_usd_balance = token_usd_client.balance(&user2);
        std::println!("User2 novote balance initial {:?}", initial_usd_balance);
                set_ledger_timestamp(&env, 1500);

        client.bet(&user2, &betx);

        let initial_usd_balance = token_usd_client.balance(&user);
        let initial_trust_balance = token_trust_client.balance(&user);


        // Verify token transfers
        assert_eq!(token_usd_client.balance(&user), initial_usd_balance - 500);
        assert_eq!(token_trust_client.balance(&user), initial_trust_balance - 150); // 30% of 1000

    }

    #[test]
    fn test_claim_money_noactive_public() {
        let (env, client, admin, key, pk, user, token_usd, token_trust, usd_client, trust_client, _, _) =
            create_test_env();
        //client.init(&admin, &token_usd, &token_trust);
        let setting =33;
        // Set up a game
        let game_id = 1;
        let game = Game {
            id: game_id,
            startTime: 1000,
            endTime: 2000,

            active: false,
            league: 1,
            description: String::from_slice(&env, "Team A vs Team B"),
            team_local: 33,
            team_away: 44,
        };
        // Encode game to Bytes (variable length
        let encoded: Vec<u8> = game.clone().to_xdr(&env).iter().collect();


        let signaturex: BytesN<64> =
            BytesN::from_array(&env, &key.sign(encoded.as_slice()).to_bytes());
        client.set_game(&game, &signaturex );
        let user2 = Address::generate(&env);
        let privateSetting = PrivateBet {
            id: setting,
            gameid: game_id,
            active: false,
            settingAdmin: user2.clone(),
            description: String::from_str(&env, "Private Bet 1"),
            amount_bet_min: 500,
            users_invated: vec![&env, user.clone(), user2.clone()],
        };
        client.set_private_bet(&user2, &privateSetting, &game_id);
        // Place a public bet
        let bet = Bet {
            id: 1,
            Setting: setting,
            bet: BetKey::Team_local,
            amount_bet: 500,
            gameid: game_id,
                                    collateralUsd: false,

        };
        client.bet(&user, &bet);
        set_ledger_timestamp(&env, 5000);

        // Claim before game starts
        let initial_usd_balance = usd_client.balance(&user);
        let initial_trust_balance = trust_client.balance(&user);

        client.claim_refund(&user, &setting);

        // Verify token refunds
        assert_eq!(usd_client.balance(&user), initial_usd_balance + 500);
        assert_eq!(trust_client.balance(&user), initial_trust_balance + 150);
    }

    #[test]
    fn test_summit_result() {
        let (            env,
            client,
            admin,
            key,
            pk,
            user,
            token_usd,
            token_trust,
            token_usd_client,
            token_trust_client,
            adm_usd,
            adm_trust,) =
            create_test_env();
        //client.init(&admin, &token_usd, &token_trust);
 let mut all_events = Vec::new();

        // Set up a game
        let game_id = 1;

        let game = Game {
            id: game_id,
            startTime: 1000,
            endTime: 2000,
            active: false,
            league: 1,
            description: String::from_slice(&env, "Team A vs Team B"),
            team_local: 33,
            team_away: 44,
        };
        // Encode game to Bytes (variable length)
        let encoded: Vec<u8> = game.clone().to_xdr(&env).iter().collect();



        let signaturex: BytesN<64> =
            BytesN::from_array(&env, &key.sign(encoded.as_slice()).to_bytes());
        client.set_game(&game, &signaturex);

        let user2 = Address::generate(&env);
        let privateSetting = PrivateBet {
            id: 11,
            gameid: game_id,
            active: false,
            settingAdmin: user2.clone(),
            description: String::from_str(&env, "Private Bet 1"),
            amount_bet_min: 500,
            users_invated: vec![&env, user.clone(), user2.clone()],
        };
        client.set_private_bet(&user2, &privateSetting, &game_id);
        
        //let's bet to active the game
        let bet = Bet {
            id: 1,
            Setting: 11,
            bet: BetKey::Team_away,
            amount_bet: 500,
            gameid: game_id,
                                    collateralUsd: false,

        };

        let initial_usd_balance = token_usd_client.balance(&user);
        let initial_trust_balance = token_trust_client.balance(&user);
        std::println!("User1 honest balance initial {:?}", initial_usd_balance);
        client.bet(&user, &bet);
        all_events.push(env.events().all());

        adm_usd.mint(&user2, &100_000_000);
        adm_trust.mint(&user2, &100_000_000);

        let betx = Bet {
            id: 2,
            Setting: 11,
            bet: BetKey::Team_local,
            amount_bet: 500,
            gameid: game_id,
                                    collateralUsd: false,

        };
        let initial_usd_balance = token_usd_client.balance(&user2);
        std::println!("User2 novote balance initial {:?}", initial_usd_balance);

        client.bet(&user2, &betx);
                all_events.push(env.events().all());

        let initial_usd_balance = token_usd_client.balance(&user);
        let initial_trust_balance = token_trust_client.balance(&user);
        // Set ledger timestamp after game end
        set_ledger_timestamp(&env, 2500);
        
        let result = ResultGame {
            id: 1,
            gameid: game_id,
            setting:11,
            result: BetKey::Team_local,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };

        client.summitResult(&user2,&11, &result);
        //events_handler(env.clone(), all_events);

        // Show events for debugging

        //assert_eq!(submitted_result, result);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #226)")]
    fn test_summit_result_unauthorized() {
        let (            env,
            client,
            admin,
            key,
            pk,
            user,
            token_usd,
            token_trust,
            token_usd_client,
            token_trust_client,
            adm_usd,
            adm_trust,) =
            create_test_env();
        //client.init(&admin, &token_usd, &token_trust);
 let mut all_events = Vec::new();

        // Set up a game
        let game_id = 1;

        let game = Game {
            id: game_id,
            startTime: 1000,
            endTime: 2000,
            active: false,
            league: 1,
            description: String::from_slice(&env, "Team A vs Team B"),
            team_local: 33,
            team_away: 44,
        };
        // Encode game to Bytes (variable length)
        let encoded: Vec<u8> = game.clone().to_xdr(&env).iter().collect();



        let signaturex: BytesN<64> =
            BytesN::from_array(&env, &key.sign(encoded.as_slice()).to_bytes());
        client.set_game(&game, &signaturex);

        let user2 = Address::generate(&env);
                let user3 = Address::generate(&env);

        let privateSetting = PrivateBet {
            id: 11,
            gameid: game_id,
            active: false,
            settingAdmin: user2.clone(),
            description: String::from_str(&env, "Private Bet 1"),
            amount_bet_min: 500,
            users_invated: vec![&env, user.clone(), user2.clone()],
        };
        client.set_private_bet(&user2, &privateSetting, &game_id);
        
        //let's bet to active the game
        let bet = Bet {
            id: 1,
            Setting: 11,
            bet: BetKey::Team_away,
            amount_bet: 500,
            gameid: game_id,
                                    collateralUsd: false,

        };

        let initial_usd_balance = token_usd_client.balance(&user);
        let initial_trust_balance = token_trust_client.balance(&user);
        std::println!("User1 honest balance initial {:?}", initial_usd_balance);
        client.bet(&user, &bet);
        all_events.push(env.events().all());

        adm_usd.mint(&user2, &100_000_000);
        adm_trust.mint(&user2, &100_000_000);

        let betx = Bet {
            id: 2,
            Setting: 11,
            bet: BetKey::Team_local,
            amount_bet: 500,
            gameid: game_id,
                                    collateralUsd: false,

        };
        let initial_usd_balance = token_usd_client.balance(&user2);
        std::println!("User2 novote balance initial {:?}", initial_usd_balance);

        client.bet(&user2, &betx);
                all_events.push(env.events().all());

        let initial_usd_balance = token_usd_client.balance(&user);
        let initial_trust_balance = token_trust_client.balance(&user);
        // Set ledger timestamp after game end
        set_ledger_timestamp(&env, 2500);
        
        let result = ResultGame {
            id: 1,
            gameid: game_id,
            setting:11,
            result: BetKey::Team_local,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };

        client.summitResult(&user3,&11, &result);
    }

    #[test]
    fn test_assess_result() {
        let  (            env,
            client,
            admin,
            key,
            pk,
            user,
            token_usd,
            token_trust,
            token_usd_client,
            token_trust_client,
            adm_usd,
            adm_trust,) =
            create_test_env();
        //client.init(&admin, &token_usd, &token_trust);
 let mut all_events = Vec::new();

        // Set up a game
        let game_id = 1;

        let game = Game {
            id: game_id,
            startTime: 1000,
            endTime: 2000,
            active: false,
            league: 1,
            description: String::from_slice(&env, "Team A vs Team B"),
            team_local: 33,
            team_away: 44,
        };
        // Encode game to Bytes (variable length)
        let encoded: Vec<u8> = game.clone().to_xdr(&env).iter().collect();



        let signaturex: BytesN<64> =
            BytesN::from_array(&env, &key.sign(encoded.as_slice()).to_bytes());
        client.set_game(&game, &signaturex);

        let user2 = Address::generate(&env);
                let user3 = Address::generate(&env);

        let privateSetting = PrivateBet {
            id: 11,
            gameid: game_id,
            active: false,
            settingAdmin: user2.clone(),
            description: String::from_str(&env, "Private Bet 1"),
            amount_bet_min: 500,
            users_invated: vec![&env, user.clone(), user2.clone()],
        };
        client.set_private_bet(&user2, &privateSetting, &game_id);
        
        //let's bet to active the game
        let bet = Bet {
            id: 1,
            Setting: 11,
            bet: BetKey::Team_away,
            amount_bet: 500,
            gameid: game_id,
                                    collateralUsd: false,

        };

        let initial_usd_balance = token_usd_client.balance(&user);
        let initial_trust_balance = token_trust_client.balance(&user);
        std::println!("User1 honest balance initial {:?}", initial_usd_balance);
        client.bet(&user, &bet);
        all_events.push(env.events().all());

        adm_usd.mint(&user2, &100_000_000);
        adm_trust.mint(&user2, &100_000_000);

        let betx = Bet {
            id: 2,
            Setting: 11,
            bet: BetKey::Team_local,
            amount_bet: 500,
            gameid: game_id,
                                    collateralUsd: false,

        };
        let initial_usd_balance = token_usd_client.balance(&user2);
        std::println!("User2 novote balance initial {:?}", initial_usd_balance);

        client.bet(&user2, &betx);
                all_events.push(env.events().all());

        let initial_usd_balance = token_usd_client.balance(&user);
        let initial_trust_balance = token_trust_client.balance(&user);
        // Set ledger timestamp after game end
        set_ledger_timestamp(&env, 2500);
        
        let result = ResultGame {
            id: 1,
            gameid: game_id,
            setting:11,
            result: BetKey::Team_local,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };

        client.summitResult(&user,&11, &result);
        client.assessResult(&user, &11, &AssessmentKey::approve);
        //client.summitResult(&summiter2, &result);

    }
   
    #[test]
    fn test_claim_winner_honest() {
        let (            env,
            client,
            admin,
            key,
            pk,
            user,
            token_usd,
            token_trust,
            token_usd_client,
            token_trust_client,
            adm_usd,
            adm_trust,) =
            create_test_env();
        //client.init(&admin, &token_usd, &token_trust);
 let mut all_events = Vec::new();

        // Set up a game
        let game_id = 1;

        let game = Game {
            id: game_id,
            startTime: 1000,
            endTime: 2000,
            active: false,
            league: 1,
            description: String::from_slice(&env, "Team A vs Team B"),
            team_local: 33,
            team_away: 44,
        };
        // Encode game to Bytes (variable length)
        let encoded: Vec<u8> = game.clone().to_xdr(&env).iter().collect();



        let signaturex: BytesN<64> =
            BytesN::from_array(&env, &key.sign(encoded.as_slice()).to_bytes());
        client.set_game(&game, &signaturex);

        let user2 = Address::generate(&env);
                let user3 = Address::generate(&env);

        let privateSetting = PrivateBet {
            id: 11,
            gameid: game_id,
            active: false,
            settingAdmin: user2.clone(),
            description: String::from_str(&env, "Private Bet 1"),
            amount_bet_min: 500,
            users_invated: vec![&env, user.clone(), user2.clone()],
        };
        client.set_private_bet(&user2, &privateSetting, &game_id);
        
        //let's bet to active the game
        let bet = Bet {
            id: 1,
            Setting: 11,
            bet: BetKey::Team_away,
            amount_bet: 500,
            gameid: game_id,
                                    collateralUsd: false,

        };

        let initial_usd_balance = token_usd_client.balance(&user);
        let initial_trust_balance = token_trust_client.balance(&user);
        std::println!("User1 honest balance initial {:?}", initial_usd_balance);
        client.bet(&user, &bet);
        all_events.push(env.events().all());

        adm_usd.mint(&user2, &100_000_000);
        adm_trust.mint(&user2, &100_000_000);

        let betx = Bet {
            id: 2,
            Setting: 11,
            bet: BetKey::Team_local,
            amount_bet: 500,
            gameid: game_id,
                                    collateralUsd: false,

        };
        let initial_usd_balance2 = token_usd_client.balance(&user2);
        std::println!("User2 novote balance initial {:?}", initial_usd_balance);
                let initial_trust_balance2 = token_trust_client.balance(&user2);


        client.bet(&user2, &betx);
                all_events.push(env.events().all());

        let initial_usd_balance = token_usd_client.balance(&user);
        let initial_trust_balance = token_trust_client.balance(&user);
        // Set ledger timestamp after game end
        set_ledger_timestamp(&env, 2500);
        
        let result = ResultGame {
            id: 1,
            gameid: game_id,
            setting:11,
            result: BetKey::Team_local,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };

        client.summitResult(&user2,&11, &result);
        client.assessResult(&user, &11, &AssessmentKey::approve);
        client.execute_distribution( &11);


        
        client.claim(&user2, &ClaimType::User, &11);
        all_events.push(env.events().all());
        client.claim(&user, &ClaimType::User, &11);
                all_events.push(env.events().all());


        // Verify token transfers (winner gets bet + share of pool)
                std::println!(" Final {:?}", token_usd_client.balance(&user2));
                                std::println!(" initial {:?}", initial_usd_balance2);


        assert!(token_usd_client.balance(&user2) > initial_usd_balance2);
        //assert_eq!(token_trust_client.balance(&user2), initial_trust_balance2); // Trust tokens returned
                        events_handler(env.clone(), all_events);

    }
    #[test]
    fn test_refund_nosupreme() {
        let (            env,
            client,
            admin,
            key,
            pk,
            user,
            token_usd,
            token_trust,
            token_usd_client,
            token_trust_client,
            adm_usd,
            adm_trust,) =
            create_test_env();
        //client.init(&admin, &token_usd, &token_trust);
 let mut all_events = Vec::new();

        // Set up a game
        let game_id = 1;

        let game = Game {
            id: game_id,
            startTime: 1000,
            endTime: 2000,
            active: false,
            league: 1,
            description: String::from_slice(&env, "Team A vs Team B"),
            team_local: 33,
            team_away: 44,
        };
        // Encode game to Bytes (variable length)
        let encoded: Vec<u8> = game.clone().to_xdr(&env).iter().collect();



        let signaturex: BytesN<64> =
            BytesN::from_array(&env, &key.sign(encoded.as_slice()).to_bytes());
        client.set_game(&game, &signaturex);

        let user2 = Address::generate(&env);
                let user3 = Address::generate(&env);

        let privateSetting = PrivateBet {
            id: 11,
            gameid: game_id,
            active: false,
            settingAdmin: user2.clone(),
            description: String::from_str(&env, "Private Bet 1"),
            amount_bet_min: 500,
            users_invated: vec![&env, user.clone(), user2.clone()],
        };
        client.set_private_bet(&user2, &privateSetting, &game_id);
        
        //let's bet to active the game
        let bet = Bet {
            id: 1,
            Setting: 11,
            bet: BetKey::Team_away,
            amount_bet: 500,
            gameid: game_id,
                                    collateralUsd: true,

        };

        let initial_usd_balance = token_usd_client.balance(&user);
        let initial_trust_balance = token_trust_client.balance(&user);
        std::println!("User1 honest balance initial {:?}", initial_usd_balance);
        client.bet(&user, &bet);
        all_events.push(env.events().all());

        adm_usd.mint(&user2, &100_000_000);
        adm_trust.mint(&user2, &100_000_000);

        let betx = Bet {
            id: 2,
            Setting: 11,
            bet: BetKey::Team_local,
            amount_bet: 500,
            gameid: game_id,
                                    collateralUsd: false,

        };
        let initial_usd_balance2 = token_usd_client.balance(&user2);
        std::println!("User2 novote balance initial {:?}", initial_usd_balance);
                let initial_trust_balance2 = token_trust_client.balance(&user2);


        client.bet(&user2, &betx);
                all_events.push(env.events().all());

        // Set ledger timestamp after game end
                set_ledger_timestamp(&env, 2003);

                let result = ResultGame {
            id: 1,
            gameid: game_id,
            setting:11,
            result: BetKey::Team_local,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };
                client.summitResult(&user,&11, &result);
                client.assessResult(&user2, &11, &AssessmentKey::reject);
        set_ledger_timestamp(&env, 1280010);

        client.claim_refund(&user, &11);
        client.claim_refund(&user2, &11);

        std::println!("User balance final {:?}", token_usd_client.balance(&user));
        std::println!("User2 balance final {:?}", token_usd_client.balance(&user2));

        std::println!("admin balance final {:?}", token_usd_client.balance(&admin));
        // Verify token transfers (winner gets bet + share of pool)
        assert_eq!(token_trust_client.balance(&user), initial_trust_balance); // Trust tokens returned
                assert_eq!(token_usd_client.balance(&user), initial_usd_balance); // Trust tokens returned

    }
    #[test]
    fn test_refund_nosummition() {
        let (            env,
            client,
            admin,
            key,
            pk,
            user,
            token_usd,
            token_trust,
            token_usd_client,
            token_trust_client,
            adm_usd,
            adm_trust,) =
            create_test_env();
        //client.init(&admin, &token_usd, &token_trust);
 let mut all_events = Vec::new();

        // Set up a game
        let game_id = 1;

        let game = Game {
            id: game_id,
            startTime: 1000,
            endTime: 2000,
            active: false,
            league: 1,
            description: String::from_slice(&env, "Team A vs Team B"),
            team_local: 33,
            team_away: 44,
        };
        // Encode game to Bytes (variable length)
        let encoded: Vec<u8> = game.clone().to_xdr(&env).iter().collect();



        let signaturex: BytesN<64> =
            BytesN::from_array(&env, &key.sign(encoded.as_slice()).to_bytes());
        client.set_game(&game, &signaturex);

        let user2 = Address::generate(&env);
                let user3 = Address::generate(&env);

        let privateSetting = PrivateBet {
            id: 11,
            gameid: game_id,
            active: false,
            settingAdmin: user2.clone(),
            description: String::from_str(&env, "Private Bet 1"),
            amount_bet_min: 500,
            users_invated: vec![&env, user.clone(), user2.clone()],
        };
        client.set_private_bet(&user2, &privateSetting, &game_id);
        
        //let's bet to active the game
        let bet = Bet {
            id: 1,
            Setting: 11,
            bet: BetKey::Team_away,
            amount_bet: 500,
            gameid: game_id,
                                    collateralUsd: false,

        };

        let initial_usd_balance = token_usd_client.balance(&user);
        let initial_trust_balance = token_trust_client.balance(&user);
        std::println!("User1 honest balance initial {:?}", initial_usd_balance);
        client.bet(&user, &bet);
        all_events.push(env.events().all());

        adm_usd.mint(&user2, &100_000_000);
        adm_trust.mint(&user2, &100_000_000);

        let betx = Bet {
            id: 2,
            Setting: 11,
            bet: BetKey::Team_local,
            amount_bet: 500,
            gameid: game_id,
                                    collateralUsd: false,

        };
        let initial_usd_balance2 = token_usd_client.balance(&user2);
        std::println!("User2 novote balance initial {:?}", initial_usd_balance);
                let initial_trust_balance2 = token_trust_client.balance(&user2);


        client.bet(&user2, &betx);
                all_events.push(env.events().all());


        // Set ledger timestamp after game end


        set_ledger_timestamp(&env, 1280010);

        client.claim_refund(&user, &11);
        client.claim_refund(&user2, &11);

        std::println!("User balance final {:?}", token_usd_client.balance(&user));
        std::println!("User2 balance final {:?}", token_usd_client.balance(&user2));

        std::println!("admin balance final {:?}", token_usd_client.balance(&admin));
        // Verify token transfers (winner gets bet + share of pool)
        assert_eq!(token_usd_client.balance(&user) , initial_usd_balance); // Trust tokens returned
        assert_eq!(token_trust_client.balance(&user), initial_trust_balance); // Trust tokens returned
    }
    #[test]
    fn test_cancel() {
        let (            env,
            client,
            admin,
            key,
            pk,
            user,
            token_usd,
            token_trust,
            token_usd_client,
            token_trust_client,
            adm_usd,
            adm_trust,) =
            create_test_env();
        //client.init(&admin, &token_usd, &token_trust);
 let mut all_events = Vec::new();

        // Set up a game
        let game_id = 1;

        let game = Game {
            id: game_id,
            startTime: 1000,
            endTime: 2000,
            active: false,
            league: 1,
            description: String::from_slice(&env, "Team A vs Team B"),
            team_local: 33,
            team_away: 44,
        };
        // Encode game to Bytes (variable length)
        let encoded: Vec<u8> = game.clone().to_xdr(&env).iter().collect();



        let signaturex: BytesN<64> =
            BytesN::from_array(&env, &key.sign(encoded.as_slice()).to_bytes());
        client.set_game(&game, &signaturex);

        let user2 = Address::generate(&env);
                let user3 = Address::generate(&env);

        let privateSetting = PrivateBet {
            id: 11,
            gameid: game_id,
            active: false,
            settingAdmin: user2.clone(),
            description: String::from_str(&env, "Private Bet 1"),
            amount_bet_min: 500,
            users_invated: vec![&env, user.clone(), user2.clone()],
        };
        client.set_private_bet(&user2, &privateSetting, &game_id);
        
        //let's bet to active the game
        let bet = Bet {
            id: 1,
            Setting: 11,
            bet: BetKey::Team_away,
            amount_bet: 500,
            gameid: game_id,
                                    collateralUsd: false,

        };

        let initial_usd_balance = token_usd_client.balance(&user);
        let initial_trust_balance = token_trust_client.balance(&user);
        std::println!("User1 honest balance initial {:?}", initial_usd_balance);
        client.bet(&user, &bet);
        all_events.push(env.events().all());

        adm_usd.mint(&user2, &100_000_000);
        adm_trust.mint(&user2, &100_000_000);

        let betx = Bet {
            id: 2,
            Setting: 11,
            bet: BetKey::Team_local,
            amount_bet: 500,
            gameid: game_id,
                                    collateralUsd: false,

        };
        let initial_usd_balance2 = token_usd_client.balance(&user2);
        std::println!("User2 novote balance initial {:?}", initial_usd_balance);
                let initial_trust_balance2 = token_trust_client.balance(&user2);



        client.bet(&user2, &betx);
                all_events.push(env.events().all());

        // Set ledger timestamp after game end
        set_ledger_timestamp(&env, 2500);
        
        let result = ResultGame {
            id: 1,
            gameid: game_id,
            setting:11,
            result: BetKey::Cancel,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };

        client.summitResult(&user2,&11, &result);
        client.assessResult(&user, &11, &AssessmentKey::approve);
        client.execute_distribution( &11);


        
        client.claim_refund(&user, &11);
        all_events.push(env.events().all());
        client.claim_refund(&user2, &11);
                all_events.push(env.events().all());


        // Verify token transfers (winner gets bet + share of pool)
                std::println!(" Final {:?}", token_usd_client.balance(&user2));
                                std::println!(" initial {:?}", initial_usd_balance2);

                        events_handler(env.clone(), all_events);

        std::println!("User balance final {:?}", token_usd_client.balance(&user));
        std::println!("User2 balance final {:?}", token_usd_client.balance(&user2));

        // Verify token transfers (winner gets bet + share of pool)
        assert_eq!(token_usd_client.balance(&user), initial_usd_balance); // Trust tokens returned
        assert_eq!(token_trust_client.balance(&user), initial_trust_balance); // Trust tokens returned
    }
        #[test]
    fn test_supreme_court() {
        let (            env,
            client,
            admin,
            key,
            pk,
            user,
            token_usd,
            token_trust,
            token_usd_client,
            token_trust_client,
            adm_usd,
            adm_trust,) =
            create_test_env();
        //client.init(&admin, &token_usd, &token_trust);
 let mut all_events = Vec::new();

        // Set up a game

        let addUser1 = Address::generate(&env);
                    adm_usd.mint(&addUser1, &100_000_000);
        adm_trust.mint(&addUser1, &100_000_000);
            let addUser2 = Address::generate(&env);
            adm_usd.mint(&addUser2, &100_000_000);
        adm_trust.mint(&addUser2, &100_000_000);
            let addUser3 = Address::generate(&env);
            adm_usd.mint(&addUser3, &100_000_000);
        adm_trust.mint(&addUser3, &100_000_000);
            let addUser4 = Address::generate(&env);
            adm_usd.mint(&addUser4, &100_000_000);
        adm_trust.mint(&addUser4, &100_000_000);
            let addUser5 = Address::generate(&env);
            adm_usd.mint(&addUser5, &100_000_000);
        adm_trust.mint(&addUser5, &100_000_000);
            let addUser6 = Address::generate(&env);
            adm_usd.mint(&addUser6, &100_000_000);
        adm_trust.mint(&addUser6, &100_000_000);

        //game setting
        let mut game_id = 1;
        let mut setting = 20;
        let game = Game {
            id: game_id,
            startTime: 1000,
            endTime: 2000,
            active: false,
            league: 1,
            description: String::from_slice(&env, "Team A vs Team B"),
            team_local: 33,
            team_away: 44,
        };
        // Encode game to Bytes (variable length)
        let encoded: Vec<u8> = game.clone().to_xdr(&env).iter().collect();



        let signaturex: BytesN<64> =
            BytesN::from_array(&env, &key.sign(encoded.as_slice()).to_bytes());
        client.set_game(&game, &signaturex);

        let privateSetting = PrivateBet {
            id: setting,
            gameid: game_id,
            active: false,
            settingAdmin: addUser1.clone(),
            description: String::from_str(&env, "Private Bet 1"),
            amount_bet_min: 500,
            users_invated: vec![&env, addUser1.clone(), addUser2.clone()],
        };
        client.set_private_bet(&addUser1, &privateSetting, &game_id);
        
        //let's bet to active the game
        let bet = Bet {
            id: 12,
            Setting: setting,
            bet: BetKey::Team_away,
            amount_bet: 500,
            gameid: game_id,
                                    collateralUsd: false,

        };

        let initial_usd_balance = token_usd_client.balance(&addUser1);
        let initial_trust_balance = token_trust_client.balance(&addUser1);
        std::println!("User1 honest balance initial {:?}", initial_usd_balance);
        client.bet(&addUser1, &bet);

        all_events.push(env.events().all());

        adm_usd.mint(&addUser2, &100_000_000);
        adm_trust.mint(&addUser2, &100_000_000);

        let betx = Bet {
            id: 2,
            Setting: setting,
            bet: BetKey::Team_local,
            amount_bet: 500,
            gameid: game_id,
                                    collateralUsd: false,

        };
        let initial_usd_balance2 = token_usd_client.balance(&addUser2);
                let initial_trust_balance2 = token_trust_client.balance(&addUser2);


        client.bet(&addUser2, &betx);
                all_events.push(env.events().all());


        // Set ledger timestamp after game end
        set_ledger_timestamp(&env, 2500);
        
        let result = ResultGame {
            id: 1,
            gameid: game_id,
            setting:setting,
            result: BetKey::Team_local,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };

        client.summitResult(&addUser2,&setting, &result);
        client.assessResult(&addUser1, &setting, &AssessmentKey::approve);
        client.execute_distribution( &setting);


        
        client.claim(&addUser2, &ClaimType::User, &setting);
        all_events.push(env.events().all());
        client.claim(&addUser1, &ClaimType::User, &setting);
                all_events.push(env.events().all());


        // Verify token transfers (winner gets bet + share of pool)
                std::println!(" Final {:?}", token_usd_client.balance(&addUser2));
                                std::println!(" initial {:?}", initial_usd_balance2);


        assert!(token_usd_client.balance(&addUser2) > initial_usd_balance2);
        //assert_eq!(token_trust_client.balance(&user2), initial_trust_balance2); // Trust tokens returned
                        events_handler(env.clone(), all_events.clone());
        



        //Second Attempt
     
                //game setting
        game_id = 122;
        setting = 21;
        let gamex = Game {
            id: game_id,
            startTime: 2500,
            endTime: 3000,
            active: false,
            league: 1,
            description: String::from_slice(&env, "Team A vs Team B"),
            team_local: 33,
            team_away: 44,
        };
        // Encode game to Bytes (variable length)
        let encoded: Vec<u8> = gamex.clone().to_xdr(&env).iter().collect();



        let signaturex: BytesN<64> =
            BytesN::from_array(&env, &key.sign(encoded.as_slice()).to_bytes());
        client.set_game(&gamex, &signaturex);
        all_events.push(env.events().all());

        let privateSettingx = PrivateBet {
            id: setting,
            gameid: game_id,
            active: false,
            settingAdmin: addUser3.clone(),
            description: String::from_str(&env, "Private Bet 1"),
            amount_bet_min: 500,
            users_invated: vec![&env, addUser3.clone(), addUser4.clone()],
        };
        client.set_private_bet(&addUser3, &privateSettingx, &game_id);
                all_events.push(env.events().all());

        //let's bet to active the game
        let betx2 = Bet {
            id: 12,
            Setting: setting,
            bet: BetKey::Team_away,
            amount_bet: 500,
            gameid: game_id,
                                    collateralUsd: false,

        };
                        events_handler(env.clone(), all_events.clone());

        let initial_usd_balance = token_usd_client.balance(&addUser4);
        let initial_trust_balance = token_trust_client.balance(&addUser4);
        std::println!("User1 honest balance initial {:?}", initial_usd_balance);
        client.bet(&addUser3, &betx2);
        all_events.push(env.events().all());

        adm_usd.mint(&addUser4, &100_000_000);
        adm_trust.mint(&addUser4, &100_000_000);

        let betxx2 = Bet {
            id: 2,
            Setting: setting,
            bet: BetKey::Team_local,
            amount_bet: 500,
            gameid: game_id,
                                    collateralUsd: false,

        };
        let initial_usd_balance2 = token_usd_client.balance(&addUser4);
                let initial_trust_balance2 = token_trust_client.balance(&addUser4);


        client.bet(&addUser4, &betxx2);
                all_events.push(env.events().all());


        // Set ledger timestamp after game end
        set_ledger_timestamp(&env, 3500);
        
        let resultxx = ResultGame {
            id: 1,
            gameid: game_id,
            setting:setting,
            result: BetKey::Team_local,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };

        client.summitResult(&addUser4,&setting, &resultxx);
        client.assessResult(&addUser3, &setting, &AssessmentKey::approve);
        client.execute_distribution( &setting);


        
        client.claim(&addUser4, &ClaimType::User, &setting);
        all_events.push(env.events().all());
        client.claim(&addUser3, &ClaimType::User, &setting);
                all_events.push(env.events().all());


        // Verify token transfers (winner gets bet + share of pool)
                std::println!(" Final {:?}", token_usd_client.balance(&addUser4));
                                std::println!(" initial {:?}", initial_usd_balance2);


        assert!(token_usd_client.balance(&addUser2) > initial_usd_balance2);
        //assert_eq!(token_trust_client.balance(&user2), initial_trust_balance2); // Trust tokens returned
                        events_handler(env.clone(), all_events.clone());
        
        //Second Attempt
     
                //game setting
        game_id = 1222;
        setting = 221;
        let gamex2 = Game {
            id: game_id,
            startTime: 3500,
            endTime: 4000,
            active: false,
            league: 1,
            description: String::from_slice(&env, "Team A vs Team B"),
            team_local: 33,
            team_away: 44,
        };
        // Encode game to Bytes (variable length)
        let encoded: Vec<u8> = gamex2.clone().to_xdr(&env).iter().collect();



        let signaturex: BytesN<64> =
            BytesN::from_array(&env, &key.sign(encoded.as_slice()).to_bytes());
        client.set_game(&gamex2, &signaturex);
        all_events.push(env.events().all());

        let privateSettingx2 = PrivateBet {
            id: setting,
            gameid: game_id,
            active: false,
            settingAdmin: addUser2.clone(),
            description: String::from_str(&env, "Private Bet 1"),
            amount_bet_min: 500,
            users_invated: vec![&env, addUser2.clone(), addUser1.clone()],
        };
        client.set_private_bet(&addUser2, &privateSettingx2, &game_id);
                all_events.push(env.events().all());

        //let's bet to active the game
        let betx23 = Bet {
            id: 12,
            Setting: setting,
            bet: BetKey::Team_away,
            amount_bet: 500,
            gameid: game_id,
                                    collateralUsd: false,

        };
                        events_handler(env.clone(), all_events.clone());

        let initial_usd_balance = token_usd_client.balance(&addUser1);
        let initial_trust_balance = token_trust_client.balance(&addUser1);
        std::println!("User1 honest balance initial {:?}", initial_usd_balance);
        client.bet(&addUser1, &betx23);
        all_events.push(env.events().all());

        adm_usd.mint(&addUser2, &100_000_000);
        adm_trust.mint(&addUser2, &100_000_000);

        let betxx21 = Bet {
            id: 2,
            Setting: setting,
            bet: BetKey::Team_local,
            amount_bet: 500,
            gameid: game_id,
                                    collateralUsd: false,

        };
        let initial_usd_balance2 = token_usd_client.balance(&addUser2);
                let initial_trust_balance2 = token_trust_client.balance(&addUser2);


        client.bet(&addUser2, &betxx21);
                all_events.push(env.events().all());


        // Set ledger timestamp after game end
        set_ledger_timestamp(&env, 4500);
        
        let resultxx2 = ResultGame {
            id: 1,
            gameid: game_id,
            setting:setting,
            result: BetKey::Team_local,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };

        client.summitResult(&addUser2,&setting, &resultxx2);
        client.assessResult(&addUser1, &setting, &AssessmentKey::approve);
        client.execute_distribution( &setting);
        client.claim(&admin, &ClaimType::Protocol, &setting);


        
        client.claim(&addUser2, &ClaimType::User, &setting);
        all_events.push(env.events().all());
        client.claim(&addUser1, &ClaimType::User, &setting);
                all_events.push(env.events().all());


        // Verify token transfers (winner gets bet + share of pool)
                std::println!(" Final {:?}", token_usd_client.balance(&addUser1));
                                std::println!(" initial {:?}", initial_usd_balance2);


        assert!(token_usd_client.balance(&addUser2) > initial_usd_balance2);
        //assert_eq!(token_trust_client.balance(&user2), initial_trust_balance2); // Trust tokens returned
                        events_handler(env.clone(), all_events.clone());


        //************************************Supreme*********** */

                
        //Second Attempt
     
                //game setting
        game_id = 12222;
        setting = 2212;
        let gamex22 = Game {
            id: game_id,
            startTime: 5500,
            endTime: 6000,
            active: false,
            league: 1,
            description: String::from_slice(&env, "Team A vs Team B"),
            team_local: 33,
            team_away: 44,
        };
        // Encode game to Bytes (variable length)
        let encoded: Vec<u8> = gamex22.clone().to_xdr(&env).iter().collect();



        let signaturex: BytesN<64> =
            BytesN::from_array(&env, &key.sign(encoded.as_slice()).to_bytes());
        client.set_game(&gamex22, &signaturex);
        all_events.push(env.events().all());
        let settingx =15;
        let privateSettingx22x = PrivateBet {
            id: settingx,
            gameid: game_id,
            active: false,
            settingAdmin: addUser2.clone(),
            description: String::from_str(&env, "Private Bet 1"),
            amount_bet_min: 500,
            users_invated: vec![&env, addUser2.clone(), addUser3.clone()],
        };
        client.set_private_bet(&addUser2, &privateSettingx22x, &game_id);
                all_events.push(env.events().all());

                        //let's bet to active the game
        let betx234 = Bet {
            id: 12,
            Setting: settingx,
            bet: BetKey::Team_away,
            amount_bet: 500,
            gameid: game_id,
                                    collateralUsd: false,

        };
                        events_handler(env.clone(), all_events.clone());


        client.bet(&addUser2, &betx234);
        all_events.push(env.events().all());



        let betxx212 = Bet {
            id: 2,
            Setting: settingx,
            bet: BetKey::Team_local,
            amount_bet: 500,
            gameid: game_id,
                                    collateralUsd: false,

        };


        client.bet(&addUser3, &betxx212);
                all_events.push(env.events().all());




        




                        let settingx1 =151;
        let privateSettingx22x1 = PrivateBet {
            id: settingx1,
            gameid: game_id,
            active: false,
            settingAdmin: addUser1.clone(),
            description: String::from_str(&env, "Private Bet 1"),
            amount_bet_min: 500,
            users_invated: vec![&env, addUser1.clone(), addUser4.clone()],
        };
        client.set_private_bet(&addUser1, &privateSettingx22x1, &game_id);
                all_events.push(env.events().all());

                //let's bet to active the game
        let betx234 = Bet {
            id: 12,
            Setting: settingx1,
            bet: BetKey::Team_away,
            amount_bet: 500,
            gameid: game_id,
                                    collateralUsd: false,

        };
                        events_handler(env.clone(), all_events.clone());


        client.bet(&addUser1, &betx234);
        all_events.push(env.events().all());



        let betxx212 = Bet {
            id: 2,
            Setting: settingx1,
            bet: BetKey::Team_local,
            amount_bet: 500,
            gameid: game_id,
                                    collateralUsd: false,

        };


        client.bet(&addUser4, &betxx212);
                all_events.push(env.events().all());







                        let privateSettingx223 = PrivateBet {
            id: setting,
            gameid: game_id,
            active: false,
            settingAdmin: addUser1.clone(),
            description: String::from_str(&env, "Private Bet 1"),
            amount_bet_min: 500,
            users_invated: vec![&env, addUser1.clone(), addUser3.clone()],
        };
        client.set_private_bet(&addUser1, &privateSettingx223, &game_id);
                all_events.push(env.events().all());

        //let's bet to active the game
        let betx234 = Bet {
            id: 12,
            Setting: setting,
            bet: BetKey::Team_away,
            amount_bet: 500,
            gameid: game_id,
                                    collateralUsd: false,

        };
                        events_handler(env.clone(), all_events.clone());


        client.bet(&addUser1, &betx234);
        all_events.push(env.events().all());



        let betxx212 = Bet {
            id: 2,
            Setting: setting,
            bet: BetKey::Team_local,
            amount_bet: 500,
            gameid: game_id,
                                    collateralUsd: false,

        };


        client.bet(&addUser3, &betxx212);
                all_events.push(env.events().all());


        // Set ledger timestamp after game end
        set_ledger_timestamp(&env, 6501);
        
        let resultxx21 = ResultGame {
            id: 1,
            gameid: game_id,
            setting:setting,
            result: BetKey::Team_local,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };

        client.summitResult(&addUser3,&setting, &resultxx21);
        client.assessResult(&addUser1, &setting, &AssessmentKey::approve);
        client.execute_distribution( &setting);


        
        client.claim(&addUser3, &ClaimType::User, &setting);
        all_events.push(env.events().all());
        client.claim(&addUser1, &ClaimType::User, &setting);
                all_events.push(env.events().all());
                // Set ledger timestamp after game end        
        let resultxx21 = ResultGame {
            id: 1,
            gameid: game_id,
            setting:settingx,
            result: BetKey::Team_local,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };

        client.summitResult(&addUser3,&settingx, &resultxx21);
        client.assessResult(&addUser2, &settingx, &AssessmentKey::approve);
        client.execute_distribution( &settingx);
                client.claim(&addUser2, &ClaimType::User, &settingx);
        all_events.push(env.events().all());
        client.claim(&addUser3, &ClaimType::User, &settingx);
                all_events.push(env.events().all());
               // Set ledger timestamp after game end
        
        let resultxx21 = ResultGame {
            id: 1,
            gameid: game_id,
            setting:settingx1,
            result: BetKey::Team_local,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };
                let resultSupreme = ResultGame {
            id: 1,
            gameid: game_id,
            setting:0,
            result: BetKey::Team_away,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };
                        let resultSupremex = ResultGame {
            id: 1,
            gameid: game_id,
            setting:0,
            result: BetKey::Team_local,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };

        client.summitResult(&addUser1,&settingx1, &resultxx21);
        client.assessResult(&addUser4, &settingx1, &AssessmentKey::reject);
                        all_events.push(env.events().all());
                        events_handler(env.clone(), all_events.clone());
        //client.execute_distribution( &settingx1);

                //set_ledger_timestamp(&env, 8101);
        client.setResult_supremCourt(&admin, &resultSupreme);
        
        all_events.push(env.events().all());
        
        client.AssestResult_supremCourt(&addUser2, &game_id, &AssessmentKey::reject);
                client.AssestResult_supremCourt(&addUser3, &game_id, &AssessmentKey::reject);
                all_events.push(env.events().all());
                        events_handler(env.clone(), all_events.clone());

        client.setResult_supremCourt(&addUser2, &resultSupremex);
                //client.AssestResult_supremCourt(&admin, &game_id, &AssessmentKey::approve);
                client.AssestResult_supremCourt(&addUser3, &game_id, &AssessmentKey::approve);

                all_events.push(env.events().all());
                        events_handler(env.clone(), all_events.clone());


        
        client.claim(&addUser1, &ClaimType::User, &settingx1);
        all_events.push(env.events().all());
        //client.claim(&addUser4, &ClaimType::User, &settingx1);
                all_events.push(env.events().all());
        client.claim(&addUser2, &ClaimType::Supreme, &game_id);
                client.claim(&addUser3, &ClaimType::Supreme, &game_id);
                               // client.claim(&addUser3, &ClaimType::Supreme, &game_id);


               // client.claim(&addUser2, &ClaimType::User, &settingx);

  




       
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #220)")]
    fn test_set_error_result_supreme_court() {
        let (
            env,
            client,
            admin,
            key,
            pk,
            user,
            token_usd,
            token_trust,
            token_usd_client,
            token_trust_client,
            adm_usd,
            adm_trust,
        ) = create_test_env();
        //client.init(&admin, &token_usd, &token_trust);

        // Set up a game
        let game_id = 1;
        let game = Game {
            id: game_id,
            startTime: 1000,
            endTime: 2000,

            active: false,
            league: 1,
            description: String::from_slice(&env, "Team A vs Team B"),
            team_local: 33,
            team_away: 44,
        };
        // Encode game to Bytes (variable length)
        let encoded: Vec<u8> = game.clone().to_xdr(&env).iter().collect();


        let signaturex: BytesN<64> =
            BytesN::from_array(&env, &key.sign(encoded.as_slice()).to_bytes());
        client.set_game(&game, &signaturex, );
        //add the user who wna tto participate as a summiter
        let summiter = Address::generate(&env);
        let summiter2 = Address::generate(&env);
        adm_usd.mint(&summiter, &100_000_000);
        adm_usd.mint(&summiter2, &100_000_000);


        //let's bet to active the game
        let bet = Bet {
            id: 1,
            Setting: game_id,
            bet: BetKey::Team_local,
            amount_bet: 1000,
            gameid: game_id,
                                    collateralUsd: false,

        };
        client.bet(&user, &bet);
        let user2 = Address::generate(&env);
        adm_usd.mint(&user2, &100_000_000);
        adm_trust.mint(&user2, &100_000_000);

        let betx = Bet {
            id: 2,
            Setting: game_id,
            bet: BetKey::Team_away,
            amount_bet: 1000,
            gameid: game_id,
                                    collateralUsd: false,

        };
        client.bet(&user2, &betx);
        // Set ledger timestamp after game end
        set_ledger_timestamp(&env, 2500);
        struct SummitersSeletedEvent {
            game_id: i128,
            main: Address,
            summiters: Vec<Address>,
        }

        let result = ResultGame {
            id: 1,
            setting:22,
            gameid: game_id,
            result: BetKey::Team_local,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };
        let events = env.events().all();
        let (_, _, value) = events.get(0).unwrap();
        let raw: SorobanVec<Val> = value.try_into_val(&env).unwrap();

        let game_id: i128 = raw.get(0).unwrap().try_into_val(&env).unwrap();
        let main: Address = raw.get(1).unwrap().try_into_val(&env).unwrap();
        let summiters: SorobanVec<Address> = raw.get(2).unwrap().try_into_val(&env).unwrap();

        std::println!("Game ID: {}", game_id);
        std::println!("Main: {:?}", main);
        for s in summiters.iter() {
            std::println!("Summiter: {:?}", s);
        }


        //client.execute_distribution(&game_id);
        std::println!("User balance final {:?}", token_usd_client.balance(&user));
        std::println!("User2 balance final {:?}", token_usd_client.balance(&user2));
        std::println!(
            "summiter balance final {:?}",
            token_usd_client.balance(&summiter)
        );
        std::println!(
            "summiter2 balance final {:?}",
            token_usd_client.balance(&summiter2)
        );
        std::println!("admin balance final {:?}", token_usd_client.balance(&admin));
    }
    #[test]
    fn test_set_result_supreme_court() {
        let (
            env,
            client,
            admin,
            key,
            pk,
            user,
            token_usd,
            token_trust,
            token_usd_client,
            token_trust_client,
            adm_usd,
            adm_trust,
        ) = create_test_env();
        //client.init(&admin, &token_usd, &token_trust);

        // Set up a game
        let game_id = 1;
        let game = Game {
            id: game_id,
            startTime: 1000,
            endTime: 2000,

            active: false,
            league: 1,
            description: String::from_slice(&env, "Team A vs Team B"),
            team_local: 33,
            team_away: 44,
        };
        // Encode game to Bytes (variable length)
        let encoded: Vec<u8> = game.clone().to_xdr(&env).iter().collect();

        let signer1 = Keypair::generate(&mut thread_rng());
        let public_key = BytesN::<32>::from_array(&env, &signer1.public.to_bytes());

        let signaturex: BytesN<64> =
            BytesN::from_array(&env, &key.sign(encoded.as_slice()).to_bytes());
        client.set_game(&game, &signaturex, );
        //add the user who wna tto participate as a summiter
        // we dont need the rol any more


        //let's bet to active the game
        let bet = Bet {
            id: 1,
            Setting: game_id,
            bet: BetKey::Team_local,
            amount_bet: 1000,
            gameid: game_id,
                                    collateralUsd: false,

        };
        client.bet(&user, &bet);
        let user2 = Address::generate(&env);
        adm_usd.mint(&user2, &100_000_000);
        adm_trust.mint(&user2, &100_000_000);

        let betx = Bet {
            id: 2,
            Setting: game_id,
            bet: BetKey::Team_away,
            amount_bet: 1000,
            gameid: game_id,
                                    collateralUsd: false,

        };
        client.bet(&user2, &betx);
        // Set ledger timestamp after game end
        set_ledger_timestamp(&env, 2500);

        let result = ResultGame {
            id: 1,
            setting:22,
            gameid: game_id,
            result: BetKey::Team_local,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };
        let result2 = ResultGame {
            id: 1,
            setting:22,
            gameid: game_id,
            result: BetKey::Team_local,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };


        client.claim(&user2, &ClaimType::User, &game_id);
        client.claim(&admin, &ClaimType::Protocol, &game_id);
        std::println!("User balance final {:?}", token_usd_client.balance(&user));
        std::println!("User2 balance final {:?}", token_usd_client.balance(&user2));

        std::println!("admin balance final {:?}", token_usd_client.balance(&admin));
    }
    #[test]
    fn test_set_result_supreme_court_claim() {
        let (
            env,
            client,
            admin,
            key,
            pk,
            user,
            token_usd,
            token_trust,
            token_usd_client,
            token_trust_client,
            adm_usd,
            adm_trust,
        ) = create_test_env();
        //client.init(&admin, &token_usd, &token_trust);

        // Set up a game
        let game_id = 1;
        let game = Game {
            id: game_id,
            startTime: 1000,
            endTime: 2000,

            active: false,
            league: 1,
            description: String::from_slice(&env, "Team A vs Team B"),
            team_local: 33,
            team_away: 44,
        };
        // Encode game to Bytes (variable length)
        let encoded: Vec<u8> = game.clone().to_xdr(&env).iter().collect();

        let signer1 = Keypair::generate(&mut thread_rng());
        let public_key = BytesN::<32>::from_array(&env, &signer1.public.to_bytes());

        let signaturex: BytesN<64> =
            BytesN::from_array(&env, &key.sign(encoded.as_slice()).to_bytes());
        client.set_game(&game, &signaturex, );
        //add the user who wna tto participate as a summiter
        let summiter = Address::generate(&env);
        let summiter2 = Address::generate(&env);
        adm_usd.mint(&summiter, &100_000_000);
        adm_usd.mint(&summiter2, &100_000_000);


        //let's bet to active the game
        let bet = Bet {
            id: 1,
            Setting: game_id,
            bet: BetKey::Team_away,
            amount_bet: 1000,
            gameid: game_id,
                                    collateralUsd: false,

        };
        let initial_usd_balance = token_usd_client.balance(&user);
        let initial_trust_balance = token_trust_client.balance(&user);
        client.bet(&user, &bet);
        let user2 = Address::generate(&env);
        adm_usd.mint(&user2, &100_000_000);
        adm_trust.mint(&user2, &100_000_000);

        let betx = Bet {
            id: 2,
            Setting: game_id,
            bet: BetKey::Team_local,
            amount_bet: 1000,
            gameid: game_id,
                                    collateralUsd: false,

        };
        client.bet(&user2, &betx);
        // Set ledger timestamp after game end
        set_ledger_timestamp(&env, 2500);

        let result = ResultGame {
            id: 1,
            setting:22,
            gameid: game_id,
            result: BetKey::Team_local,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };
        let result2 = ResultGame {
            id: 1,
            setting:22,
            gameid: game_id,
            result: BetKey::Team_away,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };

        // Execute distribution

        client.claim(&user, &ClaimType::User, &game_id);
        client.claim(&user2, &ClaimType::User, &game_id);
        client.claim(&admin, &ClaimType::Protocol, &game_id);
        std::println!("User balance final {:?}", token_usd_client.balance(&user));
        std::println!("User2 balance final {:?}", token_usd_client.balance(&user2));
        std::println!(
            "summiter balance final {:?}",
            token_usd_client.balance(&summiter)
        );
        std::println!(
            "summiter2 balance final {:?}",
            token_usd_client.balance(&summiter2)
        );
        std::println!("admin balance final {:?}", token_usd_client.balance(&admin));
        // Verify token transfers (winner gets bet + share of pool)
        assert!(token_usd_client.balance(&user) > initial_usd_balance);
        assert_eq!(token_trust_client.balance(&user), initial_trust_balance); // Trust tokens returned
    }
    #[test]
    fn test_set_private() {
        let (
            env,
            client,
            admin,
            key,
            pk,
            user,
            token_usd,
            token_trust,
            token_usd_client,
            token_trust_client,
            adm_usd,
            adm_trust,
        ) = create_test_env();
        //client.init(&admin, &token_usd, &token_trust);

        // Set up a game
        let game_id = 1;
        let game = Game {
            id: game_id,
            startTime: 1000,
            endTime: 2000,

            active: false,
            league: 1,
            description: String::from_slice(&env, "Team A vs Team B"),
            team_local: 33,
            team_away: 44,
        };
        // Encode game to Bytes (variable length)
        let encoded: Vec<u8> = game.clone().to_xdr(&env).iter().collect();

        let signer1 = Keypair::generate(&mut thread_rng());
        let public_key = BytesN::<32>::from_array(&env, &signer1.public.to_bytes());

        let signaturex: BytesN<64> =
            BytesN::from_array(&env, &key.sign(encoded.as_slice()).to_bytes());
        client.set_game(&game, &signaturex, );

        let user2 = Address::generate(&env);

        let privateSetting = PrivateBet {
            id: 11,
            gameid: game_id,
            active: false,
            settingAdmin: user2.clone(),
            description: String::from_str(&env, "Private Bet 1"),
            amount_bet_min: 500,
            users_invated: vec![&env, user.clone(), user2.clone()],
        };
        client.set_private_bet(&user2, &privateSetting, &game_id);
        //let's bet to active the game
        let bet = Bet {
            id: 1,
            Setting: 11,
            bet: BetKey::Team_away,
            amount_bet: 500,
            gameid: game_id,
                                    collateralUsd: false,

        };
        let initial_usd_balance = token_usd_client.balance(&user);
        let initial_trust_balance = token_trust_client.balance(&user);
        client.bet(&user, &bet);
        adm_usd.mint(&user2, &100_000_000);
        adm_trust.mint(&user2, &100_000_000);

        let betx = Bet {
            id: 2,
            Setting: 11,
            bet: BetKey::Team_local,
            amount_bet: 500,
            gameid: game_id,
                                    collateralUsd: false,

        };
        let initial_usd_balance2 = token_usd_client.balance(&user2);
        let initial_trust_balance2 = token_trust_client.balance(&user2);
        client.bet(&user2, &betx);
        // Set ledger timestamp after game end
        set_ledger_timestamp(&env, 2500);

        let result = ResultGame {
            id: 1,
            setting:11,
            gameid: game_id,
            result: BetKey::Team_away,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };


        //client.setResult_supremCourt(&user, &result2);
        // Execute distribution

        client.claim(&user, &ClaimType::User, &11);
        client.claim(&user2, &ClaimType::User, &11);

        client.claim(&admin, &ClaimType::Protocol, &game_id);

        // Verify token transfers (winner gets bet + share of pool)
        std::println!("User balance final {:?}", token_usd_client.balance(&user));
        std::println!("User2 balance final {:?}", token_usd_client.balance(&user2));

        std::println!("admin balance final {:?}", token_usd_client.balance(&admin));

        // Verify token transfers (winner gets bet + share of pool)
        assert!(token_usd_client.balance(&user) > initial_usd_balance);
        assert_eq!(token_trust_client.balance(&user), initial_trust_balance); // Trust tokens returned
        assert!(token_usd_client.balance(&user2) < initial_usd_balance2);
        assert_eq!(token_trust_client.balance(&user2), initial_trust_balance2); // Trust tokens returned
    }
    #[test]
    fn test_set_private_x() {
        let (
            env,
            client,
            admin,
            key,
            pk,
            user,
            token_usd,
            token_trust,
            token_usd_client,
            token_trust_client,
            adm_usd,
            adm_trust,
        ) = create_test_env();
        //client.init(&admin, &token_usd, &token_trust);

        let mut all_events = Vec::new();

        // Set up a game
        let game_id = 51;
        let game = Game {
            id: game_id,
            startTime: 1000,
            endTime: 2000,
            active: false,
            league: 1,
            description: String::from_slice(&env, "Team A vs Team B"),
            team_local: 33,
            team_away: 44,
        };
        // Encode game to Bytes (variable length)
        let encoded: Vec<u8> = game.clone().to_xdr(&env).iter().collect();

        let signer1 = Keypair::generate(&mut thread_rng());
        let public_key = BytesN::<32>::from_array(&env, &signer1.public.to_bytes());

        let signaturex: BytesN<64> =
            BytesN::from_array(&env, &key.sign(encoded.as_slice()).to_bytes());
        client.set_game(&game, &signaturex, );
        all_events.push(env.events().all());

        //add the user who wna tto participate as a summiter
        let summiter = Address::generate(&env);
        let summiter2 = Address::generate(&env);
        std::println!("summiter address: {:?}", summiter);
        std::println!("summiter2 address: {:?}", summiter2);
        adm_usd.mint(&summiter, &1000);
        adm_usd.mint(&summiter2, &1000);
        std::println!(
            "summiter balance initial {:?}",
            token_usd_client.balance(&summiter)
        );
        std::println!(
            "summiter2 balance initial {:?}",
            token_usd_client.balance(&summiter2)
        );

        let user2 = Address::generate(&env);

        let privateSetting = PrivateBet {
            id: 11,
            gameid: game_id,
            active: false,
            settingAdmin: user2.clone(),
            description: String::from_str(&env, "Private Bet 1"),
            amount_bet_min: 500,
            users_invated: vec![&env, user.clone(), user2.clone()],
        };
        client.set_private_bet(&user2, &privateSetting, &game_id);
        all_events.push(env.events().all());

        //let's bet to active the game
        let bet = Bet {
            id: 1,
            Setting: 11,
            bet: BetKey::Team_away,
            amount_bet: 500,
            gameid: game_id,
                                    collateralUsd: false,

        };

        let initial_usd_balance = token_usd_client.balance(&user);
        let initial_trust_balance = token_trust_client.balance(&user);
        std::println!("User1 honest balance initial {:?}", initial_usd_balance);
        client.bet(&user, &bet);
        all_events.push(env.events().all());

        adm_usd.mint(&user2, &100_000_000);
        adm_trust.mint(&user2, &100_000_000);

        let betx = Bet {
            id: 2,
            Setting: 11,
            bet: BetKey::Team_local,
            amount_bet: 500,
            gameid: game_id,
                                    collateralUsd: false,

        };
        let initial_usd_balance = token_usd_client.balance(&user2);
        std::println!("User2 novote balance initial {:?}", initial_usd_balance);

        client.bet(&user2, &betx);
        all_events.push(env.events().all());

        // Set ledger timestamp after game end
        set_ledger_timestamp(&env, 2500);

        let result = ResultGame {
            id: 1,
            setting:11,
            gameid: game_id,
            result: BetKey::Team_local,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };
        let result2 = ResultGame {
            id: 1,
            setting:11,
            gameid: game_id,
            result: BetKey::Team_away,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };
        //client.summitResult(&summiter, &result);
        all_events.push(env.events().all());

        //client.assessResult(&user, &11, &game_id, &AssessmentKey::reject);
        all_events.push(env.events().all());

                //client.assessResult(&user2, &11, &game_id, &AssessmentKey::reject);
        all_events.push(env.events().all());
        //client.assessResult(&summiter, &0, &game_id, &AssessmentKey::reject);
                set_ledger_timestamp(&env, 20500);

        //client.setResult_supremCourt(&result2);
        all_events.push(env.events().all());

        // Execute distribution

        client.claim(&user, &ClaimType::User, &11);
        all_events.push(env.events().all());

        //client.claim(&user2, &ClaimType::User, &11);

        client.claim(&admin, &ClaimType::Protocol, &game_id);
        all_events.push(env.events().all());

        // Verify token transfers (winner gets bet + share of pool)
        events_handler(env.clone(), all_events);
        std::println!("User balance final {:?}", token_usd_client.balance(&user));
        std::println!("User2 balance final {:?}", token_usd_client.balance(&user2));
        std::println!(
            "summiter balance initial {:?}",
            token_usd_client.balance(&summiter)
        );
        std::println!(
            "summiter2 balance initial {:?}",
            token_usd_client.balance(&summiter2)
        );
        std::println!(
            "admin balance initial {:?}",
            token_usd_client.balance(&admin)
        );
        assert!(token_usd_client.balance(&user) > initial_usd_balance);
        assert_eq!(token_trust_client.balance(&user), initial_trust_balance); // Trust tokens returned
    }
}

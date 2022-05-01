use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::env::attached_deposit;
use near_sdk::{env, near_bindgen, PanicOnDefault, ext_contract, AccountId, Gas, Promise, PromiseResult};
use near_contract_standards::non_fungible_token::Token;
use std::str::FromStr;

pub const MINT_GAS: Gas = Gas(30_000_000_000_000);
pub const GET_NFTS_GAS: Gas = Gas(30_000_000_000_000);
pub const INVALIDATE_NFTS_GAS: Gas = Gas(50_000_000_000_000);
pub const PROMISE_CHAIN_REMAIN: Gas = Gas(10_000_000_000_000);
pub const NFT_CONTRACT: &str = "kawaii-zoo-nft.cryptosketches.testnet";
pub const DONATION_ACCOUNT: &str = "kawaii-zoo-donation.cryptosketches.testnet";
pub const DONATION_PERCENTAGE: u128 = 20;

#[ext_contract(ext_nft_contract)]
trait NftContract {
    fn nft_mint(&self, token_owner_id: AccountId) -> Token;
    //fn test_mint(&self, token_owner_id: AccountId, index : u64) -> Token;
    fn get_nfts(&self, nft_ids: Vec<u64>) -> Vec<Token>;
    fn invalidate_nfts(&self, nft_ids: Vec<u64>);
}

#[ext_contract(ext_self)]
pub trait ExtSelf {
    fn check_set_callback(owner_id: AccountId, nft_set: Vec<u64>);
    fn invalidate_callback(payout_id: AccountId);
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {
        }
    }
    
    #[payable]
    pub fn buy_animal(
        &mut self,
        count: u8
    ) {   
        assert!(count <= 7, "You can buy a maximum of 7 animals at a time because of the gas limits.");
        assert!(attached_deposit() >= (count as u128)*10u128.pow(24), "Each animal costs 1 NEAR. Please attach enough NEAR.");
        let mut promise: Option<Promise> = None;
        for _ in 0..count{
            match promise{
                Some(old) => promise = Some(old.then(ext_nft_contract::nft_mint(
                    env::predecessor_account_id(), 
                    AccountId::from_str(NFT_CONTRACT).unwrap(), 
                    env::attached_deposit(),
                    MINT_GAS
                ))),
                None => promise = Some(ext_nft_contract::nft_mint(
                    env::predecessor_account_id(), 
                    AccountId::from_str(NFT_CONTRACT).unwrap(), 
                    env::attached_deposit(),
                    MINT_GAS
                ))
            }
        }
        let donation = attached_deposit()/100*DONATION_PERCENTAGE;
        Promise::new(AccountId::from_str(DONATION_ACCOUNT).unwrap()).transfer(donation);
        let donation_in_near = (donation as f64)/(10u128.pow(24) as f64);
        env::log_str(format!("Successfully minted {} kawaii animals. Donated {} NEAR to Ukrainian zoos.", count, donation_in_near).as_str());
    }
    
    pub fn payout(
        &mut self,
        nft_set: Vec<u64>
    ) {   
        assert!(nft_set.len() == 5, "A set consists of exactly 5 NFTs.");
        
        let get_nft_call = ext_nft_contract::get_nfts(
            nft_set.clone(), 
            AccountId::from_str(NFT_CONTRACT).unwrap(), 
            0, 
            GET_NFTS_GAS
        );
        let remaining_gas: Gas = env::prepaid_gas() - env::used_gas() - GET_NFTS_GAS - PROMISE_CHAIN_REMAIN;
        let callback = ext_self::check_set_callback(
            env::predecessor_account_id(), 
            nft_set,
            env::current_account_id(), 
            0, 
            remaining_gas
        );
        get_nft_call.then(callback);
    }

    #[private]
    pub fn invalidate_callback(
        &mut self,
        payout_id: AccountId
    ){
        assert_eq!(
            env::promise_results_count(),
            1,
            "This is a callback method"
        );

        // handle the result from the cross contract call this method is a callback for
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => env::panic_str("NFT invalidation failed."),
            PromiseResult::Successful(_) => {
                let payout = (env::account_balance() - 100u128*10u128.pow(24))/2u128;
                if payout > 1 {
                    let payout_in_near = (payout as f64)/(10u128.pow(24) as f64);
                    env::log_str(format!("Congratulations. You just won {} NEAR", payout_in_near).as_str());
                    Promise::new(payout_id).transfer(payout);
                }
                else{
                    env::panic_str("Game over. The game contract has no balance left. Sry.")
                }
            },
        }
    }

    #[private]
    pub fn check_set_callback(
        &mut self,
        owner_id: AccountId,
        nft_set: Vec<u64>,
        #[callback_unwrap] tokens: Vec<Token>
    ){
        assert!(tokens.iter().all(|token| token.owner_id == owner_id), "Not all tokens belong to the calling account. Only tokens owned by the calling account can be used.");
        assert!(self.check_is_valid_set(tokens), "The given NFTs are no valid set.");
        let invalidate_nft_call = ext_nft_contract::invalidate_nfts(
            nft_set, 
            AccountId::from_str(NFT_CONTRACT).unwrap(), 
            0, 
            INVALIDATE_NFTS_GAS
        );
        let remaining_gas: Gas = env::prepaid_gas() - env::used_gas() - INVALIDATE_NFTS_GAS - PROMISE_CHAIN_REMAIN;
        let callback = ext_self::invalidate_callback(
            owner_id, 
            env::current_account_id(), 
            0, 
            remaining_gas);
        invalidate_nft_call.then(callback);
    }

    fn check_is_valid_set(&self, tokens: Vec<Token>) -> bool{
        let types = tokens.iter().map(|token| token.metadata.as_ref().unwrap().extra.as_ref().unwrap().as_str());
        let folded = types.fold(String::new(), |a, b| a + ", " + b);
        env::log_str(format!("The given NFTs have the following types: {}", folded).as_str());
        self.check_is_set(&tokens, ["11".into(), "12".into(), "13".into(), "14".into(), "15".into()]) ||
        self.check_is_set(&tokens, ["21".into(), "22".into(), "23".into(), "24".into(), "25".into()]) ||
        self.check_is_set(&tokens, ["31".into(), "32".into(), "33".into(), "34".into(), "35".into()]) ||
        self.check_is_set(&tokens, ["41".into(), "42".into(), "43".into(), "44".into(), "45".into()]) ||
        self.check_is_set(&tokens, ["51".into(), "52".into(), "53".into(), "54".into(), "55".into()]) ||
        self.check_is_set(&tokens, ["11".into(), "21".into(), "31".into(), "41".into(), "51".into()]) ||
        self.check_is_set(&tokens, ["12".into(), "22".into(), "32".into(), "42".into(), "52".into()]) ||
        self.check_is_set(&tokens, ["13".into(), "23".into(), "33".into(), "43".into(), "53".into()]) ||
        self.check_is_set(&tokens, ["14".into(), "24".into(), "34".into(), "44".into(), "54".into()]) ||
        self.check_is_set(&tokens, ["15".into(), "25".into(), "35".into(), "45".into(), "55".into()])
    }

    fn check_is_set(&self, tokens: &Vec<Token>, set: [String; 5]) -> bool{
        for set_part in set.iter(){
            if tokens.iter().all(|token| token.metadata.as_ref().unwrap().extra.as_ref().unwrap().to_string() != set_part.to_string()) {
                return false
            }
        }
        true
    }
}

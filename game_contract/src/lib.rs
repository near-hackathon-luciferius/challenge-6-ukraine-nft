use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::env::attached_deposit;
use near_sdk::{env, near_bindgen, PanicOnDefault, ext_contract, AccountId, Gas, Promise};
use std::str::FromStr;

pub const MINT_GAS: Gas = Gas(30_000_000_000_000);

#[ext_contract(ext_nft_mint)]
trait MintContract {
    fn nft_mint(&self, token_owner_id: AccountId) -> Token;
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
        assert!(count <= 10, "You can buy a maximum of 10 animals at a time.");
        assert!(attached_deposit() >= (count as u128)*10u128.pow(24), "Each animal costs 1 NEAR. Please attach enough NEAR.");
        let mut promise: Option<Promise> = None;
        for _ in 0..count{
            match promise{
                Some(old) => promise = Some(old.then(ext_nft_mint::nft_mint(
                    env::predecessor_account_id(), 
                    AccountId::from_str("kawaii-zoo-nft.cryptosketches.testnet").unwrap(), 
                    env::attached_deposit(),
                    MINT_GAS
                ))),
                None => promise = Some(ext_nft_mint::nft_mint(
                    env::predecessor_account_id(), 
                    AccountId::from_str("kawaii-zoo-nft.cryptosketches.testnet").unwrap(), 
                    env::attached_deposit(),
                    MINT_GAS
                ))
            }
        }
    }
}

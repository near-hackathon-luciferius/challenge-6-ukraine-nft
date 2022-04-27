use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::env::attached_deposit;
use near_sdk::{env, near_bindgen, PanicOnDefault, ext_contract, AccountId, Gas, Promise, PromiseResult, require};
use near_contract_standards::non_fungible_token::Token;
use std::str::FromStr;

pub const MINT_GAS: Gas = Gas(30_000_000_000_000);
pub const APPROVE_CALLBACK_GAS: Gas = Gas(10_000_000_000_000);
pub const APPROVAL_GAS: Gas = Gas(20_000_000_000_000);

#[ext_contract(ext_nft_mint)]
trait MintContract {
    fn nft_mint(&self, token_owner_id: AccountId) -> Token;
}

#[ext_contract(ext_nft_approval)]
trait NonFungibleTokenApprovalManagement: NonFungibleToken {
    fn nft_approve(&mut self, token_id: String, account_id: String, msg: Option<String>);
}

//callbacks
#[ext_contract(ext_self)]
trait SelfContract {
    fn start_approval(&self) -> Promise;
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
            let next_promise = ext_nft_mint::nft_mint(
                env::predecessor_account_id(), 
                AccountId::from_str("kawaii-zoo-nft.cryptosketches.testnet").unwrap(), 
                env::attached_deposit(),
                MINT_GAS
            ).then(
                ext_self::start_approval(env::current_account_id(), 0, APPROVE_CALLBACK_GAS)
            );
            match promise{
                Some(old) => promise = Some(old.then(next_promise)),
                None => promise = Some(next_promise)
            }
        }
    }

    pub fn start_approval(&self) -> Promise {
        require!(env::predecessor_account_id() == env::current_account_id(), "Method is private");
        assert_eq!(
            env::promise_results_count(),
            1,
            "This is a callback method"
        );

        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => panic!("Previous minting failed. Stopping loop."),
            PromiseResult::Successful(result) => {
                let minted_token = near_sdk::serde_json::from_slice::<Token>(&result).unwrap();
                ext_nft_approval::nft_approve(
                    minted_token.token_id, 
                    env::current_account_id().to_string(), 
                    Some("Auto approve game for NFT.".into()), 
                    AccountId::from_str("kawaii-zoo-nft.cryptosketches.testnet").unwrap(), 
                    0, 
                    APPROVAL_GAS
                )
            },
        }
    }
}

#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Env, Address};

#[test]
fn test() {
    let env = Env::default();
    let test_admin = Address::generate(&env);
    let contract_id = env.register(RampContract, (test_admin, 10u32));
    let client = RampContractClient::new(&env, &contract_id);

    client.add_asset( &Address::generate(&env), &10u128);
}

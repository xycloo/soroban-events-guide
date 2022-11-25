#![cfg(test)]

use super::{EventsContract, EventsContractClient};

use soroban_sdk::Env;

#[test]
fn test_types() {
    let env = Env::default();

    let contract_id = env.register_contract(None, EventsContract);
    let client = EventsContractClient::new(&env, contract_id);

    let (admin_id, _) = soroban_auth::testutils::ed25519::generate(&env);

    client.init(&admin_id);
}

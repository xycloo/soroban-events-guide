#![no_std]

#[cfg(feature = "testutils")]
extern crate std;

mod test;

use soroban_auth::Identifier;
use soroban_sdk::{contractimpl, serde::Serialize, symbol, Env};

/// Contract trait
pub trait EventsContractTrait {
    fn init(e: Env, admin: Identifier);
}

pub struct EventsContract;

#[contractimpl]
impl EventsContractTrait for EventsContract {
    fn init(e: Env, admin: Identifier) {
        let event = e.events();
        let t1 = (symbol!("init"),);

        let id_bytes = admin.serialize(&e);
        event.publish(t1, (id_bytes,));
    }
}

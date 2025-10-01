mod account;
mod masm;

use account::{account_to_json, noop_auth_component::NoAuthComponent};
use masm::{run, run_prove, run_verify};
use miden_lib::account::{auth::AuthRpoFalcon512, wallets::BasicWallet};
use miden_objects::{account::{Account, AccountBuilder, AccountComponent, AccountId}, assembly::Library, crypto::{dsa::rpo_falcon512::PublicKey, rand::Randomizable}};
use miden_vm::Word;

use crate::account::add_component::AddComponent;

fn main() {
    // run();
    // let (outputs, proof) = run_prove().unwrap();
    // run_verify(outputs, proof).unwrap();

    let public_key = PublicKey::new(Word::from_random_bytes(&[0; 32]).unwrap());

    let (account, _) = AccountBuilder::new([0xff; 32])
        .with_auth_component(AuthRpoFalcon512::new(public_key))
        .with_component(BasicWallet)
        .build()
        .unwrap();
    account_to_json(account);
}

#![cfg_attr(not(test), no_main)]

mod error;
use error::CircomError;
use casper_contract::contract_api::{circom::circom_verifier, runtime, storage};
use casper_types::{CLType, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Key, contracts::NamedKeys};

#[no_mangle]
pub extern "C" fn call_verifier(){
    let circuit_payload = include_bytes!("../circuit.pem");
    let proof_payload = include_bytes!("../proof.pem");
    if circom_verifier(proof_payload, circuit_payload) != [1]{
        runtime::revert(CircomError::InvalidProof);
    }
}

#[no_mangle]
pub extern "C" fn call(){
    // entry point definitions
    let mut entry_points: EntryPoints = EntryPoints::new();
    let call_verifier: EntryPoint = EntryPoint::new(
        "call_verifier",
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract
    );
    entry_points.add_entry_point(call_verifier);
    // named keys definitions
    let mut named_keys = NamedKeys::new();
    // contract package
    let package_key_name = "circom_multiplier_contract".to_string();
    let (contract_hash, _) = storage::new_contract(
        entry_points,
        Some(named_keys),
        Some(package_key_name),
        Some("circom_multiplier_contract".to_string()),
    );
    let contract_hash_key = Key::from(contract_hash);
    // store contract package key
    runtime::put_key("circom_multiplier_contract", contract_hash_key);
}

#[cfg(test)]
mod tests{
    #[test]
    #[cfg(not(target_arch = "wasm32"))] 
    #[cfg(feature = "casper-circom")]
    fn generate_full_circom_payload(){
        use casper_circom::{CircomInput, generator, generator::CircomGenerator};
        use std::path::PathBuf;
        let mut generator = CircomGenerator{
            wasm: PathBuf::from("/users/chef/Desktop/circom-cli/casper-circom/circom/multiplier/multiplier.wasm"),
            r1cs: PathBuf::from("/users/chef/Desktop/circom-cli/casper-circom/circom/multiplier/multiplier.r1cs"),
            proof_out: PathBuf::from("proof.pem"),
            circuit_out: PathBuf::from("circuit.pem"),
            private_inputs: Vec::new(),
            public_inputs: vec![("a".to_string(), 2), ("b".to_string(), 20), ("c".to_string(), 40)]
        };
    
        /*let input = generator.generate_input();
        println!(
            "alpha_g1: {:?}, beta_g2: {:?}, delta_g2: {:?}, gamma_g2: {:?}, gamma_abc_g1: {:?}, a: {:?}, b: {:?}, c: {:?}, inputs: {:?}",
            &input.alpha_g1,
            &input.beta_g2,
            &input.delta_g2,
            &input.gamma_g2,
            &input.gamma_abc_g1,
            &input.a,
            &input.b,
            &input.c,
            &input.inputs
        );*/
        generator.dump_input();
        generator.dump_circuit();
    }    
}
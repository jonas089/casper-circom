use crate::CircomProof;
use std::path::PathBuf;

// fs and io
use serde_json;

// native ark imports
use ark_groth16::{Groth16, ProvingKey};
use ark_crypto_primitives::snark::SNARK;
use ark_ec::bls12::Bls12;
use ark_circom::{CircomConfig, CircomBuilder, CircomCircuit };
use ark_bls12_377::{Bls12_377, Config};
use ark_serialize::CanonicalSerialize;
type GrothBls = Groth16<Bls12_377>;

pub struct CircomGenerator{
    pub wasm: PathBuf,
    pub r1cs: PathBuf,
    pub proof_out: PathBuf,
    pub private_inputs: Vec<(String, i32)>,
    pub public_inputs: Vec<(String, i32)>
}

impl CircomGenerator{
    // generate CircomInput serialized
    pub fn generate_input(&mut self) -> CircomProof{
        // load circuit from files
        let cfg: CircomConfig<Bls12<Config>> = CircomConfig::<Bls12_377>::new(
            &self.wasm,
            &self.r1cs 
        ).expect("Missing Circuit file(s)!");
        // construct inputs
        let mut builder: CircomBuilder<Bls12<Config>> = CircomBuilder::new(cfg);
        // push public inputs
        for public_input in &self.public_inputs{
            builder.push_input(&public_input.0, public_input.1);
        };
        // push private inputs
        for private_input in &self.private_inputs{
            builder.push_input(&private_input.0, private_input.1);
        }
        let circom: CircomCircuit<Bls12<Config>> = builder.setup();
        // run a trusted setup
        let mut rng: rand::prelude::ThreadRng = rand::thread_rng();
        let params: ProvingKey<Bls12<Config>> = GrothBls::generate_random_parameters_with_reduction(circom, &mut rng).unwrap();       
        // generate proof
        let circom: CircomCircuit<Bls12<Config>> = builder.build().unwrap();
        let proof: ark_groth16::Proof<Bls12<Config>> = GrothBls::prove(&params, circom.clone(), &mut rng).unwrap();
        // obtain verifying key
        let pvk: ark_groth16::PreparedVerifyingKey<Bls12<Config>> = GrothBls::process_vk(&params.vk).unwrap();
        // serialize public inputs
        let public_inputs = circom.get_public_inputs().unwrap();
        let mut serialized_inputs = Vec::new();
        let _ = public_inputs.iter().map(|input| input.0.serialize_uncompressed(&mut serialized_inputs));
        // serialize the proof
        let mut serialized_proof: Vec<u8> = Vec::new();
        let _ = proof.serialize_uncompressed(&mut serialized_proof);
        // serialize the vk
        let mut serialized_vk: Vec<u8> = Vec::new();
        let _ = pvk.serialize_uncompressed(&mut serialized_vk);
        // return CircomProof instance
        CircomProof{
            vk: serialized_vk,
            proof: serialized_proof,
            inputs: serialized_inputs
        }

    }

    pub fn dump_input(&mut self){
        let inputs: CircomProof = self.generate_input();
        let buffer: Vec<u8> = serde_json::to_vec(&inputs).unwrap();
        std::fs::write(&self.proof_out, buffer).expect("Failed to write proof!");
    }
}
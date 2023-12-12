use crate::{CircomInput, CircomCircuitInput};
use std::path::PathBuf;

// fs and io
use serde_json;

// native ark imports
use ark_groth16::{Groth16, ProvingKey};
use ark_crypto_primitives::snark::SNARK;
use ark_ec::bn::Bn;
use ark_circom::{CircomConfig, CircomBuilder };
use ark_bn254::Bn254;
use ark_serialize::CanonicalSerialize;

// custom circom types (Groth16)
type GrothBn = Groth16<Bn254>;

pub struct CircomGenerator{
    pub wasm: PathBuf,
    pub r1cs: PathBuf,
    pub proof_out: PathBuf,
    pub circuit_out: PathBuf,
    pub private_inputs: Vec<(String, i32)>,
    pub public_inputs: Vec<(String, i32)>
}

impl CircomGenerator{
    // generate CircomCircuitInput serialized
    pub fn generate_circuit(&mut self) -> CircomCircuitInput{
        let wasm_buffer: Vec<u8> = std::fs::read(&self.wasm).unwrap();
        let r1cs_buffer: Vec<u8> = std::fs::read(&self.r1cs).unwrap();
        CircomCircuitInput { 
            circuit_wasm: wasm_buffer, circuit_r1cs: r1cs_buffer
        }
    }

    pub fn dump_circuit(&mut self){
        let circuit: CircomCircuitInput = self.generate_circuit();
        let buffer: Vec<u8> = serde_json::to_vec(&circuit).unwrap();
        std::fs::write(&self.circuit_out, buffer).expect("Failed to write Circuit!");
    }

    // generate CircomInput serialized
    pub fn generate_input(&mut self) -> CircomInput{
        let cfg: CircomConfig<Bn<ark_bn254::Config>> = CircomConfig::<Bn254>::new(
            &self.wasm,
            &self.r1cs
        )
        .expect("Failed to read wasm and r1cs from path!");
        let mut builder = CircomBuilder::new(cfg);
        // push private inputs
        for input in self.private_inputs.clone(){
            builder.push_input(input.0, input.1);
        };
        // push public inputs
        for input in self.public_inputs.clone(){
            builder.push_input(input.0, input.1)
        };
        let circom: ark_circom::CircomCircuit<Bn<ark_bn254::Config>> = builder.setup();
        let mut rng: rand::prelude::ThreadRng = rand::thread_rng();
        let params: ProvingKey<Bn<ark_bn254::Config>> = GrothBn::generate_random_parameters_with_reduction(circom, &mut rng).unwrap();
        let circom: ark_circom::CircomCircuit<Bn<ark_bn254::Config>> = builder.build().unwrap();

        let proof: ark_groth16::Proof<Bn<ark_bn254::Config>> = GrothBn::prove(&params, circom, &mut rng).unwrap();
        
        // Verifying key
        let mut alpha_g1_writer: Vec<u8> = Vec::new();
        let mut beta_g2_writer: Vec<u8> = Vec::new();
        let mut delta_g2_writer: Vec<u8> = Vec::new();
        let mut gamma_g2_writer: Vec<u8> = Vec::new();
        let mut gamma_abc_g1_list: Vec<Vec<u8>> = Vec::new();
        
        let _ = params.clone().vk.alpha_g1.serialize_uncompressed(&mut alpha_g1_writer);
        let _ = params.clone().vk.beta_g2.serialize_uncompressed(&mut beta_g2_writer);
        let _ = params.clone().vk.delta_g2.serialize_uncompressed(&mut delta_g2_writer);
        let _ = params.clone().vk.gamma_g2.serialize_uncompressed(&mut gamma_g2_writer);
        for gamma_abc in params.clone().vk.gamma_abc_g1{
            let mut gamma_abc_g1_writer: Vec<u8> = Vec::new();
            let _ = gamma_abc.serialize_uncompressed(&mut gamma_abc_g1_writer);
            gamma_abc_g1_list.push(gamma_abc_g1_writer);
        };
        // Proof
        let mut a_writer: Vec<u8> = Vec::new();
        let mut b_writer: Vec<u8> = Vec::new();
        let mut c_writer: Vec<u8> = Vec::new();
        let _ = proof.clone().a.serialize_uncompressed(&mut a_writer);
        let _ = proof.clone().b.serialize_uncompressed(&mut b_writer);
        let _ = proof.clone().c.serialize_uncompressed(&mut c_writer);

        CircomInput {
            alpha_g1: alpha_g1_writer,
            beta_g2: beta_g2_writer,
            delta_g2: delta_g2_writer,
            gamma_g2: gamma_g2_writer,
            gamma_abc_g1: gamma_abc_g1_list,
            a: a_writer,
            b: b_writer,
            c: c_writer,
            inputs: self.public_inputs.to_vec()
        }
    }

    pub fn dump_input(&mut self){
        let inputs: CircomInput = self.generate_input();
        let buffer: Vec<u8> = serde_json::to_vec(&inputs).unwrap();
        std::fs::write(&self.proof_out, buffer).expect("Failed to write proof!");
    }
}
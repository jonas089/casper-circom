use crate::CircomInput;
use std::path::PathBuf;

// fs and io
use std::io::{self, Read};
use std::fs::File;
use serde_json;
use serde::{Serialize, Deserialize};
use std::fs;

// native ark imports
use ark_groth16::{Groth16, ProvingKey};
use ark_crypto_primitives::snark::SNARK;
use ark_ec::{
    bn::Bn
};
use ark_circom::ethereum::{Proof, VerifyingKey};
use ark_circom::{CircomConfig, CircomBuilder, CircomCircuit};
use ark_bn254::{Bn254, Config, G1Affine, G2Affine};
use ark_circom::{circom::R1CSFile, WitnessCalculator};
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize, Write};

// custom circom types (Groth16)
type GrothBn = Groth16<Bn254>;


pub struct CircomGenerator{
    pub wasm: PathBuf,
    pub r1cs: PathBuf,
    pub output: PathBuf,
    pub private_inputs: Vec<(String, i32)>,
    pub public_inputs: Vec<(String, i32)>
}

impl CircomGenerator{
    // generate CircomCircuitInput serialized

    /* this should be done by the contract upon installation
    pub fn generate_circuit(self){
        let wasm_buffer = include_bytes!("circuit/multiplier.wasm");
        let wasm_buffer = include_bytes!("circuit/multiplier.r1cs");

    }
    */
    // generate CircomInput serialized
    pub fn generate_input(&mut self) -> CircomInput{
        let cfg = CircomConfig::<Bn254>::new(
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
        let circom = builder.setup();
        let mut rng = rand::thread_rng();
        let params = GrothBn::generate_random_parameters_with_reduction(circom, &mut rng).unwrap();
        let circom = builder.build().unwrap();

        let proof = GrothBn::prove(&params, circom, &mut rng).unwrap();
        
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

        CircomInput{
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
        let inputs = &self.generate_input();
        let buffer = serde_json::to_vec(&inputs).unwrap();
        std::fs::write(&self.output, buffer);
    }
}

#[test]
fn test_input_generator(){
    let mut generator = CircomGenerator{
        wasm: PathBuf::from("/users/chef/Desktop/circom-cli/casper-circom/circom/multiplier/multiplier.wasm"),
        r1cs: PathBuf::from("/users/chef/Desktop/circom-cli/casper-circom/circom/multiplier/multiplier.r1cs"),
        output: PathBuf::from("proof.pem"),
        private_inputs: Vec::new(),
        public_inputs: vec![("a".to_string(), 2), ("b".to_string(), 20), ("c".to_string(), 40)]
    };

    let input = generator.generate_input();
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
    );
    generator.dump_input();
}
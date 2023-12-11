# Full circom support in casper-node :closed_lock_with_key:

## Setup a local network with the `casper-circom` branch
To be able to use the circom host-side verifier, integrated in the Casper node, you need to setup a local test network, because this feature has not been merged to the official release branch. Once The setup is complete and the network is running, you can use this crate to generate a valid circom proof for `any circuit` (tested with the multiplier2 `hello-world` circuit provided in the `official circom documentation`). The proof will be written to a file `proof.pem` and the circuit payload (which is required by the on-chain verifier) will be written to `circuit.pem`. The example smart contract utilizes the `include_bytes!` macro to load these files at compile-time, which reduces the gas cost `from ~25 CSPR to < .001 CSPR`.

To setup the network of casper nodes, we will utilize my `nctl-titano-env` configuration.

First, we need to clone the `nctl-titano-env` github repository and checkout to the `circom` branch:

```
git clone git@github.com:jonas089/nctl-titano-env
cd nctl-titano-env
git checkout circom
git pull
```

Next, we want to build the docker image for the custom node with circom support enabled, located at `git@github.com:jonas089/casper-node` branch `circom-verifier`:
```bash
cd nctl-titano-env/custom
./build.sh
```
The build process can take a while (10 Minutes +):

```bash
    <insert example output>
```


Once the node + docker image was successfully built, we can create a new container with the node's image:

```bash
./init-container.sh
```

And source the nctl command line tool:

```bash
source casper-nctl-docker/nctl-activate.sh
```

Then, to start the network, run:

```bash
nctl-start
```

You should see an output similar to this:

```bash
    <insert example output>

```

## Write a circom circuit

The example circuit from the [circom documentation](https://docs.circom.io/getting-started/writing-circuits/) looks like this:

```rust
pragma circom 2.0.0;

/*This circuit template checks that c is the multiplication of a and b.*/  

template Multiplier2 () {  

   // Declaration of signals.  
   signal input a;  
   signal input b;  
   signal output c;  

   // Constraints.  
   c <== a * b;  
}
component main = Multiplier2();
```
`Hint: don't forget to add the last line (it is missing in the official documentation)`



## Generate the payload for the verifier
Run `cargo test --features casper-circom` to generate the payload (`proof.pem`, `circuit.pem`) for the example circuit `multiplier2`. If you want to generate proofs for other circuits, import the `casper-circom` library and construct a custom `Generator`:

```rust
    ...
    let mut generator = CircomGenerator{
        wasm: PathBuf::from("/users/chef/Desktop/circom-cli/casper-circom/circom/multiplier/multiplier.wasm"),
        r1cs: PathBuf::from("/users/chef/Desktop/circom-cli/casper-circom/circom/multiplier/multiplier.r1cs"),
        proof_out: PathBuf::from("proof.pem"),
        circuit_out: PathBuf::from("circuit.pem"),
        private_inputs: Vec::new(),
        public_inputs: vec![("a".to_string(), 2), ("b".to_string(), 20), ("c".to_string(), 40)]
    };
    ...
```
The `casper-circom` library enables anyone to quickly collect circuit inputs and generate payloads for casper smart contracts.

## Deploy the smart contract and await execution
To deploy the smart contract, we use the `casper-client-rs` binary. Run `cargo install casper-client-rs` to install it.

```bash
    casper-client put-deploy ...
    ./get_deploy.sh DEPLOY_HASH
```


If the smart contract deployment returns `Success`, it means that the proof was successfully verified by the on-chain host function. In the event of an invalid proof, `User Error 0` will be returned which maps to `CircomError::InvalidProof`.

Example output for a valid proof:

```bash
    <insert example output>


```

Congratulations :rocket:, you now know how to generate and verify Circom proofs on the Casper Blockchain. While this is still an experimental feature, it's a powerful exercise and might benefit the blockchain ecosystem in the near future :key:.


## Purge the docker image and container
If you want to get rid of all data produced by this tutorial, run the following:

```
nctl-stop && ./rm-container.sh && ../prune.sh
```

This will remove:
    - The docker container that contains the image
    - The docker image

:warning: If you delete the node image, you will have to re-build it before being able to run a local testnet.
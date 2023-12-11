# Full circom support in casper-node :closed_lock_with_key:
This project enables the verification of general purpose `circom` zero knowledge proofs on-chain. This readme serves as a documentation and step-by-step instruction on how to practically apply my `circom` research and get started with on-chain circuit development on the Casper blockchain. :seedling:=>:deciduous_tree:

## What is circom
!todo: write introduction to circom & add links


## Setup a local network with the `casper-circom` branch :computer:
To be able to use the circom host-side verifier, integrated in the Casper node, you need to setup a local test network, because this feature has not been merged to the official release branch. Once The setup is complete and the network is running, you can use this crate to generate a valid circom proof for `any circuit` (tested with the multiplier2 `hello-world` circuit provided in the `official circom documentation`). The proof will be written to a file `proof.pem` and the circuit payload (which is required by the on-chain verifier) will be written to `circuit.pem`. The example smart contract utilizes the `include_bytes!` macro to load these files at compile-time, which reduces the gas cost significantly. The exact cost depends on the size of the `proof.pem` file. Installing the contract is somewhat expensive (for the example it's about `78` CSPR), since the entire serialized circuit is submitted as a payload. But this will usually only happen once in a production system

To setup the network of casper nodes, we will utilize my `nctl-titano-env` github repository, which includes a lot of useful scripts that extend the default docker image.
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

![Output](https://github.com/jonas089/casper-circom/blob/master/resources/build_output.png)

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
    2023-12-11T15:03:41.626978 [INFO] [850] NCTL :: starting node(s) complete        │
    validators-1:casper-net-1-node-1    RUNNING   pid 790, uptime 0:00:08            │
    validators-1:casper-net-1-node-2    RUNNING   pid 791, uptime 0:00:08            │
    validators-1:casper-net-1-node-3    RUNNING   pid 792, uptime 0:00:08            │
    validators-2:casper-net-1-node-4    RUNNING   pid 819, uptime 0:00:07            │
    validators-2:casper-net-1-node-5    RUNNING   pid 820, uptime 0:00:07            │
    validators-3:casper-net-1-node-10   STOPPED   Not started                        │
    validators-3:casper-net-1-node-6    STOPPED   Not started                        │
    validators-3:casper-net-1-node-7    STOPPED   Not started                        │
    validators-3:casper-net-1-node-8    STOPPED   Not started                        │
    validators-3:casper-net-1-node-9    STOPPED   Not started
```

:exclamation: I recommend that before proceeding you make sure that your network is actually producing blocks. To do so, 
run the command `nctl-view-chain-height` a few time (wait several seconds in between) and make sure that the current block height is increasing for all nodes. 
The network can get stuck, though this usually only happens when re-starting it several times & not during initial setup.

## Write a circom circuit :pencil2:
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
    component main {public [a,b,c]} = Multiplier2();
```
:warning: I modified this circuit slightly by making `a, b, c` public inputs. This is because I wanted to make sure that public inputs are passed correctly when utilizing this implementation for more advanced circuits.



## Generate the payload for the verifier :crystal_ball:
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

## Compile and optimize the smart contract
Compiling the smart contract involves two steps, the cargo build process and wasm optimization.

### 1. Compile the smart contract to WebAssembly:
```bash
    rustup target add wasm32-unknown-unknown
    cargo build --release --target wasm32-unknown-unknown
```

The compiled contract can be found in `target/wasm32-unknown-unknown/release/circom-contract.wasm`

### 2. Optimize the smart contract
Requirement: `wasm-opt`:

Install `wasm-opt` as described [here](https://github.com/WebAssembly/binaryen/discussions/3797), or:
```bash 
    cargo install wasm-opt
```

Use `wasm-opt` to optimize `circom-contract.wasm`:
```bash
    wasm-opt --strip-debug --signext-lowering circom-contract.wasm -o contract.wasm
```

This will output the optimized `contract.wasm` file that we will want to deploy to our custom casper network.

## Setup accounts
In order to deploy the smart contract, we need to obtain a funded account's secret key from the network.

To obtain the users directory from the nctl environment, run:
```bash
    ./cp-users.sh
```
Copy the `secret_key.pem` file from `users/user-1/`. This account holds a very large amount of tokens on our testnet and we can spend those tokens to make deploys (install and execute smart contracts).


## Deploy the smart contract and await execution 
To deploy the smart contract, we use the `casper-client-rs` binary. Run `cargo install casper-client-rs` to install it. 
Should you experience issues installing the `casper-client-rs`,
please refer to [the developer portal / documentation](https://docs.casper.network/resources/quick-start/).

```bash
    casper-client put-deploy --node-address http://127.0.0.1:11101 \
        --secret-key ./secret_key.pem \
        --payment-amount 100000000000 \
        --session-path ./contract.wasm \
        --chain-name casper-net-1
```

This will output the deploy hash of your smart contract deployment. To check the status/ execution result of the deploy, run:

```bash
    casper-client get-deploy YOUR_DEPLOY_HASH \
        --node-address http://127.0.0.1:11101
```

If the resolved smart contract deploy returns `Success`, it means that the proof was successfully verified by the on-chain host function.
In the event of an invalid proof, `User Error 0` will be returned which maps to `CircomError::InvalidProof`.

Example output for a valid proof:

```bash
    ...
        {
            "key": "balance-dc401b81391f90ce9b2767c5b51aa95b49aca1831f89fc5063b89b56200b2144",
            "transform": {
            "AddUInt512": "21013038535"
            }
        },
        {
            "key": "hash-8b18a64bb7b8aff6ba80fadf9bb7ebd31ee4ac8e4ae998bc85807bcf3155f32e",
            "transform": "Identity"
        },
        {
            "key": "hash-42b91e9228283325f4a4500ffa368760b84987bfe421837bee58174f8c4f8ed8",
            "transform": "Identity"
        },
        {
            "key": "hash-8b18a64bb7b8aff6ba80fadf9bb7ebd31ee4ac8e4ae998bc85807bcf3155f32e",
            "transform": "Identity"
        },
        {
            "key": "balance-1e594a04c36f46d5e5ebb72064ebfea5659f53ee38e552229b8e989ff57462fd",
            "transform": "Identity"
        },
        {
            "key": "balance-0ea6305f5eb1c20c9295e3d93334a74ce0105d852267eb3c320b716fc5089e95",
            "transform": "Identity"
        },
        {
            "key": "balance-1e594a04c36f46d5e5ebb72064ebfea5659f53ee38e552229b8e989ff57462fd",
            "transform": {
            "WriteCLValue": {
                "cl_type": "U512",
                "bytes": "00",
                "parsed": "0"
            }
            }
        },
        {
            "key": "balance-0ea6305f5eb1c20c9295e3d93334a74ce0105d852267eb3c320b716fc5089e95",
            "transform": {
            "AddUInt512": "78986961465"
            }
        }
        ]
    },
    "transfers": [],
    "cost": "78774708550"
    }
```

Congratulations :rocket:, you now know how to generate and verify Circom proofs on the Casper Blockchain. While this is still an experimental feature, it's a powerful exercise and might benefit the blockchain ecosystem in the near future :key:.


## Purge the docker image and container
If you want to get rid of all data produced by this tutorial, run the following:

```bash
    nctl-stop && ./rm-container.sh && ../prune.sh
```

This will remove:
    - The docker container that contains the image
    - The docker image

:warning: If you delete the node image, you will have to re-build it before being able to run a local testnet.


!todo: explain account setup / obtain secret_key
!todo: add example output
!todo: add links
!todo: explain the example circuit
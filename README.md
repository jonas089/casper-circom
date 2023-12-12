# Full circom support in casper-node :closed_lock_with_key:
This project enables the verification of general purpose [circom](https://docs.circom.io/) zero knowledge proofs on-chain. This readme serves as a documentation and step-by-step instruction on how to practically apply my circom research and get started with on-chain circuit development on the [Casper](https://casper.network/en-us/) blockchain.

Circom is a language with a compiler that enables the implementation of high-level zero knowlege circuits that are reduced to R1CS constraint systems. Read morea about circom [here](https://docs.circom.io/)

## Setup a local network with the `casper-circom` branch :computer:
To be able to use the circom host-side verifier, integrated in the Casper node, you need to setup a local test network, because this feature has not been merged to the official release branch. Once The setup is complete and the network is running, you can use this crate to generate a valid circom proof for any circuit (tested with a variation of the `multiplier2` circuit provided in the [official circom documentation](https://docs.circom.io/getting-started/writing-circuits/)). The proof will be written to a file `proof.pem` and the circuit payload (which is required by the on-chain verifier) will be written to `circuit.pem`. The example smart contract utilizes the `include_bytes!` macro to load these files at compile-time, which reduces the gas cost significantly. The exact cost depends on the size of the `proof.pem` file. Installing the contract is somewhat expensive (for the example it's about `78 CSPR`). The cost of verifying a proof should generally be very low, likely less than `0.001` CSPR per call.

To setup the network of Casper nodes, we will utilize my `nctl-titano-env` github repository, which includes a lot of useful scripts that extend the default docker image.
First, we need to clone the `nctl-titano-env` github repository and checkout to the `circom` branch:

```bash
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

Example output `nctl-view-chain-height`:
```bash
2023-12-11T15:56:33.352392 [INFO] [1729] NCTL :: chain height @ node-1 = 705
2023-12-11T15:56:33.532019 [INFO] [1729] NCTL :: chain height @ node-2 = 705
2023-12-11T15:56:33.712204 [INFO] [1729] NCTL :: chain height @ node-3 = 705
2023-12-11T15:56:33.890155 [INFO] [1729] NCTL :: chain height @ node-4 = 706
2023-12-11T15:56:34.074809 [INFO] [1729] NCTL :: chain height @ node-5 = 706
2023-12-11T15:56:34.218825 [INFO] [1729] NCTL :: chain height @ node-6 = N/A
2023-12-11T15:56:34.368193 [INFO] [1729] NCTL :: chain height @ node-7 = N/A
2023-12-11T15:56:34.513191 [INFO] [1729] NCTL :: chain height @ node-8 = N/A
2023-12-11T15:56:34.654538 [INFO] [1729] NCTL :: chain height @ node-9 = N/A
2023-12-11T15:56:34.799254 [INFO] [1729] NCTL :: chain height @ node-10 = N/A
```

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
Should you experience issues installing the `casper-client-rs`, please refer to [the developer portal / documentation](https://docs.casper.network/resources/quick-start/).

```bash
casper-client put-deploy --node-address http://127.0.0.1:11101 \
    --secret-key ./secret_key.pem \
    --payment-amount 100000000000 \
    --session-path ./contract.wasm \
    --chain-name casper-net-1
```

This will output the deploy hash of your smart contract deployment:
```
    ```
        {
        "jsonrpc": "2.0",
        "id": -4966943512413558264,
        "result": {
            "api_version": "1.0.0",
            "deploy_hash": "47973d600d923c8bcc4c8fe342c62c7a7f65549842c0f51a782cecf35bd12319"
        }
        }
    ```
```

To check the status/ execution result of the deploy, run:
```bash
casper-client get-deploy YOUR_DEPLOY_HASH \
    --node-address http://127.0.0.1:11101
```
*where YOUR_DEPLOY_HASH=47973d600d923c8bcc4c8fe342c62c7a7f65549842c0f51a782cecf35bd12319*

If the resolved smart contract deploy returns `Success`, it means that the contract was successfully installed and a reference to it was created under our the account's `named keys`.

Example output for a successfully deployed contract:

```
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

## Call the verifier endpoint
In order to call the circom verifier entry point of the smart contract, named `call_verifier`, we need to first obtain the contract-hash of the smart contract. To do this we will query a node for our account's named keys and then resolve the contract package:

1. obtain the current `state root hash`
```
casper-client get-state-root-hash --node-address http://127.0.0.1:11101
```
This will output a `state root hash`.

2. use the `account-hash` and `state root hash` to query the account's `named keys`:
```bash
casper-client query-global-state -s STATE_ROOT_HASH \
    --node-address http://127.0.0.1:11101 \
    --key ACCOUNT_HASH
```
*Hint: To obtain the account-hash, import the secret_key.pem file in the casper-signer or casper-wallet browser extension and login to testnet.cspr.live*

Example output:

```
{
"jsonrpc": "2.0",
"id": -1004515512565586593,
"result": {
    "api_version": "1.0.0",
    "block_header": null,
    "stored_value": {
    "Account": {
        "account_hash": "account-hash-2d076da10abff1b755825696d5339b761af7b58b6bd669670e695febbe461a1f",
        "named_keys": [
        {
            "name": "circom_multiplier_contract",
            "key": "hash-eea3193f0ba0dca2b05aacfa763bab8a5422f75d30b45a44e7cd3ebdf63ad7a4"
        }
        ],
        "main_purse": "uref-dc401b81391f90ce9b2767c5b51aa95b49aca1831f89fc5063b89b56200b2144-007",
        "associated_keys": [
        {
            "account_hash": "account-hash-2d076da10abff1b755825696d5339b761af7b58b6bd669670e695febbe461a1f",
            "weight": 1
        }
        ],
        "action_thresholds": {
        "deployment": 1,
        "key_management": 1
        }
    }
    },
    "merkle_proof": "[2490 hex chars]"
}
}
```

3. query the node for the contract package of `circom_multiplier_contract`:
```bash
casper-client query-global-state --node-address http://127.0.0.1:11101 \
    -s STATE_ROOT_HASH \
    --key CONTRACT_IN_NAMED_KEYS
```
*where CONTRACT_IN_NAMED_KEYS = hash-eea3193f0ba0dca2b05aacfa763bab8a5422f75d30b45a44e7cd3ebdf63ad7a4 (see output above) and STATE_ROOT_HASH is the same that was used before*

Example output:

```
{
"jsonrpc": "2.0",
"id": -8431458920986110361,
"result": {
    "api_version": "1.0.0",
    "block_header": null,
    "stored_value": {
    "Contract": {
        "contract_package_hash": "contract-package-dd58020573202dd645b26924ff3cd45a87cfe6f4a8f3bf2658948cbdb8851ae4",
        "contract_wasm_hash": "contract-wasm-73fc7c9e9570d0d3b0109cd80fd931cc6f968f44dc2500dd3d482a22964583fc",
        "named_keys": [],
        "entry_points": [
        {
            "name": "call_verifier",
            "args": [],
            "ret": "Unit",
            "access": "Public",
            "entry_point_type": "Contract"
        }
        ],
        "protocol_version": "1.0.0"
    }
    },
    "merkle_proof": "[1854 hex chars]"
}
}
```

4. query the node for the contract package contents:
```bash
casper-client query-global-state --node-address http://127.0.0.1:11101 \
    -s STATE_ROOT_HASH \
    --key CONTRACT_PACKAGE_HASH
```
*where CONTRACT_PACKAGE_HASH = hash-dd58020573202dd645b26924ff3cd45a87cfe6f4a8f3bf2658948cbdb8851ae4 (see output above) and STATE_ROOT_HASH is the same that was used before*

Example output:

```
{
"jsonrpc": "2.0",
"id": -8946518891340588621,
"result": {
    "api_version": "1.0.0",
    "block_header": null,
    "stored_value": {
    "ContractPackage": {
        "access_key": "uref-28a079a6c9f90fe0e82a615087950bcb63c576fe783f4c10ca3f77abe000cd56-007",
        "versions": [
        {
            "protocol_version_major": 1,
            "contract_version": 1,
            "contract_hash": "contract-eea3193f0ba0dca2b05aacfa763bab8a5422f75d30b45a44e7cd3ebdf63ad7a4"
        }
        ],
        "disabled_versions": [],
        "groups": [],
        "lock_status": "Unlocked"
    }
    },
    "merkle_proof": "[1776 hex chars]"
}
}
```

Now that we have obtained the contract_hash (in this case "hash-eea3193f0ba0dca2b05aacfa763bab8a5422f75d30b45a44e7cd3ebdf63ad7a4"), we can call the `call_verifier` entry point of the smart contract, to verify the circom proof using the node's host-function verifier:
```bash
casper-client put-deploy --node-address http://127.0.0.1:11101 \
    --session-hash CONTRACT_HASH \
    --session-entry-point call_verifier \
    --payment-amount 100000000000 \
    --chain-name casper-net-1 \
    --secret-key ./secret_key.pem
```
*where CONTRACT_HASH = hash-eea3193f0ba0dca2b05aacfa763bab8a5422f75d30b45a44e7cd3ebdf63ad7a4*

Example output:

```
{
"jsonrpc": "2.0",
"id": -4966943512413558264,
"result": {
    "api_version": "1.0.0",
    "deploy_hash": "37973d600d923c8bcc4c8fe342c62c7a7f65549842c0f51a782cecf35bd12318"
}
}
```

As we did when installing the smart contract, we will now obtain the execution result of this deploy:

```bash
    casper-client get-deploy YOUR_DEPLOY_HASH \
        --node-address http://127.0.0.1:11101
```
*where DEPLOY_HASH=37973d600d923c8bcc4c8fe342c62c7a7f65549842c0f51a782cecf35bd12318*

Example output:

```
...
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
            "AddUInt512": "1000264053"
        }
        }
    ]
    },
    "transfers": [],
    "cost": "266720"
}
```

If the execution result does not include an error message, it means that the proof was successfully verified. In this case the execution cost of verifying the proof was only `266720 Motes`. This is equivalent to `0.00026672 CSPR` or (at the time of writing) `0.00001 USD`. The reason why this cheap verification is possible is, that the host function cost is hardcoded to be `200 Gas` / `200 Motes` per call.

In the event of an invalid proof, `User Error 0` will be returned which maps to `CircomError::InvalidProof`.

Congratulations :rocket:, you now know how to generate and verify Circom proofs on the Casper Blockchain. While this is still an experimental feature, it's a powerful exercise and might benefit the blockchain ecosystem in the near future :key:.


## Purge the docker image and container
If you want to get rid of all data produced by this tutorial, run the following:

```bash
nctl-stop && ./rm-container.sh && ../prune.sh
```

This will remove:
```
- The docker container that contains the image
- The docker image
```
:warning: If you delete the node image, you will have to re-build it before being able to run a local testnet.
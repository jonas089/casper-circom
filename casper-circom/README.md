# Structures

CircomInput{
    points
}


CircomCircuitInput{
    .wasm buffer
    .r1cs buffer
}
# Functions

## Generate inputs

inputs:
    - output path
    - .wasm path
    - .r1cs path
    - public inputs to circuit

generate() -> proof.json
a serialized "CircomInput"

see `contract_api::circom::types::CircomInput`
## Serialize circuit

serialize() -> circuit.json

# Usage in contract

use §es! macro to load CircomInput and CircomCircuitInput

see `contract_api::circom::types::CircomCircuitInput`


-> The library is used to generate serialized circuit inputs. Those serialized circuit inputs are loaded by the contract using include_bytes! macro.

# Contract pseudocode

// circuit should be loaded as bytes, not as string
circuit_serialized = include_bytes!!(path)
proof_serialized = include_bytes!!(path)

-> include_bytes!: bytes???

is_valid = circom::circom_verifier(proof, circuit);

...


# Outputs

1. Verifier (as CircomInput)

2. Circuit buffer (as CircomCircuitInput)

# Production

Requires the passing of a serialized CircomInput via SDK - in a deploy. This is always possbile.

# Development

Send payload as contract with include_bytes! -> bytes. Avoid serialization of the wasm buffer (expensive af).

Parse as bytes at compile time using a proc macro.

-> done by contract.

Don't worry about the contract until the generator is sound.



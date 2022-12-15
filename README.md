# YSIP Contract

Smart contracts to build ysip platform.

Consisted within three different types of Contracts, which interacts with ysip chain.
Every Schema of each contract is under /schema directory

### Lifecycle of Contact
1. store on chain
2. instantiate
3. execute / query

cosmwasm docs: https://docs.cosmwasm.com/docs/1.0/


## ICO
Initiate ICO, Create CW20 token for Channel token

## Pair
Create pairs which allow users to trade channel token and uKRW, coin of ysip chain 

## Token
CW20 spec token stands for channel token

How it works?
1. When LLVM rust compiler compiles the contracts, it creates wasm32-unknown-unknown files, which can be run in wasm runtime named Wasmer(https://docs.wasmer.io/)
2. The client sends the transaction storing .wasm file on to chain, wasm files are converted into byte codes, stored every node consisting the blockchain.
3. Extendable wasm module on Cosmos SDK interacts with the contract byte code for instantiate, execute, query

Interaction example: https://github.com/2022-cau-capstone-team02/ysip-cosmwasmjs-example

Production frontend: https://github.com/2022-cau-capstone-team02/ysip-cosmwasmjs-example
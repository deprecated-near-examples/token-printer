Proof of Work Transfer Faucet
=============================

<!-- MAGIC COMMENT: DO NOT DELETE! Everything above this line is hidden on NEAR Examples page -->

Try it out: https://near-examples.github.io/token-printer/

It consists of 2 parts:

## Transfer Faucet contract

A Faucet contract allows to transfer tokens to a desired account for doing required Proof of Work.
This contract is based on PoW faucet example: https://github.com/near-examples/pow-faucet

The transfer amount is set to 100N tokens. It's enough to deploy a 1Mb contract.

Proof of Work works the following way:

You need to compute a u64 salt (nonce) for a given account in such a way
that the `sha256(account_id + ':' + salt)` has at the amount of leading zero bits as
the required `min_difficulty`. The hash has to be unique in order to receive transfer.
One account can request multiple transfers.

## Faucet frontend

Allows to enter the account ID to receive transfer. And it computes the Proof of Work required by the contract using front-end JS.

https://near-examples.github.io/token-printer/

## Testing
To test run:
```bash
cargo test
```

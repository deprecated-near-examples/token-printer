Proof of Work Faucet
====================

<!-- MAGIC COMMENT: DO NOT DELETE! Everything above this line is hidden on NEAR Examples page -->

Try it out: https://near-examples.github.io/pow-faucet/

It consists of 2 parts:

## Faucet contract

A Faucet contract that creates and funds accounts if the caller provides basic proof of work
to avoid sybil attacks and draining balance too fast.

The new account always receives 1/1000 of the remaining balance.

Proof of Work works the following way:

You need to compute a u64 salt (nonce) for a given account and a given public key in such a way
that the `sha256(account_id + ':' + public_key + ':' + salt)` has at the amount of leading zero bits as
the required `min_difficulty`.

## Faucet frontend

Allows to create a new account with suffix `.meta` by computing the Proof of Work required by the contract using front-end JS.

https://near-examples.github.io/pow-faucet/

## Testing
To test run:
```bash
cargo test
```

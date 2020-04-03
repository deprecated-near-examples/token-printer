# Account faucet with Proof of Work

A Faucet contract that creates and funds accounts if the caller provides basic proof of work
to avoid sybil attacks and draining balance too fast.

The new account always receives 1/1000 of the remaining balance.

Proof of Work works the following way:

You need to compute a u64 salt (nonce) for a given account and a given public key in such a way
that the `sha256(account_id + ':' + public_key + ':' + salt)` has more leading zero bits than
the required `min_difficulty`.

## Testing
To test run:
```bash
cargo test
```

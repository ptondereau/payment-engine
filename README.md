# Payment engine

## Description
A payment engine that can process CSV data to produce an account's holder view of its payments.

## Assumption
- 

## Technical details
- The main engine doesn't have a hard complexity thanks to `HashMap`. I've used this to store transaction for an account and also processed account.
- I've used [MPSC](https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html) from Tokio library to handle efficiency by using channels to process transactions. There are 3 channel engines:
  - For reading CSV lines
  - For handling action to apply to an account
  - For writing to stdout
- Account worker is just a gateway to react to a command and apply business logic to an account.
- I've used `rust_decimal` to wrap the amount column because it provides some useful error handling and especially to check against overflow when processing `add` operation.
- I've tried to define explicit error handling in `src/errors.rs` instead of using dynamic one and also in additon to `env_logger`.

## Issues
- I don't know how to define the right buffer size for all channels. 
- I don't know if MPSC is the best fit for efficiency.
- Testing data: as a developer of the team, I would grab more datas from business people to compose multiple CSV files to handle especially for automated tests.
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

### Breaking Changes

## zap [0.2.0] [PR #15](https://github.com/MeteoraAg/zap-program/pull/15)

### Added
New endpoints `initialize_ledger_account`, `close_ledger_account`, `set_ledger_balance`, `update_ledger_balance_after_swap` and `zap_in_damm_v2` that allow user to zap in damm v2 easily. Some examples:

1. User has 1 SOL, and want to add liquidity in pool SOL-USDC, then they will send a batch of transactions (can use jito):
- Swap 0.5 SOL to USDC in JUP or directly through AMMs
- Call endpoint `initialize_ledger_account` to create a ledger account
- Set balance for token a (SOL) to 0.5 SOL in ledger account through endpoint `set_ledger_balance`
- Set balance for token b (USDC) through endpoint `update_ledger_balance_after_swap`, it will take output USDC from step 1
- Call `zap_in_damm_v2` to add liquidity in damm v2
- Close ledger account through endpoint `close_ledger_account`

2. User has 1 SOL, and want to add liquidity in pool MET-USDC, then they will send a batch of transactions (can use jito):
- Swap 0.5 SOL to MET in JUP or directly through AMMs
- Swap 0.5 SOL to USDC in JUP or directly through AMMs
- Call endpoint `initialize_ledger_account` to create a ledger account
- Set balance for token a (MET) through endpoint `update_ledger_balance_after_swap`, it will take output MET from step 1
- Set balance for token b (USDC) through endpoint `update_ledger_balance_after_swap`, it will take output USDC from step 2
- Call `zap_in_damm_v2` to add liquidity in damm v2
- Close ledger account through endpoint `close_ledger_account`
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
New endpoints `transfer_delta_balance`, `transfer_full_balance`, `transfer_max_balance` and `zap_in_damm_v2` that allow user to zap in damm v2 easily. Some examples:

1. User has 1 SOL, and want to add liquidity in pool SOL-USDC, then they will send a batch of transactions (can use jito):
- Swap 0.5 SOL to USDC in JUP or directly through AMMs
- Create a new auxiliary token account for SOL, and transfer 0.5 SOL to that token account 
- Create a new auxiliary token account for USDC, and use function `transfer_delta_balance` to transfer output USDC from step 1 to that token account
- Call `zap_in_damm_v2` to add liquidity in damm v2
- Use function `transfer_full_balance` to transfer all remaining amounts of auxiliary token accounts to their main token accounts (ATA)
- Close auxiliary token accounts

2. User has 1 SOL, and want to add liquidity in pool USDC-MET, then they will send a batch of transactions (can use jito):
- Swap 0.5 SOL to USDC in JUP or directly through AMMs
- Swap 0.5 SOL to MET in JUP or directly through AMMs
- Create a new auxiliary token account for USDC, and use function `transfer_delta_balance` to transfer output USDC from step 1 to that token account
- Create a new auxiliary token account for MET, and use function `transfer_delta_balance` to transfer output MET from step 1 to that token account
- Call `zap_in_damm_v2` to add liquidity in damm v2
- Use function `transfer_full_balance` to transfer all remaining amounts of auxiliary token accounts to their main token accounts (ATA)
- Close auxiliary token accounts
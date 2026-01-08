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

## zap [0.2.1] [PR #41](https://github.com/MeteoraAg/zap-program/pull/41)

### Fixed

- Fix zap in damm-v2 with new base fee mode

## zap [0.2.0] [PR #15](https://github.com/MeteoraAg/zap-program/pull/15)

### Added

New endpoints `initialize_ledger_account`, `close_ledger_account`, `set_ledger_balance`, `update_ledger_balance_after_swap`, `zap_in_damm_v2`, `zap_in_dlmm_for_initialized_position` and `zap_in_dlmm_for_uninitialized_position` that allow user to zap in damm v2 and dlmm easily. Refer `ZAPIN.md` for zap_in examples

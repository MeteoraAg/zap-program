# ZAP

The Zap program provide util functions that allow user to zap in/out from any Amms or any protocols, but we mostly support Meteora AMMs for now. 

## Zap out

User can withdraw liquidity or claim fees from AMM pools and immediately swaps the withdrawn tokens through direct pools (Damm V2 or DLMM) or Jupiter.

## Zap in (Please refer examples in ZAPIN.md)

- Swap and deposit in Damm V2
- Swap and deposit in DLMM

## Development

### Dependencies

- anchor 1.0.2
- solana 3.1.10
- rust 1.93.0

### Build

```
anchor build --ignore-keys
```

### Test

```
bun install
bun run build-local-test
```

### Program Address

- Mainnet-beta and Devnet: zapvX9M3uf5pvy4wRPAbQgdQsM1xmuiFnkfHKPvwMiz

# ZAP

The Zap program provide util functions that allow user to zap in/out from any Amms or any protocols, but we mostly support Meteora AMMs for now. 

## Zap out

User can withdraw liquidity or claim fees from Meteora pools immediately swaps the withdrawn tokens through direct pools (Damm V2 or DLMM) or Jupiter.

## Zap in

### TODO
- Swap and deposit in Damm V2
- Swap and deposit in DLMM

## Development

### Dependencies

- anchor 0.31.0
- solana 2.1.0
- rust 1.79.0



### Build

Program

```
anchor build
```

### Test

```
pnpm install
pnpm test
```

### Program Address

- Mainnet-beta and Devnet: zapvX9M3uf5pvy4wRPAbQgdQsM1xmuiFnkfHKPvwMiz

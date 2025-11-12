## Examples to zap_in damm v2

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


## Examples to zap_in DLMM

1. User has 1 SOL, and want to add liquidity in pool SOL-USDC, then they will send a batch of transactions (can use jito):
- Swap 0.5 SOL to USDC in JUP or directly through AMMs
- Call endpoint `initialize_ledger_account` to create a ledger account
- Set balance for token x (SOL) to 0.5 SOL in ledger account through endpoint `set_ledger_balance`
- Set balance for token y (USDC) through endpoint `update_ledger_balance_after_swap`, it will take output USDC from step 1
- Call `zap_in_dlmm_for_uninitialized_position`, that will create position and add liquidity to that position
- Close ledger account through endpoint `close_ledger_account`

2. User has a position (SOL-USDC) that is out of range and they want to rebalance the position to concenstrate around pool price, they they will send a batch of transactions:
- Withdraw 100% position,
- Call endpoint `zap_out` to swap half of SOL to USDC
- Call endpoint `initialize_ledger_account` to create a ledger account
- Set balance for token x (SOL) through endpoint `update_ledger_balance_after_swap`, delta of SOL changed in user token balance
- Set balance for token y (USDC) through endpoint `update_ledger_balance_after_swap`, delta of USDC changed in user token balance
- Call `zap_in_dlmm_for_initialized_position`, that will rebalance position with the new balances
- Close ledger account through endpoint `close_ledger_account`
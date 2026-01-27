/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/jupiter.json`.
 */
export type Jupiter = {
  address: "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4";
  metadata: {
    name: "jupiter";
    version: "0.1.0";
    spec: "0.1.0";
  };
  instructions: [
    {
      name: "route";
      docs: ["route_plan Topologically sorted trade DAG"];
      discriminator: [229, 23, 203, 151, 122, 227, 173, 42];
      accounts: [
        {
          name: "tokenProgram";
        },
        {
          name: "userTransferAuthority";
          signer: true;
        },
        {
          name: "userSourceTokenAccount";
          writable: true;
        },
        {
          name: "userDestinationTokenAccount";
          writable: true;
        },
        {
          name: "destinationTokenAccount";
          writable: true;
          optional: true;
        },
        {
          name: "destinationMint";
        },
        {
          name: "platformFeeAccount";
          writable: true;
          optional: true;
        },
        {
          name: "eventAuthority";
        },
        {
          name: "program";
        }
      ];
      args: [
        {
          name: "routePlan";
          type: {
            vec: {
              defined: {
                name: "routePlanStep";
              };
            };
          };
        },
        {
          name: "inAmount";
          type: "u64";
        },
        {
          name: "quotedOutAmount";
          type: "u64";
        },
        {
          name: "slippageBps";
          type: "u16";
        },
        {
          name: "platformFeeBps";
          type: "u8";
        }
      ];
      returns: "u64";
    },
    {
      name: "routeWithTokenLedger";
      discriminator: [150, 86, 71, 116, 167, 93, 14, 104];
      accounts: [
        {
          name: "tokenProgram";
        },
        {
          name: "userTransferAuthority";
          signer: true;
        },
        {
          name: "userSourceTokenAccount";
          writable: true;
        },
        {
          name: "userDestinationTokenAccount";
          writable: true;
        },
        {
          name: "destinationTokenAccount";
          writable: true;
          optional: true;
        },
        {
          name: "destinationMint";
        },
        {
          name: "platformFeeAccount";
          writable: true;
          optional: true;
        },
        {
          name: "tokenLedger";
        },
        {
          name: "eventAuthority";
        },
        {
          name: "program";
        }
      ];
      args: [
        {
          name: "routePlan";
          type: {
            vec: {
              defined: {
                name: "routePlanStep";
              };
            };
          };
        },
        {
          name: "quotedOutAmount";
          type: "u64";
        },
        {
          name: "slippageBps";
          type: "u16";
        },
        {
          name: "platformFeeBps";
          type: "u8";
        }
      ];
      returns: "u64";
    },
    {
      name: "exactOutRoute";
      discriminator: [208, 51, 239, 151, 123, 43, 237, 92];
      accounts: [
        {
          name: "tokenProgram";
        },
        {
          name: "userTransferAuthority";
          signer: true;
        },
        {
          name: "userSourceTokenAccount";
          writable: true;
        },
        {
          name: "userDestinationTokenAccount";
          writable: true;
        },
        {
          name: "destinationTokenAccount";
          writable: true;
          optional: true;
        },
        {
          name: "sourceMint";
        },
        {
          name: "destinationMint";
        },
        {
          name: "platformFeeAccount";
          writable: true;
          optional: true;
        },
        {
          name: "token2022Program";
          optional: true;
        },
        {
          name: "eventAuthority";
        },
        {
          name: "program";
        }
      ];
      args: [
        {
          name: "routePlan";
          type: {
            vec: {
              defined: {
                name: "routePlanStep";
              };
            };
          };
        },
        {
          name: "outAmount";
          type: "u64";
        },
        {
          name: "quotedInAmount";
          type: "u64";
        },
        {
          name: "slippageBps";
          type: "u16";
        },
        {
          name: "platformFeeBps";
          type: "u8";
        }
      ];
      returns: "u64";
    },
    {
      name: "sharedAccountsRoute";
      docs: [
        "Route by using program owned token accounts and open orders accounts."
      ];
      discriminator: [193, 32, 155, 51, 65, 214, 156, 129];
      accounts: [
        {
          name: "tokenProgram";
        },
        {
          name: "programAuthority";
        },
        {
          name: "userTransferAuthority";
          signer: true;
        },
        {
          name: "sourceTokenAccount";
          writable: true;
        },
        {
          name: "programSourceTokenAccount";
          writable: true;
        },
        {
          name: "programDestinationTokenAccount";
          writable: true;
        },
        {
          name: "destinationTokenAccount";
          writable: true;
        },
        {
          name: "sourceMint";
        },
        {
          name: "destinationMint";
        },
        {
          name: "platformFeeAccount";
          writable: true;
          optional: true;
        },
        {
          name: "token2022Program";
          optional: true;
        },
        {
          name: "eventAuthority";
        },
        {
          name: "program";
        }
      ];
      args: [
        {
          name: "id";
          type: "u8";
        },
        {
          name: "routePlan";
          type: {
            vec: {
              defined: {
                name: "routePlanStep";
              };
            };
          };
        },
        {
          name: "inAmount";
          type: "u64";
        },
        {
          name: "quotedOutAmount";
          type: "u64";
        },
        {
          name: "slippageBps";
          type: "u16";
        },
        {
          name: "platformFeeBps";
          type: "u8";
        }
      ];
      returns: "u64";
    },
    {
      name: "sharedAccountsRouteWithTokenLedger";
      discriminator: [230, 121, 143, 80, 119, 159, 106, 170];
      accounts: [
        {
          name: "tokenProgram";
        },
        {
          name: "programAuthority";
        },
        {
          name: "userTransferAuthority";
          signer: true;
        },
        {
          name: "sourceTokenAccount";
          writable: true;
        },
        {
          name: "programSourceTokenAccount";
          writable: true;
        },
        {
          name: "programDestinationTokenAccount";
          writable: true;
        },
        {
          name: "destinationTokenAccount";
          writable: true;
        },
        {
          name: "sourceMint";
        },
        {
          name: "destinationMint";
        },
        {
          name: "platformFeeAccount";
          writable: true;
          optional: true;
        },
        {
          name: "token2022Program";
          optional: true;
        },
        {
          name: "tokenLedger";
        },
        {
          name: "eventAuthority";
        },
        {
          name: "program";
        }
      ];
      args: [
        {
          name: "id";
          type: "u8";
        },
        {
          name: "routePlan";
          type: {
            vec: {
              defined: {
                name: "routePlanStep";
              };
            };
          };
        },
        {
          name: "quotedOutAmount";
          type: "u64";
        },
        {
          name: "slippageBps";
          type: "u16";
        },
        {
          name: "platformFeeBps";
          type: "u8";
        }
      ];
      returns: "u64";
    },
    {
      name: "sharedAccountsExactOutRoute";
      docs: [
        "Route by using program owned token accounts and open orders accounts."
      ];
      discriminator: [176, 209, 105, 168, 154, 125, 69, 62];
      accounts: [
        {
          name: "tokenProgram";
        },
        {
          name: "programAuthority";
        },
        {
          name: "userTransferAuthority";
          signer: true;
        },
        {
          name: "sourceTokenAccount";
          writable: true;
        },
        {
          name: "programSourceTokenAccount";
          writable: true;
        },
        {
          name: "programDestinationTokenAccount";
          writable: true;
        },
        {
          name: "destinationTokenAccount";
          writable: true;
        },
        {
          name: "sourceMint";
        },
        {
          name: "destinationMint";
        },
        {
          name: "platformFeeAccount";
          writable: true;
          optional: true;
        },
        {
          name: "token2022Program";
          optional: true;
        },
        {
          name: "eventAuthority";
        },
        {
          name: "program";
        }
      ];
      args: [
        {
          name: "id";
          type: "u8";
        },
        {
          name: "routePlan";
          type: {
            vec: {
              defined: {
                name: "routePlanStep";
              };
            };
          };
        },
        {
          name: "outAmount";
          type: "u64";
        },
        {
          name: "quotedInAmount";
          type: "u64";
        },
        {
          name: "slippageBps";
          type: "u16";
        },
        {
          name: "platformFeeBps";
          type: "u8";
        }
      ];
      returns: "u64";
    },
    {
      name: "setTokenLedger";
      discriminator: [228, 85, 185, 112, 78, 79, 77, 2];
      accounts: [
        {
          name: "tokenLedger";
          writable: true;
        },
        {
          name: "tokenAccount";
        }
      ];
      args: [];
    },
    {
      name: "createOpenOrders";
      discriminator: [229, 194, 212, 172, 8, 10, 134, 147];
      accounts: [
        {
          name: "openOrders";
          writable: true;
        },
        {
          name: "payer";
          writable: true;
          signer: true;
        },
        {
          name: "dexProgram";
        },
        {
          name: "systemProgram";
        },
        {
          name: "rent";
        },
        {
          name: "market";
        }
      ];
      args: [];
    },
    {
      name: "createTokenAccount";
      discriminator: [147, 241, 123, 100, 244, 132, 174, 118];
      accounts: [
        {
          name: "tokenAccount";
          writable: true;
        },
        {
          name: "user";
          writable: true;
          signer: true;
        },
        {
          name: "mint";
        },
        {
          name: "tokenProgram";
        },
        {
          name: "systemProgram";
        }
      ];
      args: [
        {
          name: "bump";
          type: "u8";
        }
      ];
    },
    {
      name: "createProgramOpenOrders";
      discriminator: [28, 226, 32, 148, 188, 136, 113, 171];
      accounts: [
        {
          name: "openOrders";
          writable: true;
        },
        {
          name: "payer";
          writable: true;
          signer: true;
        },
        {
          name: "programAuthority";
        },
        {
          name: "dexProgram";
        },
        {
          name: "systemProgram";
        },
        {
          name: "rent";
        },
        {
          name: "market";
        }
      ];
      args: [
        {
          name: "id";
          type: "u8";
        }
      ];
    },
    {
      name: "claim";
      discriminator: [62, 198, 214, 193, 213, 159, 108, 210];
      accounts: [
        {
          name: "wallet";
          writable: true;
        },
        {
          name: "programAuthority";
          writable: true;
        },
        {
          name: "systemProgram";
        }
      ];
      args: [
        {
          name: "id";
          type: "u8";
        }
      ];
      returns: "u64";
    },
    {
      name: "claimToken";
      discriminator: [116, 206, 27, 191, 166, 19, 0, 73];
      accounts: [
        {
          name: "payer";
          writable: true;
          signer: true;
        },
        {
          name: "wallet";
        },
        {
          name: "programAuthority";
        },
        {
          name: "programTokenAccount";
          writable: true;
        },
        {
          name: "destinationTokenAccount";
          writable: true;
        },
        {
          name: "mint";
        },
        {
          name: "associatedTokenTokenProgram";
        },
        {
          name: "associatedTokenProgram";
        },
        {
          name: "systemProgram";
        }
      ];
      args: [
        {
          name: "id";
          type: "u8";
        }
      ];
      returns: "u64";
    },
    {
      name: "createTokenLedger";
      discriminator: [232, 242, 197, 253, 240, 143, 129, 52];
      accounts: [
        {
          name: "tokenLedger";
          writable: true;
          signer: true;
        },
        {
          name: "payer";
          writable: true;
          signer: true;
        },
        {
          name: "systemProgram";
        }
      ];
      args: [];
    }
  ];
  accounts: [
    {
      name: "tokenLedger";
      discriminator: [156, 247, 9, 188, 54, 108, 85, 77];
    }
  ];
  events: [
    {
      name: "swapEvent";
      discriminator: [64, 198, 205, 232, 38, 8, 113, 226];
    },
    {
      name: "feeEvent";
      discriminator: [73, 79, 78, 127, 184, 213, 13, 220];
    }
  ];
  errors: [
    {
      code: 6000;
      name: "emptyRoute";
      msg: "Empty route";
    },
    {
      code: 6001;
      name: "slippageToleranceExceeded";
      msg: "Slippage tolerance exceeded";
    },
    {
      code: 6002;
      name: "invalidCalculation";
      msg: "Invalid calculation";
    },
    {
      code: 6003;
      name: "missingPlatformFeeAccount";
      msg: "Missing platform fee account";
    },
    {
      code: 6004;
      name: "invalidSlippage";
      msg: "Invalid slippage";
    },
    {
      code: 6005;
      name: "notEnoughPercent";
      msg: "Not enough percent to 100";
    },
    {
      code: 6006;
      name: "invalidInputIndex";
      msg: "Token input index is invalid";
    },
    {
      code: 6007;
      name: "invalidOutputIndex";
      msg: "Token output index is invalid";
    },
    {
      code: 6008;
      name: "notEnoughAccountKeys";
      msg: "Not Enough Account keys";
    },
    {
      code: 6009;
      name: "nonZeroMinimumOutAmountNotSupported";
      msg: "Non zero minimum out amount not supported";
    },
    {
      code: 6010;
      name: "invalidRoutePlan";
      msg: "Invalid route plan";
    },
    {
      code: 6011;
      name: "invalidReferralAuthority";
      msg: "Invalid referral authority";
    },
    {
      code: 6012;
      name: "ledgerTokenAccountDoesNotMatch";
      msg: "Token account doesn't match the ledger";
    },
    {
      code: 6013;
      name: "invalidTokenLedger";
      msg: "Invalid token ledger";
    },
    {
      code: 6014;
      name: "incorrectTokenProgramId";
      msg: "Token program ID is invalid";
    },
    {
      code: 6015;
      name: "tokenProgramNotProvided";
      msg: "Token program not provided";
    },
    {
      code: 6016;
      name: "swapNotSupported";
      msg: "Swap not supported";
    },
    {
      code: 6017;
      name: "exactOutAmountNotMatched";
      msg: "Exact out amount doesn't match";
    }
  ];
  types: [
    {
      name: "amountWithSlippage";
      type: {
        kind: "struct";
        fields: [
          {
            name: "amount";
            type: "u64";
          },
          {
            name: "slippageBps";
            type: "u16";
          }
        ];
      };
    },
    {
      name: "routePlanStep";
      type: {
        kind: "struct";
        fields: [
          {
            name: "swap";
            type: {
              defined: {
                name: "swap";
              };
            };
          },
          {
            name: "percent";
            type: "u8";
          },
          {
            name: "inputIndex";
            type: "u8";
          },
          {
            name: "outputIndex";
            type: "u8";
          }
        ];
      };
    },
    {
      name: "side";
      type: {
        kind: "enum";
        variants: [
          {
            name: "bid";
          },
          {
            name: "ask";
          }
        ];
      };
    },
    {
      name: "swap";
      type: {
        kind: "enum";
        variants: [
          {
            name: "saber";
          },
          {
            name: "saberAddDecimalsDeposit";
          },
          {
            name: "saberAddDecimalsWithdraw";
          },
          {
            name: "tokenSwap";
          },
          {
            name: "sencha";
          },
          {
            name: "step";
          },
          {
            name: "cropper";
          },
          {
            name: "raydium";
          },
          {
            name: "crema";
            fields: [
              {
                name: "aToB";
                type: "bool";
              }
            ];
          },
          {
            name: "lifinity";
          },
          {
            name: "mercurial";
          },
          {
            name: "cykura";
          },
          {
            name: "serum";
            fields: [
              {
                name: "side";
                type: {
                  defined: {
                    name: "side";
                  };
                };
              }
            ];
          },
          {
            name: "marinadeDeposit";
          },
          {
            name: "marinadeUnstake";
          },
          {
            name: "aldrin";
            fields: [
              {
                name: "side";
                type: {
                  defined: {
                    name: "side";
                  };
                };
              }
            ];
          },
          {
            name: "aldrinV2";
            fields: [
              {
                name: "side";
                type: {
                  defined: {
                    name: "side";
                  };
                };
              }
            ];
          },
          {
            name: "whirlpool";
            fields: [
              {
                name: "aToB";
                type: "bool";
              }
            ];
          },
          {
            name: "invariant";
            fields: [
              {
                name: "xToY";
                type: "bool";
              }
            ];
          },
          {
            name: "meteora";
          },
          {
            name: "gooseFx";
          },
          {
            name: "deltaFi";
            fields: [
              {
                name: "stable";
                type: "bool";
              }
            ];
          },
          {
            name: "balansol";
          },
          {
            name: "marcoPolo";
            fields: [
              {
                name: "xToY";
                type: "bool";
              }
            ];
          },
          {
            name: "dradex";
            fields: [
              {
                name: "side";
                type: {
                  defined: {
                    name: "side";
                  };
                };
              }
            ];
          },
          {
            name: "lifinityV2";
          },
          {
            name: "raydiumClmm";
          },
          {
            name: "openbook";
            fields: [
              {
                name: "side";
                type: {
                  defined: {
                    name: "side";
                  };
                };
              }
            ];
          },
          {
            name: "phoenix";
            fields: [
              {
                name: "side";
                type: {
                  defined: {
                    name: "side";
                  };
                };
              }
            ];
          },
          {
            name: "symmetry";
            fields: [
              {
                name: "fromTokenId";
                type: "u64";
              },
              {
                name: "toTokenId";
                type: "u64";
              }
            ];
          },
          {
            name: "tokenSwapV2";
          },
          {
            name: "heliumTreasuryManagementRedeemV0";
          },
          {
            name: "stakeDexStakeWrappedSol";
          },
          {
            name: "stakeDexSwapViaStake";
            fields: [
              {
                name: "bridgeStakeSeed";
                type: "u32";
              }
            ];
          },
          {
            name: "gooseFxv2";
          },
          {
            name: "perps";
          },
          {
            name: "perpsAddLiquidity";
          },
          {
            name: "perpsRemoveLiquidity";
          },
          {
            name: "meteoraDlmm";
          },
          {
            name: "openBookV2";
            fields: [
              {
                name: "side";
                type: {
                  defined: {
                    name: "side";
                  };
                };
              }
            ];
          },
          {
            name: "raydiumClmmV2";
          },
          {
            name: "clone";
            fields: [
              {
                name: "poolIndex";
                type: "u8";
              },
              {
                name: "quantityIsInput";
                type: "bool";
              },
              {
                name: "quantityIsCollateral";
                type: "bool";
              }
            ];
          },
          {
            name: "whirlpoolSwapV2";
            fields: [
              {
                name: "aToB";
                type: "bool";
              },
              {
                name: "remainingAccountsInfo";
                type: {
                  option: {
                    defined: {
                      name: "remainingAccountsInfo";
                    };
                  };
                };
              }
            ];
          },
          {
            name: "oneIntro";
          },
          {
            name: "pumpdotfunWrappedBuy";
          },
          {
            name: "pumpdotfunWrappedSell";
          },
          {
            name: "perpsV2";
          },
          {
            name: "perpsV2AddLiquidity";
          },
          {
            name: "perpsV2RemoveLiquidity";
          },
          {
            name: "moonshotWrappedBuy";
          },
          {
            name: "moonshotWrappedSell";
          },
          {
            name: "stabbleStableSwap";
          },
          {
            name: "stabbleWeightedSwap";
          },
          {
            name: "obric";
            fields: [
              {
                name: "xToY";
                type: "bool";
              }
            ];
          },
          {
            name: "foxBuyFromEstimatedCost";
          },
          {
            name: "foxClaimPartial";
            fields: [
              {
                name: "isY";
                type: "bool";
              }
            ];
          },
          {
            name: "solFi";
            fields: [
              {
                name: "isQuoteToBase";
                type: "bool";
              }
            ];
          },
          {
            name: "solayerDelegateNoInit";
          },
          {
            name: "solayerUndelegateNoInit";
          },
          {
            name: "tokenMill";
            fields: [
              {
                name: "side";
                type: {
                  defined: {
                    name: "side";
                  };
                };
              }
            ];
          },
          {
            name: "daosFunBuy";
          },
          {
            name: "daosFunSell";
          }
        ];
      };
    },
    {
      name: "remainingAccountsSlice";
      type: {
        kind: "struct";
        fields: [
          {
            name: "accountsType";
            type: {
              defined: {
                name: "accountsType";
              };
            };
          },
          {
            name: "length";
            type: "u8";
          }
        ];
      };
    },
    {
      name: "remainingAccountsInfo";
      type: {
        kind: "struct";
        fields: [
          {
            name: "slices";
            type: {
              vec: {
                defined: {
                  name: "remainingAccountsSlice";
                };
              };
            };
          }
        ];
      };
    },
    {
      name: "accountsType";
      type: {
        kind: "enum";
        variants: [
          {
            name: "transferHookA";
          },
          {
            name: "transferHookB";
          }
        ];
      };
    },
    {
      name: "tokenLedger";
      type: {
        kind: "struct";
        fields: [
          {
            name: "tokenAccount";
            type: "pubkey";
          },
          {
            name: "amount";
            type: "u64";
          }
        ];
      };
    },
    {
      name: "swapEvent";
      type: {
        kind: "struct";
        fields: [
          {
            name: "amm";
            type: "pubkey";
          },
          {
            name: "inputMint";
            type: "pubkey";
          },
          {
            name: "inputAmount";
            type: "u64";
          },
          {
            name: "outputMint";
            type: "pubkey";
          },
          {
            name: "outputAmount";
            type: "u64";
          }
        ];
      };
    },
    {
      name: "feeEvent";
      type: {
        kind: "struct";
        fields: [
          {
            name: "account";
            type: "pubkey";
          },
          {
            name: "mint";
            type: "pubkey";
          },
          {
            name: "amount";
            type: "u64";
          }
        ];
      };
    }
  ];
};

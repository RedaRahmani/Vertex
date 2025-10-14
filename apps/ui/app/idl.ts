export const DEFAULT_IDL = {
  "address": "B4yaCkpGZB9Xnm2ZRcj9k1stdkXzkCJdbU26EWj8h7Dc",
  "metadata": {
    "name": "keystone_fee_router",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "Fee router integrating Meteora cp-amm (DAMM v2) and Streamflow for quote-only honorary positions with daily distribution crank"
  },
  "docs": [
    "Program instructions."
  ],
  "instructions": [
    {
      "name": "crank_distribute",
      "docs": [
        "Permissionless daily crank to claim quote fees and distribute to investors."
      ],
      "discriminator": [
        157,
        246,
        241,
        181,
        107,
        30,
        218,
        241
      ],
      "accounts": [
        {
          "name": "cp_program"
        },
        {
          "name": "cp_pool"
        },
        {
          "name": "policy",
          "writable": true
        },
        {
          "name": "progress",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  103,
                  114,
                  101,
                  115,
                  115
                ]
              },
              {
                "kind": "account",
                "path": "policy.cp_pool",
                "account": "Policy"
              }
            ]
          }
        },
        {
          "name": "payer",
          "docs": [
            "Signer paying rent for progress pagination account if needed."
          ],
          "writable": true,
          "signer": true
        },
        {
          "name": "vault_authority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "policy"
              }
            ]
          }
        },
        {
          "name": "treasury_quote_ata",
          "docs": [
            "Treasury quote ATA to transfer from."
          ],
          "writable": true
        },
        {
          "name": "creator_quote_ata",
          "docs": [
            "Creator ATA to receive remainder on day close."
          ],
          "writable": true
        },
        {
          "name": "investor_quote_ata",
          "docs": [
            "Investor quote ATA for this page entry (must match policy.quote_mint)."
          ],
          "writable": true
        },
        {
          "name": "stream"
        },
        {
          "name": "token_program",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        },
        {
          "name": "system_program",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "args",
          "type": {
            "defined": {
              "name": "CrankArgs"
            }
          }
        }
      ]
    },
    {
      "name": "init_honorary_position",
      "docs": [
        "Initialize an empty honorary position owned by a PDA."
      ],
      "discriminator": [
        172,
        129,
        201,
        31,
        252,
        254,
        81,
        150
      ],
      "accounts": [
        {
          "name": "authority",
          "writable": true,
          "signer": true,
          "relations": [
            "policy"
          ]
        },
        {
          "name": "policy",
          "writable": true
        },
        {
          "name": "cp_pool"
        },
        {
          "name": "quote_mint"
        },
        {
          "name": "owner_pda",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "policy"
              },
              {
                "kind": "const",
                "value": [
                  105,
                  110,
                  118,
                  101,
                  115,
                  116,
                  111,
                  114,
                  95,
                  102,
                  101,
                  101,
                  95,
                  112,
                  111,
                  115,
                  95,
                  111,
                  119,
                  110,
                  101,
                  114
                ]
              }
            ]
          }
        },
        {
          "name": "cp_position"
        },
        {
          "name": "honorary_position",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  111,
                  115,
                  105,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "policy"
              }
            ]
          }
        },
        {
          "name": "system_program",
          "address": "11111111111111111111111111111111"
        },
        {
          "name": "rent",
          "address": "SysvarRent111111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "init_policy",
      "docs": [
        "Initialize policy PDA per cp-amm pool with quote mint."
      ],
      "discriminator": [
        45,
        234,
        110,
        100,
        209,
        146,
        191,
        86
      ],
      "accounts": [
        {
          "name": "authority",
          "writable": true,
          "signer": true
        },
        {
          "name": "policy",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  111,
                  108,
                  105,
                  99,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "cp_pool"
              }
            ]
          }
        },
        {
          "name": "cp_pool"
        },
        {
          "name": "quote_mint"
        },
        {
          "name": "creator_quote_ata",
          "writable": true
        },
        {
          "name": "treasury_quote_ata",
          "writable": true
        },
        {
          "name": "vault_authority",
          "docs": [
            "Vault authority PDA must own treasury_quote_ata."
          ],
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "policy"
              }
            ]
          }
        },
        {
          "name": "token_program",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        },
        {
          "name": "system_program",
          "address": "11111111111111111111111111111111"
        },
        {
          "name": "rent",
          "address": "SysvarRent111111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "args",
          "type": {
            "defined": {
              "name": "InitPolicyArgs"
            }
          }
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "HonoraryPosition",
      "discriminator": [
        238,
        164,
        37,
        108,
        84,
        131,
        245,
        25
      ]
    },
    {
      "name": "Policy",
      "discriminator": [
        222,
        135,
        7,
        163,
        235,
        177,
        33,
        68
      ]
    },
    {
      "name": "Progress",
      "discriminator": [
        125,
        4,
        195,
        102,
        134,
        179,
        253,
        6
      ]
    }
  ],
  "events": [
    {
      "name": "CreatorPayoutDayClosed",
      "discriminator": [
        142,
        22,
        53,
        87,
        241,
        61,
        177,
        45
      ]
    },
    {
      "name": "HonoraryPositionInitialized",
      "discriminator": [
        7,
        212,
        240,
        190,
        102,
        41,
        75,
        41
      ]
    },
    {
      "name": "InvestorPayoutPage",
      "discriminator": [
        42,
        53,
        234,
        101,
        225,
        240,
        203,
        121
      ]
    },
    {
      "name": "PolicyInitialized",
      "discriminator": [
        102,
        184,
        59,
        178,
        235,
        69,
        251,
        181
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "QuoteOnlyViolation",
      "msg": "Quote-only guarantee violated by pool or claim result"
    },
    {
      "code": 6001,
      "name": "DailyWindowNotReady",
      "msg": "Daily window not ready"
    },
    {
      "code": 6002,
      "name": "InvalidInvestorPage",
      "msg": "Invalid investor page"
    },
    {
      "code": 6003,
      "name": "CapExceeded",
      "msg": "Cap exceeded"
    },
    {
      "code": 6004,
      "name": "ArithmeticOverflow",
      "msg": "Arithmetic overflow"
    },
    {
      "code": 6005,
      "name": "ConstraintViolation",
      "msg": "Constraint violation"
    },
    {
      "code": 6006,
      "name": "Unauthorized",
      "msg": "Unauthorized"
    }
  ],
  "types": [
    {
      "name": "CrankArgs",
      "docs": [
        "Crank arguments per page."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "page_cursor",
            "docs": [
              "Caller-chosen opaque cursor value for pagination bookkeeping."
            ],
            "type": "u64"
          },
          {
            "name": "is_last_page",
            "docs": [
              "Mark if this is the last page for the day."
            ],
            "type": "bool"
          }
        ]
      }
    },
    {
      "name": "CreatorPayoutDayClosed",
      "docs": [
        "Emitted on day close when routing remainder to creator."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "day",
            "docs": [
              "Day key."
            ],
            "type": "i64"
          },
          {
            "name": "remainder",
            "docs": [
              "Remainder routed to creator."
            ],
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "HonoraryPosition",
      "docs": [
        "The empty ‘honorary’ DAMM v2 fee position (quote-only)."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "owner_pda",
            "docs": [
              "Owner PDA that holds the cp-amm position."
            ],
            "type": "pubkey"
          },
          {
            "name": "position",
            "docs": [
              "Meteora cp-amm position account pubkey."
            ],
            "type": "pubkey"
          },
          {
            "name": "cp_pool",
            "docs": [
              "Bound pool and quote mint for defense-in-depth."
            ],
            "type": "pubkey"
          },
          {
            "name": "quote_mint",
            "docs": [
              "Quote mint bound."
            ],
            "type": "pubkey"
          },
          {
            "name": "bump",
            "docs": [
              "Bump."
            ],
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "HonoraryPositionInitialized",
      "docs": [
        "Emitted when we set up the honorary position binding."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "pool",
            "docs": [
              "Pool account."
            ],
            "type": "pubkey"
          },
          {
            "name": "position",
            "docs": [
              "Position account."
            ],
            "type": "pubkey"
          },
          {
            "name": "owner_pda",
            "docs": [
              "PDA owner of the position."
            ],
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "InitPolicyArgs",
      "docs": [
        "Init policy arguments."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "y0_total",
            "docs": [
              "Total investor allocation minted at TGE (Y0)."
            ],
            "type": "u64"
          },
          {
            "name": "investor_fee_share_bps",
            "docs": [
              "Max investor share in bps (<= 10_000)."
            ],
            "type": "u16"
          },
          {
            "name": "daily_cap_quote",
            "docs": [
              "Optional per-day cap in quote lamports (0 disables cap)."
            ],
            "type": "u64"
          },
          {
            "name": "min_payout_lamports",
            "docs": [
              "Dust threshold."
            ],
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "InvestorPayoutPage",
      "docs": [
        "Emitted when a page of investors was paid."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "day",
            "docs": [
              "Current day key (floor(ts/86400))."
            ],
            "type": "i64"
          },
          {
            "name": "page_cursor",
            "docs": [
              "Cursor supplied by caller."
            ],
            "type": "u64"
          },
          {
            "name": "investors",
            "docs": [
              "Number of investors in page."
            ],
            "type": "u32"
          },
          {
            "name": "paid_total",
            "docs": [
              "Total paid to investors this page."
            ],
            "type": "u64"
          },
          {
            "name": "carry_after",
            "docs": [
              "Carry remainder after payouts."
            ],
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "Policy",
      "docs": [
        "Global policy per pool (immutable except by authority)."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "authority",
            "docs": [
              "Program authority that may update policy parameters."
            ],
            "type": "pubkey"
          },
          {
            "name": "cp_pool",
            "docs": [
              "Meteora cp-amm pool bound to this policy."
            ],
            "type": "pubkey"
          },
          {
            "name": "quote_mint",
            "docs": [
              "Quote mint; must match pool quote side."
            ],
            "type": "pubkey"
          },
          {
            "name": "creator_quote_ata",
            "docs": [
              "Creator quote ATA to receive day-end remainder."
            ],
            "type": "pubkey"
          },
          {
            "name": "treasury_quote_ata",
            "docs": [
              "Program treasury ATA (quote) owned by vault PDA."
            ],
            "type": "pubkey"
          },
          {
            "name": "investor_fee_share_bps",
            "docs": [
              "Max investor fee share in basis points (<= 10_000)."
            ],
            "type": "u16"
          },
          {
            "name": "y0_total",
            "docs": [
              "Y0 total allocation used for eligibility scaling."
            ],
            "type": "u64"
          },
          {
            "name": "daily_cap_quote",
            "docs": [
              "Daily cap in quote lamports; 0 disables cap."
            ],
            "type": "u64"
          },
          {
            "name": "min_payout_lamports",
            "docs": [
              "Minimum per-investor payout; smaller amounts are carried."
            ],
            "type": "u64"
          },
          {
            "name": "bump",
            "docs": [
              "Bump for PDA derivation."
            ],
            "type": "u8"
          },
          {
            "name": "initialized",
            "docs": [
              "Whether initialized (sticky true after init)."
            ],
            "type": "bool"
          }
        ]
      }
    },
    {
      "name": "PolicyInitialized",
      "docs": [
        "Emitted after init with a config hash for auditability."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "policy",
            "docs": [
              "Policy account."
            ],
            "type": "pubkey"
          },
          {
            "name": "config_hash",
            "docs": [
              "Keccak hash of key config fields."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          }
        ]
      }
    },
    {
      "name": "Progress",
      "docs": [
        "Tracks idempotent, paginated daily distribution."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "current_day",
            "docs": [
              "Unix day (UTC) we’re currently distributing (floor(ts/86400))."
            ],
            "type": "i64"
          },
          {
            "name": "last_distribution_ts",
            "docs": [
              "Last distribution unix timestamp."
            ],
            "type": "i64"
          },
          {
            "name": "claimed_quote_today",
            "docs": [
              "Total claimed quote this day (from cp-amm)."
            ],
            "type": "u64"
          },
          {
            "name": "distributed_quote_today",
            "docs": [
              "Total distributed to investors today."
            ],
            "type": "u64"
          },
          {
            "name": "carry_quote_today",
            "docs": [
              "Remainder carry within the same day across pages."
            ],
            "type": "u64"
          },
          {
            "name": "page_cursor",
            "docs": [
              "Pagination cursor (opaque, provided by caller)."
            ],
            "type": "u64"
          },
          {
            "name": "day_closed",
            "docs": [
              "True once day’s final page is settled (creator remainder routed)."
            ],
            "type": "bool"
          },
          {
            "name": "bump",
            "docs": [
              "Bump for PDA derivation."
            ],
            "type": "u8"
          }
        ]
      }
    }
  ]
};

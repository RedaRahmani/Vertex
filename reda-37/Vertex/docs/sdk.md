# SDK Overview

## Rust

The Rust SDK provides typed instruction builders for each program and integrates with `anchor-client`.

```rust
use keystone_sdk::launchpad;
use keystone_launchpad::accounts;
use solana_program::pubkey::Pubkey;

let ix = launchpad::buy(
    keystone_launchpad::ID,
    accounts::Buy {
        buyer,
        quote_account,
        buyer_receipt,
        launch_config,
        treasury_authority,
        treasury_vault,
        sale_state,
        mint,
        token_program,
    },
    1_000,
    None,
    1_500,
);
```

## TypeScript

The TypeScript SDK focuses on high-level flows using Anchor providers.

```ts
import { AnchorProvider, Idl } from '@coral-xyz/anchor';
import { buildLaunchpadBuyInstruction } from '@keystone-labs/vertex-sdk';

const ix = await buildLaunchpadBuyInstruction({
  provider: AnchorProvider.env(),
  programId,
  idl: launchpadIdl as Idl,
  amount: BigInt(1_000_000_000),
  maxQuote: BigInt(1_200_000_000),
  accounts: {
    buyer,
    quoteAccount,
    buyerReceipt,
    launchConfig,
    treasuryVault,
    saleState,
    mint,
    tokenProgram
  }
});
```

Additional cookbook examples live under `sdk/ts/examples` (TBD).

#!/bin/bash

# Environment setup
POLICY_PROGRAM="B4yaCkpGZB9Xnm2ZRcj9k1stdkXzkCJdbU26EWj8h7Dc"
CP_POOL="QnCi53pPuxT1ud9wysgFBRZk3yewF4vKhmDjfuvyqL3"
QUOTE_MINT="F78WQiHNwnQXHZXkzhBd6WdKB2hKb14E3opQnkJSZ6CM"
CREATOR_ATA="2smoPgXg4HBzduPgDH8zHP2UvX8KKr2kqSwx5FBUzRZf"

# Find the policy PDA
POLICY=$(solana address-lookup --skip-seed-phrase-validation policy $CP_POOL --program-id $POLICY_PROGRAM)
echo "Policy PDA: $POLICY"

# Initialize the policy
solana program-call $POLICY_PROGRAM init_policy \
    --args policy:$POLICY \
    --args authority:$(solana address) \
    --args cp_pool:$CP_POOL \
    --args quote_mint:$QUOTE_MINT \
    --args creator_quote_ata:$CREATOR_ATA \
    --args treasury_quote_ata:$CREATOR_ATA
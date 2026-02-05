ERROR: The arguments provided to a program instruction were invalid

PROGRAM LOGS:
 22222222222222222222222222222222222222222222 invoke [1]
 ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL invoke [2]
 log: Create
 TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]
 log: Instruction: GetAccountDataSize
 TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 1569 of 1392488 compute units
 return: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA pQAAAAAAAAA=
 TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
 11111111111111111111111111111111 invoke [3]
 11111111111111111111111111111111 success
 log: Initialize the associated token account
 TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]
 log: Instruction: InitializeImmutableOwner
 log: Please upgrade to SPL Token 2022 for immutable owner support
 TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 1405 of 1385901 compute units
 TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
 TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]
 log: Instruction: InitializeAccount3
 TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4188 of 1382019 compute units
 TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
 ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL consumed 20337 of 1397885 compute units
 ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL success
 ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL invoke [2]
 log: Create
 TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]
 log: Instruction: GetAccountDataSize
 TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 1569 of 1370258 compute units
 return: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA pQAAAAAAAAA=
 TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
 11111111111111111111111111111111 invoke [3]
 11111111111111111111111111111111 success
 log: Initialize the associated token account
 TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]
 log: Instruction: InitializeImmutableOwner
 log: Please upgrade to SPL Token 2022 for immutable owner support
 TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 1405 of 1363671 compute units
 TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
 TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]
 log: Instruction: InitializeAccount3
 TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4188 of 1359787 compute units
 TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
 ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL consumed 20438 of 1375733 compute units
 ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL success
 log: [Take] process start
 log: [Take] vault token amount (before transfer)：100000000
 log: [Take] vault lamports (before close)：2039280
 log: [Take] escrow PDA lamports (before close):1677360
 TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]
 log: Instruction: Transfer
 TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4645 of 1351368 compute units
 TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
 log: [Take] vault -> taker_ata_a transfer ok
 TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]
 log: Instruction: CloseAccount
 TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 3015 of 1345328 compute units
 TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
 log: [Take] vault closed, vault lamports after CloseAccount
 TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]
 log: Instruction: Transfer
 TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4645 of 1340918 compute units
 TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
 log: [Take] taker_ata_b -> maker_ata_b transfer ok
 log: [close] start
 11111111111111111111111111111111 invoke [2]
r: `from` must not carry data
 11111111111111111111111111111111 failed: invalid program argument
 22222222222222222222222222222222222222222222 consumed 65209 of 1400000 compute units
 22222222222222222222222222222222222222222222 failed: invalid program argument
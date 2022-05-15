// import * as anchor from "@project-serum/anchor";
// import { Program } from "@project-serum/anchor";
// import {
//   ASSOCIATED_TOKEN_PROGRAM_ID,
//   Token,
//   TOKEN_PROGRAM_ID,
// } from "@solana/spl-token";
// import {
//   SystemProgram,
//   SYSVAR_CLOCK_PUBKEY,
//   SYSVAR_RENT_PUBKEY,
// } from "@solana/web3.js";
// import asset from "assert";
// import {
//   feeAmountPerContract,
//   FEE_OWNER_KEY,
// } from "../../packages/options-ts/src/fees";
// import { OptionMarketV2 } from "../../packages/options-ts/src/types";
// import { CpiExamples } from "../../target/types/cpi_examples";
// import { OptionsTrading } from "../../target/types/options_trading";
// import { createMinter, initOptionMarket, initSetup } from "../../utils/helper";

// const textEncoder = new TextEncoder();
// let optionMarket: OptionMarketV2,
//   underlyingToken: Token,
//   quoteToken: Token,
//   optionToken: Token;
// let vault: anchor.web3.PublicKey,
//   vaultAuthority: anchor.web3.PublicKey,
//   _vaultBump: number,
//   vaultAuhorityBump: number;

// describe("cpi_examples mint", () => {
//   const payer = anchor.web3.Keypair.generate();
//   const user = anchor.web3.Keypair.generate();
//   const mintAuthority = anchor.web3.Keypair.generate();
//   const program = anchor.workspace.CpiExamples as Program<CpiExamples>;
//   const provider = program.provider;
//   const optionsProgram = anchor.workspace
//     .OptionsTrading as Program<OptionsTrading>;

//   before(async () => {
//     await provider.connection.confirmTransaction(
//       await provider.connection.requestAirdrop(payer.publicKey, 10_000_000_000),
//       "confirmed"
//     );

//     await provider.connection.confirmTransaction(
//       await provider.connection.requestAirdrop(user.publicKey, 10_000_000_000),
//       "confirmed"
//     );

//     const {
//       instructions,
//       optionMarket: newOptionMarket,
//       optionMarketKey: _optionMarketKey,
//       optionToken: _optionToken,
//       quoteToken: _quoteToken,
//       underlyingToken: _underlyingToken,
//       remainingAccounts,
//     } = await initSetup(
//       provider,
//       (provider.wallet as anchor.Wallet).payer,
//       mintAuthority,
//       optionsProgram
//     );
//     optionMarket = newOptionMarket;
//     quoteToken = _quoteToken;
//     underlyingToken = _underlyingToken;
//     optionToken = _optionToken;

//     await initOptionMarket(
//       optionsProgram,
//       (provider.wallet as anchor.Wallet).payer,
//       optionMarket,
//       remainingAccounts,
//       instructions
//     );
//   });
// });

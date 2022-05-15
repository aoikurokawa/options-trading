import * as anchor from "@project-serum/anchor";
import { Program, AnchorError } from "@project-serum/anchor";
import { Token, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import {
  AccountMeta,
  TransactionInstruction,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import assert from "assert";

import { OptionMarketV2 } from "../packages/options-ts/src/types";
import { OptionsTrading } from "../target/types/options_trading";
import { initNewTokenMint, initOptionMarket, initSetup } from "../utils/helper";

describe("options-trading", () => {
  // Configure the client to use the local cluster.
  const payer = anchor.web3.Keypair.generate();
  const mintAuthority = anchor.web3.Keypair.generate();
  const program = anchor.workspace.OptionsTrading as Program<OptionsTrading>;
  const provider = program.provider;

  let quoteToken: Token;
  let underlyingToken: Token;
  let optionToken: Token;
  let underlyingAmountPerContract: anchor.BN;
  let quoteAmountPerContract: anchor.BN;
  let expiration: anchor.BN;
  let optionMarket: OptionMarketV2;
  let remainingAccounts: AccountMeta[] = [];
  let instructions: TransactionInstruction[] = [];

  beforeEach(async () => {
    underlyingAmountPerContract = new anchor.BN("10_000_000_000");
    quoteAmountPerContract = new anchor.BN("50_000_000_000");
    expiration = new anchor.BN(new Date().getTime() / 1000 + 3600);
    remainingAccounts = [];
    instructions = [];
    // airdrop to the user so it has funds to use
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(payer.publicKey, 10_000_000_000),
      "confirmed"
    );
  });

  describe("good account setup", () => {
    beforeEach(async () => {
      ({
        quoteToken,
        underlyingToken,
        optionToken,
        optionMarket,
        remainingAccounts,
        instructions,
      } = await initSetup(provider, payer, mintAuthority, program, {}));
    });

    it("Creates new OptionMarket", async () => {
      try {
        await initOptionMarket(
          program,
          payer,
          optionMarket,
          remainingAccounts,
          instructions
        );
      } catch (err) {
        console.error((err as AnchorError).error.errorMessage);
        throw err;
      }

      // Fetch the account for the newly created OptionMarket
      const onChainOptionMarket = (await program.account.optionMarket.fetch(
        optionMarket.key
      )) as OptionMarketV2;

      assert.equal(
        onChainOptionMarket.underlyingAssetMint?.toString(),
        underlyingToken.publicKey.toString()
      );
    });
  });

  //   anchor.setProvider(anchor.AnchorProvider.env());

  //   const program = anchor.workspace.OptionsTrading as Program<OptionsTrading>;

  //   it("Is initialized!", async () => {
  //     // Add your test here.
  //     const tx = await program.methods.initialize().rpc();
  //     console.log("Your transaction signature", tx);
  //   });
});

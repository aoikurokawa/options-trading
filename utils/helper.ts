import * as anchor from "@project-serum/anchor";
import { BN, Program, Provider } from "@project-serum/anchor";
import {
  AccountLayout,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  MintLayout,
  Token,
  TOKEN_PROGRAM_ID,
  u64,
} from "@solana/spl-token";
import {
  AccountMeta,
  Connection,
  Keypair,
  PublicKey,
  sendAndConfirmTransaction,
  SystemProgram,
  SYSVAR_CLOCK_PUBKEY,
  SYSVAR_RENT_PUBKEY,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";
import { getOrAddAssociatedTokenAccountTx } from "../packages/options-ts/src";
import {
  feeAmountPerContract,
  FEE_OWNER_KEY,
} from "../packages/options-ts/src/fees";
import { OptionMarketV2 } from "../packages/options-ts/src/types";
import { OptionsTrading } from "../target/types/options_trading";

export const wait = (delayMS: number) =>
  new Promise((resolve) => setTimeout(resolve, delayMS));

export const createUnderlyingAndQuoteMints = async (
  provider: Provider,
  wallet: Keypair,
  mintAuthority: Keypair
) => {
  const underlyingToken = await Token.createMint(
    provider.connection,
    wallet,
    mintAuthority.publicKey,
    null,
    0,
    TOKEN_PROGRAM_ID
  );

  const quoteToken = await Token.createMint(
    provider.connection,
    wallet,
    mintAuthority.publicKey,
    null,
    0,
    TOKEN_PROGRAM_ID
  );

  return {
    quoteToken,
    underlyingToken,
  };
};

export const initNewTokenMint = async (
  connection: Connection,
  owner: PublicKey,
  wallet: Keypair
) => {
  const mintAccount = new Keypair();
  const transaction = new Transaction();

  const mintRentBalance = await connection.getMinimumBalanceForRentExemption(
    MintLayout.span
  );

  transaction.add(
    SystemProgram.createAccount({
      fromPubkey: wallet.publicKey,
      newAccountPubkey: mintAccount.publicKey,
      lamports: mintRentBalance,
      space: MintLayout.span,
      programId: TOKEN_PROGRAM_ID,
    })
  );

  transaction.add(
    Token.createInitMintInstruction(
      TOKEN_PROGRAM_ID,
      mintAccount.publicKey,
      8,
      owner,
      null
    )
  );

  await sendAndConfirmTransaction(
    connection,
    transaction,
    [wallet, mintAccount],
    {
      commitment: "confirmed",
    }
  );

  return {
    mintAccount,
  };
};

export const initNewTokenAccount = async (
  connection: Connection,
  owner: PublicKey,
  mint: PublicKey,
  wallet: Keypair
) => {
  const tokenAccount = new Keypair();
  const transaction = new Transaction();

  const assetPoolRentBalance =
    await connection.getMinimumBalanceForRentExemption(AccountLayout.span);

  transaction.add(
    SystemProgram.createAccount({
      fromPubkey: wallet.publicKey,
      newAccountPubkey: tokenAccount.publicKey,
      lamports: assetPoolRentBalance,
      space: AccountLayout.span,
      programId: TOKEN_PROGRAM_ID,
    })
  );

  transaction.add(
    Token.createInitAccountInstruction(
      TOKEN_PROGRAM_ID,
      mint,
      tokenAccount.publicKey,
      owner
    )
  );

  await sendAndConfirmTransaction(
    connection,
    transaction,
    [wallet, tokenAccount],
    {
      commitment: "confirmed",
    }
  );

  return {
    tokenAccount,
  };
};

/**
 *
 * TODO: This should be trasformed to use associated token program accounts. That will make it easier.
 *
 * @param connection
 * @param minter
 * @param mintAuthority
 * @param underlyingToken
 * @param underlyingAmount
 * @param optionMint
 * @param writerTokenMint
 * @param quoteToken
 * @param quoteAmount
 * @returns
 */
export const createMinter = async (
  connection: Connection,
  minter: Keypair,
  mintAuthority: Keypair,
  underlyingToken: Token,
  underlyingAmount: number,
  optionMint: PublicKey,
  writerTokenMint: PublicKey,
  quoteToken: Token,
  quoteAmount: number = 0
) => {
  const transaction = new Transaction();

  const underlyingAccount = new Keypair();
  const assetPoolRentBalance =
    await connection.getMinimumBalanceForRentExemption(AccountLayout.span);
  transaction.add(
    SystemProgram.createAccount({
      fromPubkey: minter.publicKey,
      newAccountPubkey: underlyingAccount.publicKey,
      lamports: assetPoolRentBalance,
      space: AccountLayout.span,
      programId: TOKEN_PROGRAM_ID,
    })
  );

  transaction.add(
    Token.createInitAccountInstruction(
      TOKEN_PROGRAM_ID,
      underlyingToken.publicKey,
      underlyingAccount.publicKey,
      minter.publicKey
    )
  );

  const quoteAccount = new Keypair();
  transaction.add(
    SystemProgram.createAccount({
      fromPubkey: minter.publicKey,
      newAccountPubkey: quoteAccount.publicKey,
      lamports: assetPoolRentBalance,
      space: AccountLayout.span,
      programId: TOKEN_PROGRAM_ID,
    })
  );
  transaction.add(
    Token.createInitAccountInstruction(
      TOKEN_PROGRAM_ID,
      quoteToken.publicKey,
      quoteAccount.publicKey,
      minter.publicKey
    )
  );

  const optionAccount = new Keypair();
  transaction.add(
    SystemProgram.createAccount({
      fromPubkey: minter.publicKey,
      newAccountPubkey: optionAccount.publicKey,
      lamports: assetPoolRentBalance,
      space: AccountLayout.span,
      programId: TOKEN_PROGRAM_ID,
    })
  );
  transaction.add(
    Token.createInitAccountInstruction(
      TOKEN_PROGRAM_ID,
      optionMint,
      optionAccount.publicKey,
      minter.publicKey
    )
  );

  const writerTokenAccount = new Keypair();
  transaction.add(
    SystemProgram.createAccount({
      fromPubkey: minter.publicKey,
      newAccountPubkey: writerTokenAccount.publicKey,
      lamports: assetPoolRentBalance,
      space: AccountLayout.span,
      programId: TOKEN_PROGRAM_ID,
    })
  );
  transaction.add(
    Token.createInitAccountInstruction(
      TOKEN_PROGRAM_ID,
      writerTokenMint,
      writerTokenAccount.publicKey,
      minter.publicKey
    )
  );

  await sendAndConfirmTransaction(
    connection,
    transaction,
    [
      minter,
      underlyingAccount,
      quoteAccount,
      optionAccount,
      writerTokenAccount,
    ],
    {
      commitment: "confirmed",
    }
  );

  // mint underlying tokens to the minter's account
  await underlyingToken.mintTo(
    underlyingAccount.publicKey,
    mintAuthority,
    [],
    underlyingAmount
  );

  if (quoteAmount > 0) {
    await quoteToken.mintTo(
      quoteAccount.publicKey,
      mintAuthority,
      [],
      quoteAmount
    );
  }

  return { optionAccount, quoteAccount, underlyingAccount, writerTokenAccount };
};

export const createExerciser = async (
  connection: Connection,
  exerciser: Keypair,
  mintAuthority: Keypair,
  quoteToken: Token,
  quoteAmount: number,
  optionMint: PublicKey,
  underlyingTokenMint: PublicKey
) => {
  const transaction = new Transaction();

  const quoteAccount = new Keypair();
  const assetPoolRentBalance =
    await connection.getMinimumBalanceForRentExemption(AccountLayout.span);

  transaction.add(
    SystemProgram.createAccount({
      fromPubkey: exerciser.publicKey,
      newAccountPubkey: quoteAccount.publicKey,
      lamports: assetPoolRentBalance,
      space: AccountLayout.span,
      programId: TOKEN_PROGRAM_ID,
    })
  );

  transaction.add(
    Token.createInitAccountInstruction(
      TOKEN_PROGRAM_ID,
      quoteToken.publicKey,
      quoteAccount.publicKey,
      exerciser.publicKey
    )
  );

  // create an associated token account to hold the options
  const optionAccount = new Keypair();
  transaction.add(
    SystemProgram.createAccount({
      fromPubkey: exerciser.publicKey,
      newAccountPubkey: optionAccount.publicKey,
      lamports: assetPoolRentBalance,
      space: AccountLayout.span,
      programId: TOKEN_PROGRAM_ID,
    })
  );
  transaction.add(
    Token.createInitAccountInstruction(
      TOKEN_PROGRAM_ID,
      optionMint,
      optionAccount.publicKey,
      exerciser.publicKey
    )
  );

  // create an associated token account to hold the underlying tokens
  const underlyingAccount = new Keypair();
  transaction.add(
    SystemProgram.createAccount({
      fromPubkey: exerciser.publicKey,
      newAccountPubkey: underlyingAccount.publicKey,
      lamports: assetPoolRentBalance,
      space: AccountLayout.span,
      programId: TOKEN_PROGRAM_ID,
    })
  );
  transaction.add(
    Token.createInitAccountInstruction(
      TOKEN_PROGRAM_ID,
      underlyingTokenMint,
      underlyingAccount.publicKey,
      exerciser.publicKey
    )
  );
  await sendAndConfirmTransaction(
    connection,
    transaction,
    [exerciser, quoteAccount, optionAccount, underlyingAccount],
    {
      commitment: "confirmed",
    }
  );

  // mint underlying tokens to the minter's account
  await quoteToken.mintTo(
    quoteAccount.publicKey,
    mintAuthority,
    [],
    quoteAmount
  );
  return { optionAccount, quoteAccount, underlyingAccount };
};

export const initSetup = async (
  provider: anchor.Provider,
  payer: Keypair,
  mintAuthority: Keypair,
  program: anchor.Program<OptionsTrading>,
  opts: {
    underlyingAmountPerContract?: anchor.BN;
    quoteAmountPerContract?: anchor.BN;
    mintFeeToken?: Token;
    exerciseFeeToken?: Token;
    mintFeeOwner?: PublicKey;
    exerciseFeeOwner?: PublicKey;
    expiration?: anchor.BN;
  } = {}
) => {
  const textEncoder = new TextEncoder();
  let quoteToken: Token;
  let underlyingToken: Token;
  let underlyingAmountPerContract =
    opts.underlyingAmountPerContract || new anchor.BN("10000000000");
  let quoteAmountPerContract =
    opts.quoteAmountPerContract || new anchor.BN("50000000000");
  let expiration =
    opts.expiration || new anchor.BN(new Date().getTime() / 1000 + 3600);
  let optionMarketkey: PublicKey;
  let bumpSeed: number;
  let mintFeeKey = new Keypair().publicKey;
  let exerciseFeeKey = new Keypair().publicKey;
  let remainingAccounts: AccountMeta[] = [];
  let instructions: TransactionInstruction[] = [];
  ({ underlyingToken, quoteToken } = await createUnderlyingAndQuoteMints(
    provider,
    payer,
    mintAuthority
  ));
  [optionMarketkey, bumpSeed] = await anchor.web3.PublicKey.findProgramAddress(
    [
      underlyingToken.publicKey.toBuffer(),
      quoteToken.publicKey.toBuffer(),
      underlyingAmountPerContract.toBuffer("le", 8),
      quoteAmountPerContract.toBuffer("le", 8),
      expiration.toBuffer("le", 8),
    ],
    program.programId
  );

  const [optionMintKey, optionMintBump] =
    await anchor.web3.PublicKey.findProgramAddress(
      [optionMarketkey.toBuffer(), textEncoder.encode("optionToken")],
      program.programId
    );

  const [writeMintKey, writerMintBump] =
    await anchor.web3.PublicKey.findProgramAddress(
      [optionMarketkey.toBuffer(), textEncoder.encode("writerToken")],
      program.programId
    );

  const [quoteAssetPoolKey, quoteAssetPoolBump] =
    await anchor.web3.PublicKey.findProgramAddress(
      [optionMarketkey.toBuffer(), textEncoder.encode("quoteAssetPool")],
      program.programId
    );

  const [underlyingAssetPoolKey, underlyingAssetPoolBump] =
    await anchor.web3.PublicKey.findProgramAddress(
      [optionMarketkey.toBuffer(), textEncoder.encode("underlyingAssetPool")],
      program.programId
    );

  // Get the associated fee address if the market requires a fee
  const mintFeePerContract = feeAmountPerContract(underlyingAmountPerContract);
  if (mintFeePerContract.gtn(0)) {
    mintFeeKey = await Token.getAssociatedTokenAddress(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      opts.mintFeeToken?.publicKey || underlyingToken.publicKey,
      opts.mintFeeOwner || FEE_OWNER_KEY
    );

    remainingAccounts.push({
      pubkey: mintFeeKey,
      isWritable: true,
      isSigner: false,
    });
    const ix = await getOrAddAssociatedTokenAccountTx(
      mintFeeKey,
      opts.mintFeeToken || underlyingToken,
      payer.publicKey,
      opts.mintFeeOwner || FEE_OWNER_KEY
    );
    if (ix) {
      instructions.push(ix);
    }
  }

  const exerciseFee = feeAmountPerContract(quoteAmountPerContract);
  if (exerciseFee.gtn(0)) {
    exerciseFeeKey = await Token.getAssociatedTokenAddress(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      opts.exerciseFeeToken?.publicKey || quoteToken.publicKey,
      opts.exerciseFeeOwner || FEE_OWNER_KEY
    );
    remainingAccounts.push({
      pubkey: exerciseFeeKey,
      isWritable: false,
      isSigner: false,
    });
    const ix = await getOrAddAssociatedTokenAccountTx(
      exerciseFeeKey,
      opts.exerciseFeeToken || quoteToken,
      payer.publicKey,
      opts.exerciseFeeOwner || FEE_OWNER_KEY
    );
    if (ix) {
      instructions.push(ix);
    }
  }

  const optionMarket: OptionMarketV2 = {
    key: optionMarketkey,
    optionMint: optionMintKey,
    writerTokenMint: writeMintKey,
    underlyingAssetMint: underlyingToken.publicKey,
    quoteAssetMint: quoteToken.publicKey,
    underlyingAssetPool: underlyingAssetPoolKey,
    quoteAssetPool: quoteAssetPoolKey,
    mintFeeAccount: mintFeeKey,
    exerciseFeeAccount: exerciseFeeKey,
    underlyingAmountPerContract,
    quoteAmountPerContract,
    expirationUnixTimestamp: expiration,
    expired: false,
    bumpSeed,
  };

  const optionToken = new Token(
    provider.connection,
    optionMintKey,
    TOKEN_PROGRAM_ID,
    payer
  );

  return {
    quoteToken,
    underlyingToken,
    optionToken,
    underlyingAmountPerContract,
    quoteAmountPerContract,
    expiration,
    optionMarketkey,
    bumpSeed,
    mintFeeKey,
    exerciseFeeKey,
    optionMintKey,
    writeMintKey,
    underlyingAssetPoolKey,
    quoteAssetPoolKey,
    optionMarket,
    remainingAccounts,
    instructions,
  };
};

export const initOptionMarket = async (
  program: anchor.Program<OptionsTrading>,
  payer: Keypair,
  optionMarket: OptionMarketV2,
  remainingAccounts: AccountMeta[],
  instructions: TransactionInstruction[]
) => {
  await program.methods
    .initializeMarket(
      optionMarket.underlyingAmountPerContract,
      optionMarket.quoteAmountPerContract,
      optionMarket.expirationUnixTimestamp,
      optionMarket.bumpSeed
    )
    .accounts({
      authority: payer.publicKey,
      underlyingAssetMint: optionMarket.underlyingAssetMint,
      quoteAssetMint: optionMarket.quoteAssetMint,
      optionMint: optionMarket.optionMint,
      writerTokenMint: optionMarket.writerTokenMint,
      quoteAssetPool: optionMarket.quoteAssetPool,
      underlyingAssetPool: optionMarket.underlyingAssetPool,
      optionMarket: optionMarket.key,
      feeOwner: FEE_OWNER_KEY,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
      systemProgram: SystemProgram.programId,
      clock: SYSVAR_CLOCK_PUBKEY,
    })
    .remainingAccounts(remainingAccounts)
    .signers([payer])
    .instruction()
    .rpc();
};

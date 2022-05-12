import { PublicKey } from "@solana/web3.js";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  Token,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { FEE_OWNER_KEY } from "./fees";

export const getOrAddAssociatedTokenAccountTx = async (
  associatedAddress: PublicKey,
  token: Token,
  payer: PublicKey,
  owner: PublicKey = FEE_OWNER_KEY
) => {
  try {
    await token.getAccountInfo(associatedAddress);
    return null;
  } catch (err: any) {
    if (
      err.message === "Failed to find account" ||
      err.message === "Invalid account owner"
    ) {
      return Token.createAssociatedTokenAccountInstruction(
        ASSOCIATED_TOKEN_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
        token.publicKey,
        associatedAddress,
        owner,
        payer
      );
    } else {
      err;
    }
  }
};

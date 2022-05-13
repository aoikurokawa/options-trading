/* eslint-disable max-classes-per-file */
import * as BufferLayout from "buffer-layout";
import { PublicKey } from "@solana/web3.js";
import { BN } from "@project-serum/anchor";
import { blob, Layout } from "@solana/buffer-layout";

// class PublicKeyLayout extends BufferLayout.Blob {
//   constructor(property: string) {
//     super(32, property);
//   }

//   decode(b: Buffer, offset?: number) {
//     return new PublicKey(super.decode(b, offset));
//   }

//   encode(src: any, b: Buffer, offset?: number) {
//     const srcBuf = src.toBuffer();
//     let span = this.length;
//     if (this.length instanceof BufferLayout.ExternalLayout) {
//       span = srcBuf.length;
//     }
//     if (span !== srcBuf.length) {
//       throw new TypeError("wrong length for pubkey");
//     }
//     if (offset + span > b.length) {
//       throw new RangeError("encoding overruns Buffer");
//     }
//     b.write(srcBuf.toString("hex"), offset, span, "hex");
//     if (this.length instanceof BufferLayout.ExternalLayout) {
//       this.length.encode(span, b, offset);
//     }
//     return span;
//   }
// }

class BNLayout extends BufferLayout.Blob {
  decode(b: Buffer, offset?: number) {
    return new BN(super.decode(b, offset), 10, "le");
  }

  encode(src: BN, b: Buffer, offset?: number) {
    return super.encode(src.toArrayLike(Buffer, "le", this.span), b, offset);
  }
}

// eslint-disable-next-line no-shadow
export enum AccountType {
  Market = 0,
  Registry = 1,
}

export const accountType = (property: string) => BufferLayout.u8(property);
/**
 * Layout for a public key
 */
// export const publicKey = (property: string) => new PublicKeyLayout(property);

/**
 * Layout for a 64bit unsigned value
 */
export const uint64 = (property: string) => new BNLayout(8, property);

/**
 * Layout for the OptionInstruction tag
 */
export const INTRUCTION_TAG_LAYOUT = BufferLayout.u8("instructionTag");

export const publicKey = (property?: string): Layout<PublicKey> => {
  const layout = blob(32, property);
  const { encode, decode } = encodeDecode(layout);

  const publicKeyLayout = layout as Layout<unknown> as Layout<PublicKey>;

  publicKeyLayout.decode = (buffer: Buffer, offset: number) => {
    const src = decode(buffer, offset);
    return new PublicKey(src);
  };

  publicKeyLayout.encode = (
    publicKey: PublicKey,
    buffer: Buffer,
    offset: number
  ) => {
    const src = publicKey.toBuffer();
    return encode(src, buffer, offset);
  };

  return publicKeyLayout;
};

export interface EncodeDecode<T> {
  decode(buffer: Buffer, offset?: number): T;
  encode(src: T, buffer: Buffer, offset?: number): number;
}

export const encodeDecode = <T>(layout: Layout<T>): EncodeDecode<T> => {
  const decode = layout.decode.bind(layout);
  const encode = layout.encode.bind(layout);
  return { decode, encode };
};

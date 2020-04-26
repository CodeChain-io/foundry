import { exchange, x25519GetPublicFromPrivate } from "..";

describe("exchange", () => {
    // The testing oracle is generated from "libsodium-wrappers"
    const AliceX25519Sk =
        "49023b9f7f997d3c8f1763f762a2f79d3d0fc5d4204ee25bf05ef57ca190d91b";
    const AliceX25519Pk =
        "687d59b5c23877b6e822e3459d5a9a45e801a6a3752cdada2dfa7c76eaadb36f";
    const BobX25519Pk =
        "f039531944bb30f89df41dedfb3ae1a385bc3052bec473060a57f3b2ec248c64";
    const BobX25519Sk =
        "e16af6bbf808b73137ec93b7d26ec2ca59e8e5367ced0cbee11553e8b316a564";

    const shared_secret =
        "3e57e4716cbb9ffbcedce8c9b1516fe8b43dd6688020523b62e9142fc2cc5879";

    test("Alice-side", () => {
        expect(exchange(BobX25519Pk, AliceX25519Sk)).toEqual(shared_secret);
    });

    test("Bob-side", () => {
        expect(exchange(AliceX25519Pk, BobX25519Sk)).toEqual(shared_secret);
    });
});

test("x25519GetPublicFromPrivate", () => {
    // The testing oracle is generated from "libsodium-wrappers"
    const x25519Sk =
        "49023b9f7f997d3c8f1763f762a2f79d3d0fc5d4204ee25bf05ef57ca190d91b";
    const x25519Pk =
        "687d59b5c23877b6e822e3459d5a9a45e801a6a3752cdada2dfa7c76eaadb36f";

    expect(x25519GetPublicFromPrivate(x25519Sk)).toEqual(x25519Pk);
});

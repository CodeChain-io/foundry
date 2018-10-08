import {
    signEcdsa,
    verifyEcdsa,
    recoverEcdsa,
    getPublicFromPrivate
} from "../utils";

const priv = "99053a6568a93b9f194ef983c84ddfa9eb2b37888e47433558d40b2f4770b2d8";
const msg = "00000000c0dec6a100000000c0dec6a100000000c0dec6a100000000c0dec6a1";

test("public key", () => {
    const pub = getPublicFromPrivate(priv);
    expect(pub.length).toBe(128);
});

test("sign", () => {
    const signature = signEcdsa(msg, priv);
    expect(signature).toEqual({
        r: "d8706a863775325b1b8c3f16c19ff337c2699c4f857be903e08a5a9234c5a5c7",
        s: "19d685ae28e52081480b08a3a1e5d8dd1f852b78f65a7e99af37ad42ebc5f375",
        v: 0
    });
});

test("verify - success", () => {
    const result = verifyEcdsa(
        msg,
        {
            r:
                "d8706a863775325b1b8c3f16c19ff337c2699c4f857be903e08a5a9234c5a5c7",
            s:
                "19d685ae28e52081480b08a3a1e5d8dd1f852b78f65a7e99af37ad42ebc5f375",
            v: 0
        },
        getPublicFromPrivate(priv)
    );
    expect(result).toBe(true);
});

test("verify - fail", () => {
    const result = verifyEcdsa(
        "0000000000000000000000000000000000000000000000000000000000000000",
        {
            r:
                "d8706a863775325b1b8c3f16c19ff337c2699c4f857be903e08a5a9234c5a5c7",
            s:
                "19d685ae28e52081480b08a3a1e5d8dd1f852b78f65a7e99af37ad42ebc5f375",
            v: 0
        },
        getPublicFromPrivate(priv)
    );
    expect(result).toBe(false);
});

test("recover", () => {
    const a = recoverEcdsa(msg, {
        r: "d8706a863775325b1b8c3f16c19ff337c2699c4f857be903e08a5a9234c5a5c7",
        s: "19d685ae28e52081480b08a3a1e5d8dd1f852b78f65a7e99af37ad42ebc5f375",
        v: 0
    });
    expect(a).toBe(getPublicFromPrivate(priv));
});

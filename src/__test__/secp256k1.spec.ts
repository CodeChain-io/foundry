import {
    signEcdsa,
    verifyEcdsa,
    recoverEcdsa,
    getPublicFromPrivate
} from "../utils";

const priv = "99053a6568a93b9f194ef983c84ddfa9eb2b37888e47433558d40b2f4770b2d8";
const msg = "hello";

test("public key", () => {
    const pub = getPublicFromPrivate(priv);
    expect(pub.length).toBe(128);
});

test("sign", () => {
    const signature = signEcdsa(msg, priv);
    expect(signature).toEqual({
        r: "7b5e0ee8644c6f585fc297364143280a458445025304ab8f8bd17012e0817189",
        s: "68d7d28f062724c5ec3033d3deb968aeb7eaf2931aeba07c6fea1540065835e3",
        v: 0
    });
});

test("verify - success", () => {
    const result = verifyEcdsa(
        msg,
        {
            r:
                "7b5e0ee8644c6f585fc297364143280a458445025304ab8f8bd17012e0817189",
            s:
                "68d7d28f062724c5ec3033d3deb968aeb7eaf2931aeba07c6fea1540065835e3",
            v: 0
        },
        getPublicFromPrivate(priv)
    );
    expect(result).toBe(true);
});

test("verify - fail", () => {
    const result = verifyEcdsa(
        "hi",
        {
            r:
                "7b5e0ee8644c6f585fc297364143280a458445025304ab8f8bd17012e0817189",
            s:
                "68d7d28f062724c5ec3033d3deb968aeb7eaf2931aeba07c6fea1540065835e3",
            v: 0
        },
        getPublicFromPrivate(priv)
    );
    expect(result).toBe(false);
});

test("recover", () => {
    const a = recoverEcdsa(msg, {
        r: "7b5e0ee8644c6f585fc297364143280a458445025304ab8f8bd17012e0817189",
        s: "68d7d28f062724c5ec3033d3deb968aeb7eaf2931aeba07c6fea1540065835e3",
        v: 0
    });
    expect(a).toBe(getPublicFromPrivate(priv));
});

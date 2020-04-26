import {
    generatePrivateKey,
    getPublicFromPrivate,
    signEd25519,
    verifyEd25519
} from "..";

test("generatePrivateKey", () => {
    const priv = generatePrivateKey();
    expect(/^[0-9a-fA-F]{128}$/.test(priv)).toBe(true);
});

test("getPublicFromPrivate", () => {
    const priv = generatePrivateKey();
    const pub = getPublicFromPrivate(priv);
    expect(/^[0-9a-fA-F]{64}$/.test(pub)).toBe(true);
});

test("sign & verify ECDSA", () => {
    const msg =
        "0000000000000000000000000000000000000000000000000000000000000000";
    const priv = generatePrivateKey();
    const pub = getPublicFromPrivate(priv);
    const sig = signEd25519(msg, priv);
    expect(verifyEd25519(msg, sig, pub)).toBe(true);
});

// Examples are generated from Ed25519 signature in pysodium
describe.each([
    [
        "2730417b940503dfc8dddfe5dfdbfc029b269fec9bc0170a156bcfe30f5afda8",
        "36e13e2debaa6702fd9d5b6804e9c7890735f00a588aa06b7cd832368297b2fddb688c1df2093113bfc6dc1c61984be8dec0e5fa97587ec611f4dcd04182d89a",
        "c61d9d519fc9d158b26072b9c6f46091a04c2492121b0468200136e2f802131d50fd9548c74b7aaf412aa5d3424c9b13e6827510a6dae5670f7e3631a5913802"
    ],
    [
        "52d9ec33be855d9f27e1459dabf195266e1c4ca2bd1f44bbc7c6c0c2ebd0b280",
        "2b7730eb5906b9b01aa45c243aa717acae206568ed6fed07c15ce5d7604992a2437c1f5c797544b0088d0f558db12957be6d4b2c8dd5da64472be798631c1e34",
        "5147bfe6ef37ec7c80bbaf52a27935af5db43a9636101b00df5ce74c679e1ff2bfbcaf44b68ecceb080b9c85b779ce78ae795374b372b64d51ed90959f4d9409"
    ],
    [
        "44fd4a087cbf2a0ef6762cd7de0f3020fd71b11f13afa014741dcf3f098e1de1",
        "aa40e530b371a336b1912e221701ce1f0166bf3e492ab7149e99206d60eab69623ea53cb47deaa10e36fcf1fa85144bfe2b5498397a040874a8c392bee622405",
        "4eaeeff776f9894099f87880ce498b09fca4b0e7c6cee42244e801791a8388726d24fe6edcf7c4e51a97c3de6299ef7e45e7bd4f3df95243b74b52e5752b7a08"
    ],
    [
        "3d52843f74e47b24edd77fcf0b2041ba1b57984eb20a4a17144c08c9588d2a0a",
        "b996d633716c0c185e1b5048b2ca2f79f11e6f8f7c3a2ee7711095f00743f29297fb4dd8375d0e0403c1f72cd40a59d5077f31649a5f472427c5cf315c050d8e",
        "50caa79ba7dcb5303e614bf2cbd6dd931545f048e17295a21861bfc1a94fa12f6365c73fdd6579b88bc81460c83a362b1b62caa06006eb356e6f5d3ceb77270e"
    ],
    [
        "99a819048cc03500e0ad3debeb9beace8e26e5960f7045a551fa5c14247b8805",
        "01ed6bd345d313f35f7b6c50efe991f4a7248e5eece900f8886b60fba5a2b437851b578d9cc0b6ffb256c3f41a7f9fa56915180b03bd2228bbbf1a38a0cfce8c",
        "70400fe11c2e10f3ae1d805c2083268afecaf3864a050f14af8d26023be194dbf66175dd621d592dbabb4f291fe460b4748ef88454b354d4516dc50a0b16b703"
    ]
])("verify Ed25519 with example: %p", (msg, priv, sigStr) => {
    const pub = getPublicFromPrivate(priv);

    test("verify", () => {
        expect(verifyEd25519(msg, sigStr, pub)).toBe(true);
    });
});

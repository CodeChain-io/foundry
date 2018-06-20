test("commonjs", async () => {
    const SDK = require("../");

    expect(SDK).toEqual(expect.any(Function));

    expect(SDK.SDK).toEqual(expect.any(Function));

    expect(SDK.H160).toEqual(expect.any(Function));
    expect(SDK.H256).toEqual(expect.any(Function));
    expect(SDK.H512).toEqual(expect.any(Function));
    expect(SDK.U256).toEqual(expect.any(Function));
    expect(SDK.Parcel).toEqual(expect.any(Function));
    expect(SDK.SignedParcel).toEqual(expect.any(Function));
    expect(SDK.Invoice).toEqual(expect.any(Function));
    expect(SDK.Asset).toEqual(expect.any(Function));
    expect(SDK.AssetScheme).toEqual(expect.any(Function));
    expect(SDK.Block).toEqual(expect.any(Function));

    expect(SDK.AssetMintTransaction).toEqual(expect.any(Function));
    expect(SDK.AssetTransferTransaction).toEqual(expect.any(Function));
    expect(SDK.AssetTransferInput).toEqual(expect.any(Function));
    expect(SDK.AssetOutPoint).toEqual(expect.any(Function));
    expect(SDK.AssetTransferOutput).toEqual(expect.any(Function));
    expect(SDK.getTransactionFromJSON).toEqual(expect.any(Function));

    expect(SDK.blake256).toEqual(expect.any(Function));
    expect(SDK.blake256WithKey).toEqual(expect.any(Function));
    expect(SDK.ripemd160).toEqual(expect.any(Function));
    expect(SDK.signEcdsa).toEqual(expect.any(Function));
    expect(SDK.privateKeyToAddress).toEqual(expect.any(Function));
    expect(SDK.privateKeyToPublic).toEqual(expect.any(Function));
});

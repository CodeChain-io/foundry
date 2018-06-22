import { AssetTransferTransaction, AssetTransferInput, AssetOutPoint, AssetTransferOutput } from "..";
import { H256 } from "../..";

test("AssetTransferTransaction toJSON", () => {
    const t = new AssetTransferTransaction(17, {
        burns: [],
        inputs: [],
        outputs: []
    }, 54321);
    expect(AssetTransferTransaction.fromJSON(t.toJSON())).toEqual(t);
});

test("AssetOutPoint toJSON", () => {
    const outPoint = new AssetOutPoint({
        transactionHash: new H256("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        index: 0,
        assetType: new H256("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
        amount: 1
    });
    expect(AssetOutPoint.fromJSON(outPoint.toJSON())).toEqual(outPoint);
});

test("AssetTransferInput toJSON", () => {
    const outPoint = new AssetOutPoint({
        transactionHash: new H256("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        index: 0,
        assetType: new H256("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
        amount: 1
    });
    const input = new AssetTransferInput({
        prevOut: outPoint,
        lockScript: Buffer.from([0x01, 0x02]),
        unlockScript: Buffer.from([0x03])
    });
    expect(AssetTransferInput.fromJSON(input.toJSON())).toEqual(input);
});

test("AssetTransferOutput toJSON", () => {
    const output = new AssetTransferOutput({
        lockScriptHash: new H256("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        parameters: [Buffer.from([0x04, 0x05]), Buffer.from([0x06])],
        assetType: new H256("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
        amount: 321
    });
    expect(AssetTransferOutput.fromJSON(output.toJSON())).toEqual(output);
});

test("AssetTransferTransaction hashWithoutScript", () => {
    const outPoint = new AssetOutPoint({
        transactionHash: new H256("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        index: 0,
        assetType: new H256("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
        amount: 1
    });
    const input1 = new AssetTransferInput({
        prevOut: outPoint,
        lockScript: Buffer.from([0x01, 0x02]),
        unlockScript: Buffer.from([0x03])
    });
    const input2 = new AssetTransferInput({
        prevOut: outPoint,
        lockScript: Buffer.from([]),
        unlockScript: Buffer.from([])
    });
    const t1 = new AssetTransferTransaction(17, {
        burns: [],
        inputs: [input1],
        outputs: []
    }, 54321);
    const t2 = new AssetTransferTransaction(17, {
        burns: [],
        inputs: [input2],
        outputs: []
    }, 54321);
    expect(t1.hashWithoutScript()).toEqual(t2.hash());
});

import { AssetTransferTransaction } from "../AssetTransferTransaction";
import { H256 } from "../../H256";
import { AssetOutPoint } from "../AssetOutPoint";
import { AssetTransferInput } from "../AssetTransferInput";
import { AssetTransferOutput } from "../AssetTransferOutput";

test("AssetTransferTransaction toJSON", () => {
    const t = new AssetTransferTransaction({
        burns: [],
        inputs: [],
        outputs: [],
        networkId: 17,
        nonce: 54321
    });
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
    const t1 = new AssetTransferTransaction({
        burns: [],
        inputs: [input1],
        outputs: [],
        networkId: 17,
        nonce: 54321
    });
    const t2 = new AssetTransferTransaction({
        burns: [],
        inputs: [input2],
        outputs: [],
        networkId: 17,
        nonce: 54321
    });
    expect(t1.hashWithoutScript()).toEqual(t2.hash());
});

test("AssetTransferOutput shard id", () => {
    const output = new AssetTransferOutput({
        lockScriptHash: new H256("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        parameters: [Buffer.from([0x04, 0x05]), Buffer.from([0x06])],
        assetType: new H256("4100BAADCAFEBEEFbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
        amount: 321
    });
    expect(output.shardId()).toEqual(parseInt("BAAD", 16));
});

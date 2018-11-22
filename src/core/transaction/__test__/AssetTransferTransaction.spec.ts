import { H160 } from "../../H160";
import { H256 } from "../../H256";
import { U64 } from "../../U64";
import { AssetOutPoint } from "../AssetOutPoint";
import { AssetTransferInput } from "../AssetTransferInput";
import { AssetTransferOutput } from "../AssetTransferOutput";
import { AssetTransferTransaction } from "../AssetTransferTransaction";

test("AssetTransferTransaction toJSON", () => {
    const t = new AssetTransferTransaction({
        burns: [],
        inputs: [],
        outputs: [],
        orders: [],
        networkId: "tc"
    });
    expect(AssetTransferTransaction.fromJSON(t.toJSON())).toEqual(t);
});

test("AssetOutPoint toJSON", () => {
    const outPoint = new AssetOutPoint({
        transactionHash: new H256(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        ),
        index: 0,
        assetType: new H256(
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        ),
        amount: new U64(1)
    });
    expect(AssetOutPoint.fromJSON(outPoint.toJSON())).toEqual(outPoint);
});

test("AssetTransferInput toJSON", () => {
    const outPoint = new AssetOutPoint({
        transactionHash: new H256(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        ),
        index: 0,
        assetType: new H256(
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        ),
        amount: new U64(1)
    });
    const input = new AssetTransferInput({
        prevOut: outPoint,
        timelock: null,
        lockScript: Buffer.from([0x01, 0x02]),
        unlockScript: Buffer.from([0x03])
    });
    expect(AssetTransferInput.fromJSON(input.toJSON())).toEqual(input);
});

test("AssetTransferOutput toJSON", () => {
    const output = new AssetTransferOutput({
        lockScriptHash: new H160("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        parameters: [Buffer.from([0x04, 0x05]), Buffer.from([0x06])],
        assetType: new H256(
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        ),
        amount: new U64(321)
    });
    expect(AssetTransferOutput.fromJSON(output.toJSON())).toEqual(output);
});

test("AssetTransferOutput shard id", () => {
    const output = new AssetTransferOutput({
        lockScriptHash: new H160("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        parameters: [Buffer.from([0x04, 0x05]), Buffer.from([0x06])],
        assetType: new H256(
            "4100BAADCAFEBEEFbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        ),
        amount: new U64(321)
    });
    expect(output.shardId()).toEqual(parseInt("BAAD", 16));
});

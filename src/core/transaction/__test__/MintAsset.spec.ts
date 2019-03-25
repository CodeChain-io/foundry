import { H160, U64 } from "codechain-primitives";
import { SignedTransaction } from "../../SignedTransaction";
import { AssetMintOutput } from "../AssetMintOutput";
import { MintAsset } from "../MintAsset";

function createMintAsset(): MintAsset {
    return new MintAsset({
        networkId: "tc",
        shardId: 0,
        metadata: "metadata",
        output: new AssetMintOutput({
            lockScriptHash: H160.zero(),
            parameters: [],
            supply: new U64(1)
        }),
        approver: null,
        registrar: null,
        allowedScriptHashes: [],
        approvals: []
    });
}
test("approval doesn't change the tracker", () => {
    const t1 = createMintAsset();
    const t2 = createMintAsset();
    t2.addApproval("some approval");
    expect(t1.tracker().value).toEqual(t2.tracker().value);
});

test("approval changes the hash", () => {
    const t1 = createMintAsset();
    const t2 = createMintAsset();
    t2.addApproval("someone's approval");
    expect((t1 as any).approvals).not.toEqual((t2 as any).approvals);

    const fee = 10;
    const seq = 20;
    t1.setFee(fee);
    t1.setSeq(seq);
    t2.setFee(fee);
    t2.setSeq(seq);

    const signed1 = new SignedTransaction(t1, "signature");
    const signed2 = new SignedTransaction(t2, "signature");
    expect(signed1.hash().value).not.toEqual(signed2.hash().value);
});

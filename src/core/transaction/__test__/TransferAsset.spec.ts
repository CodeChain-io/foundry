import { SignedTransaction } from "../../SignedTransaction";
import { TransferAsset } from "../TransferAsset";

function createTransferAsset(): TransferAsset {
    return new TransferAsset({
        inputs: [],
        burns: [],
        outputs: [],
        orders: [],
        metadata: "metadata",
        approvals: [],
        expiration: null,
        networkId: "tc"
    });
}
test("approval doesn't change the tracker", () => {
    const t1 = createTransferAsset();
    const t2 = createTransferAsset();
    t2.addApproval("some approval");
    expect(t1.tracker().value).toEqual(t2.tracker().value);
});

test("approval changes the hash", () => {
    const t1 = createTransferAsset();
    const t2 = createTransferAsset();
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

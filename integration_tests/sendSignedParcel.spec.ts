import { SDK } from "../";

import { ACCOUNT_ADDRESS, ACCOUNT_SECRET, SERVER_URL } from "./helper";

const sdk = new SDK({ server: SERVER_URL });

const secret = ACCOUNT_SECRET;
const address = ACCOUNT_ADDRESS;

test("sendSignedTransaction", async () => {
    const seq = await sdk.rpc.chain.getSeq(address);
    const p = sdk.core.createPayTransaction({
        recipient: address,
        amount: 0
    });
    const hash = await sdk.rpc.chain.sendSignedTransaction(
        p.sign({
            secret,
            seq,
            fee: 10
        })
    );
    expect(hash).toMatchObject({
        value: expect.stringMatching(/[0-9a-f]{32}/)
    });
});

import { SDK, H256, PaymentTransaction, AssetMintTransaction } from "../";
import { mintAsset, transferAsset } from "./helper";

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
const sdk = new SDK(SERVER_URL);

test("AssetMintTransaction fromJSON", async () => {
    const metadata = "";
    const lockScriptHash = new H256("0000000000000000000000000000000000000000000000000000000000000000");
    const amount = 100;
    const parameters: Buffer[] = [];
    const registrar = null;
    const { parcelHash } = await mintAsset({ metadata, lockScriptHash, amount, parameters, registrar });
    const parcel = await sdk.getParcel(parcelHash);
    expect(parcel.unsigned.action.transactions[0]).toMatchObject({
        type: expect.stringMatching("assetMint"),
        data: expect.objectContaining({
            metadata: expect.anything(),
            lockScriptHash: expect.any(H256),
            // FIXME: Buffer[]
            parameters: expect.anything(),
            // FIXME: Change it to U256
            amount: expect.anything(),
            // FIXME: null or H160
            registrar: null,
            nonce: expect.anything()
        })
    });
});

test("AssetTransferTransaction fromJSON", async () => {
    const emptyLockScriptHash = new H256("50a2c0d145539c1fb32f60e0d8425b1c03f6120c40171971b8de9c0017a4bfb3");
    const mint = new AssetMintTransaction({
        metadata: "metadata of non-permissioned asset",
        lockScriptHash: emptyLockScriptHash,
        parameters: [],
        amount: 100,
        registrar: null,
        nonce: 0,
    });

    const { parcelHash } = await transferAsset({ mintTx: mint });
    const parcel = await sdk.getParcel(parcelHash);
    expect(parcel.unsigned.action.transactions[1]).toMatchObject({
        type: expect.stringMatching("assetTransfer"),
        inputs: expect.arrayContaining([{
            prevOut: expect.objectContaining({
                data: expect.objectContaining({
                    transactionHash: expect.any(H256),
                    // number
                    index: expect.anything(),
                    assetType: expect.any(H256),
                    // number
                    amount: expect.anything(),
                })
            }),
            // Buffer
            lockScript: expect.anything(),
            // Buffer
            unlockScript: expect.anything(),
        }]),
        outputs: expect.arrayContaining([{
            data: expect.objectContaining({
                lockScriptHash: expect.any(H256),
                // number
                parameters: expect.anything(),
                assetType: expect.any(H256),
                // number
                amount: expect.anything(),
            })
        }]),
        // number
        nonce: expect.anything(),
    });
});

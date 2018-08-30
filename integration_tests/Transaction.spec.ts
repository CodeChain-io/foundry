import { SDK } from "..";
import { MemoryKeyStore } from "../lib/key/MemoryKeyStore";
import { mintAsset, sendTransactions } from "./helper";

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
const sdk = new SDK({ server: SERVER_URL });
const { H256, ChangeShardState } = SDK.Core.classes;

test("AssetMintTransaction fromJSON", async () => {
    const metadata = "";
    const lockScriptHash = new H256(
        "0000000000000000000000000000000000000000000000000000000000000000"
    );
    const amount = 100;
    const registrar = null;
    const { parcelHash } = await mintAsset({
        metadata,
        lockScriptHash,
        amount,
        registrar
    });
    const parcel = await sdk.rpc.chain.getParcel(parcelHash);

    if (!(parcel.unsigned.action instanceof ChangeShardState)) {
        throw Error("Invalid action");
    }

    expect(parcel.unsigned.action.transactions[0]).toMatchObject({
        type: expect.stringMatching("assetMint"),
        metadata: expect.anything(),
        output: {
            lockScriptHash: expect.any(H256),
            // FIXME: Buffer[]
            parameters: expect.anything(),
            // FIXME: Change it to U256
            amount: expect.anything()
        },
        // FIXME: null or H160
        registrar: null,
        nonce: expect.anything()
    });
});

test("AssetTransferTransaction fromJSON", async () => {
    const keyStore = new MemoryKeyStore();
    const p2pkh = await sdk.key.createP2PKH({ keyStore });
    const addressA = await p2pkh.createAddress();
    const addressB = await p2pkh.createAddress();
    const mintTx = sdk.core
        .createAssetScheme({
            shardId: 0,
            worldId: 0,
            metadata: "metadata of non-permissioned asset",
            amount: 100,
            registrar: null
        })
        .createMintTransaction({ recipient: addressA });
    await sendTransactions({ transactions: [mintTx] });
    const firstAsset = await sdk.rpc.chain.getAsset(mintTx.hash(), 0);

    const transferTx = await firstAsset.createTransferTransaction({
        recipients: [
            {
                address: addressB,
                amount: 100
            }
        ]
    });
    transferTx.signInput(0, { signer: p2pkh });
    const { parcelHash } = await sendTransactions({
        transactions: [transferTx]
    });
    const parcel = await sdk.rpc.chain.getParcel(parcelHash);
    // FIXME: Remove anythings when *Data fields are flattened
    const expectedInput = expect.objectContaining({
        prevOut: expect.objectContaining({
            transactionHash: expect.any(H256),
            index: expect.anything(),
            assetType: expect.any(H256),
            amount: expect.anything()
        }),
        lockScript: expect.anything(),
        unlockScript: expect.anything()
    });
    const expectedOutput = expect.objectContaining({
        lockScriptHash: expect.anything(),
        parameters: [],
        assetType: expect.anything(),
        amount: expect.anything()
    });

    if (!(parcel.unsigned.action instanceof ChangeShardState)) {
        throw Error("Invalid action");
    }

    expect(parcel.unsigned.action.transactions[0]).toMatchObject({
        type: expect.stringMatching("assetTransfer"),
        burns: [],
        inputs: expect.arrayContaining([expectedInput]),
        outputs: expect.anything(),
        networkId: expect.anything(),
        nonce: expect.anything()
    });
});

import { SDK, Parcel, U256, H256, H160, H512, PaymentTransaction } from "../";
import { privateKeyToAddress } from "../src/utils";
import { payment, mintAsset, setRegularKey } from "./helper";

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
const sdk = new SDK(SERVER_URL);

test("PaymentTransaction fromJSON", async () => {
    const hash = await payment();
    const parcel = await sdk.getParcel(hash);
    expect(parcel.unsigned.transactions[0]).toMatchObject({
        type: expect.stringMatching("payment"),
        data: expect.objectContaining({
            nonce: expect.any(U256),
            sender: expect.any(H160),
            receiver: expect.any(H160),
            value: expect.any(U256),
        }),
    });
});

test("SetRegularKeyTransaction fromJSON", async () => {
    const parcelHash = await setRegularKey();
    const parcel = await sdk.getParcel(parcelHash);
    expect(parcel.unsigned.transactions[0]).toMatchObject({
        type: expect.stringMatching("setRegularKey"),
        data: expect.objectContaining({
            address: expect.any(H160),
            nonce: expect.any(U256),
            key: expect.any(H512),
        })
    });
});

test("AssetMintTransaction fromJSON", async () => {
    const metadata = "";
    const lockScriptHash = new H256("0000000000000000000000000000000000000000000000000000000000000000");
    const amount = 100;
    const parameters = [];
    const registrar = null;
    const { parcelHash } = await mintAsset({ metadata, lockScriptHash, amount, parameters, registrar });
    const parcel = await sdk.getParcel(parcelHash);
    expect(parcel.unsigned.transactions[0]).toMatchObject({
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
        })
    });
});

test.skip("AssetTransferTransaction fromJSON", async () => {});

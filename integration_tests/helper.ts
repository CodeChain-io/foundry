import { SDK, AssetMintTransaction, H256, Parcel, H160, U256 } from "../";
import { privateKeyToAddress } from "../src/utils";

const SERVER_URL = "http://localhost:8080";
const sdk = new SDK(SERVER_URL);

const secret = new H256("ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd");
const address = new H160(privateKeyToAddress(secret.value));

export const mintAsset = async ({ metadata, amount, lockScriptHash, parameters, registrar }) => {
    const assetMintTransaction = new AssetMintTransaction({
        metadata,
        lockScriptHash,
        parameters,
        amount,
        registrar,
    });
    const nonce = await sdk.getNonce(address);
    const fee = new U256(10);
    const networkId = 17;

    const parcel = new Parcel(nonce, fee, assetMintTransaction, networkId);
    await sdk.sendSignedParcel(parcel.sign(secret));

    return assetMintTransaction;
};
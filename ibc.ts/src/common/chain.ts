import { Datagram } from "./datagram/index";
import { SDK } from "codechain-sdk";
import { H256, PlatformAddress } from "codechain-primitives";
import { IBC } from "./foundry/transaction";
import { delay } from "./util";
import Debug from "debug";

const debug = Debug("common:tx");

export interface ChainConfig {
    /**
     * Example: "http://localhost:8080"
     */
    server: string;
    networkId: string;
    faucetAddress: PlatformAddress;
}

export class Chain {
    private readonly sdk: SDK;
    private readonly faucetAddress: PlatformAddress;

    public constructor(config: ChainConfig) {
        this.sdk = new SDK({
            server: config.server,
            networkId: config.networkId
        });
        this.faucetAddress = config.faucetAddress;
    }

    public async submitDatagram(datagram: Datagram): Promise<void> {
        const ibcAction = new IBC(this.sdk.networkId, datagram.rlpBytes());

        const seq = await this.sdk.rpc.chain.getSeq(this.faucetAddress);
        const signedTx = await this.sdk.key.signTransaction(ibcAction, {
            account: this.faucetAddress,
            fee: 100,
            seq
        });

        const txHash = await this.sdk.rpc.chain.sendSignedTransaction(signedTx);
        waitForTx(this.sdk, txHash);
    }
}

async function waitForTx(sdk: SDK, txHash: H256) {
    const timeout = delay(10 * 1000).then(() => {
        throw new Error("Timeout");
    });
    const wait = (async () => {
        while (true) {
            debug(`wait tx: ${txHash.toString()}`);
            if (sdk.rpc.chain.containsTransaction(txHash)) {
                return;
            }
            await delay(500);
        }
    })();
    return Promise.race([timeout, wait]);
}

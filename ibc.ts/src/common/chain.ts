import { Datagram } from "./datagram/index";
import { SDK } from "codechain-sdk";
import { H256, PlatformAddress } from "codechain-primitives";
import { IBC } from "./foundry/transaction";
import { delay } from "./util";
import Debug from "debug";
import { ClientState } from "./foundry/types";
import { IBCHeader, IBCQueryResult, ConnectionEnd, ChannelEnd } from "./types";

const debug = Debug("common:tx");

export interface CounterpartyIdentifiers {
    /**
     * Identifier for counter party chain's light client saved in this chain
     */
    client: string;
    /**
     * Identifier for connection with counterparty chain
     */
    connection: string;
    /**
     * Identifier for channel with counterparty chain
     */
    channel: string;
}

export interface ChainConfig {
    /**
     * Example: "http://localhost:8080"
     */
    server: string;
    networkId: string;
    faucetAddress: PlatformAddress;
    counterpartyIdentifiers: CounterpartyIdentifiers;
    keystorePath: string;
}

export class Chain {
    private readonly sdk: SDK;
    private readonly faucetAddress: PlatformAddress;
    public readonly counterpartyIdentifiers: CounterpartyIdentifiers;

    public constructor(config: ChainConfig) {
        this.sdk = new SDK({
            server: config.server,
            networkId: config.networkId,
            keyStoreType: {
                type: "local",
                path: config.keystorePath
            }
        });
        this.faucetAddress = config.faucetAddress;
        this.counterpartyIdentifiers = config.counterpartyIdentifiers;
    }

    public async submitDatagram(datagram: Datagram): Promise<void> {
        await this.submitDatagrams([datagram]);
    }

    public async submitDatagrams(datagrams: Datagram[]): Promise<void> {
        const txHashes = [];
        const seq = await this.sdk.rpc.chain.getSeq(this.faucetAddress);

        for (let i = 0; i < datagrams.length; i += 1) {
            const datagram = datagrams[i];
            const ibcAction = new IBC(this.sdk.networkId, datagram.rlpBytes());

            const signedTx = await this.sdk.key.signTransaction(ibcAction, {
                account: this.faucetAddress,
                fee: 100,
                seq: seq + i
            });

            debug(`Send tx with seq ${seq + i}`);
            const txHash = await this.sdk.rpc.chain.sendSignedTransaction(
                signedTx
            );
            txHashes.push(txHash);
        }

        for (const txHash of txHashes) {
            debug(`Wait for tx ${txHash}`);
            await waitForTx(this.sdk, txHash);
        }
    }

    public async latestHeight(): Promise<number> {
        return await this.sdk.rpc.chain.getBestBlockNumber();
    }

    public async queryClient(
        blockNumber?: number
    ): Promise<IBCQueryResult<ClientState> | null> {
        return this.sdk.rpc.sendRpcRequest("ibc_query_client_state", [
            this.counterpartyIdentifiers.client,
            blockNumber
        ]);
    }

    public async queryIBCHeader(
        blockNumber: number
    ): Promise<IBCHeader | null> {
        return this.sdk.rpc.sendRpcRequest("ibc_compose_header", [blockNumber]);
    }

    public async queryChainHeader(blockNumber: number): Promise<string | null> {
        return this.sdk.rpc.sendRpcRequest("chain_getRawHeaderByNumber", [
            blockNumber
        ]);
    }

    public async queryConnection(
        blockNumber?: number
    ): Promise<IBCQueryResult<ConnectionEnd> | null> {
        return this.sdk.rpc.sendRpcRequest("ibc_query_connection", [
            this.counterpartyIdentifiers.connection,
            blockNumber
        ]);
    }

    public async queryClientConnections(
        blockNumber?: number
    ): Promise<IBCQueryResult<string[]> | null> {
        return this.sdk.rpc.sendRpcRequest("ibc_query_client_connections", [
            this.counterpartyIdentifiers.client,
            blockNumber
        ]);
    }

    public async queryChannel(
        blockNumber?: number
    ): Promise<IBCQueryResult<ChannelEnd> | null> {
        return this.sdk.rpc.sendRpcRequest("ibc_query_channel_end", [
            "DEFAULT_PORT",
            this.counterpartyIdentifiers.channel,
            blockNumber
        ]);
    }
}

async function waitForTx(sdk: SDK, txHash: H256) {
    const timeout = delay(10 * 1000).then(() => {
        throw new Error("Timeout");
    });
    const wait = (async () => {
        while (true) {
            debug(`wait tx: ${txHash.toString()}`);
            if (await sdk.rpc.chain.containsTransaction(txHash)) {
                return;
            }
            await delay(500);
        }
    })();
    return Promise.race([timeout, wait]);
}
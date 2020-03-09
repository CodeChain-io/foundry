import Debug from "debug";
import { Chain } from "../common/chain";
import { Datagram } from "../common/datagram/index";
import { delay } from "../common/util";
import { getConfig } from "../common/config";
import { PlatformAddress } from "codechain-primitives/lib";
import { UpdateClientDatagram } from "../common/datagram/updateClient";
import { strict as assert } from "assert";
import { ConnOpenTryDatagram } from "../common/datagram/connOpenTry";
import { ConnOpenAckDatagram } from "../common/datagram/connOpenAck";
import { ConnOpenConfirmDatagram } from "../common/datagram/connOpenConfirm";

require("dotenv").config();

const debug = Debug("relayer:main");

async function main() {
    const config = getConfig();
    const chainA = new Chain({
        server: config.chainA.rpcURL,
        networkId: config.chainA.networkId,
        faucetAddress: PlatformAddress.fromString(config.chainA.relayerAddress),
        counterpartyIdentifiers: {
            client: config.chainA.counterpartyClientId,
            connection: config.chainA.counterpartyConnectionId,
            channel: config.chainA.counterpartyChannelId
        },
        keystorePath: config.chainA.keystorePath
    });
    const chainB = new Chain({
        server: config.chainB.rpcURL,
        networkId: config.chainB.networkId,
        faucetAddress: PlatformAddress.fromString(config.chainB.relayerAddress),
        counterpartyIdentifiers: {
            client: config.chainB.counterpartyClientId,
            connection: config.chainB.counterpartyConnectionId,
            channel: config.chainB.counterpartyChannelId
        },
        keystorePath: config.chainB.keystorePath
    });

    while (true) {
        debug("Run relay");
        await relay(chainA, chainB);
        await delay(1000);
    }
}

main().catch(console.error);

async function relay(chainA: Chain, chainB: Chain) {
    await relayFromTo({ chain: chainA, counterpartyChain: chainB });
    await relayFromTo({ chain: chainB, counterpartyChain: chainA });
}

async function relayFromTo({
    chain,
    counterpartyChain
}: {
    chain: Chain;
    counterpartyChain: Chain;
}) {
    const { localDatagrams, counterpartyDatagrams } = await pendingDatagrams({
        chain,
        counterpartyChain
    });

    await chain.submitDatagrams(localDatagrams);

    await counterpartyChain.submitDatagrams(counterpartyDatagrams);
}

async function pendingDatagrams({
    chain,
    counterpartyChain
}: {
    chain: Chain;
    counterpartyChain: Chain;
}): Promise<{ localDatagrams: Datagram[]; counterpartyDatagrams: Datagram[] }> {
    let height = await chain.latestHeight();
    let counterpartyChainHeight = await counterpartyChain.latestHeight();
    let localDatagrams: Datagram[] = [];
    let counterpartyDatagrams: Datagram[] = [];

    localDatagrams = localDatagrams.concat(
        await updateLightClient({
            chain,
            counterpartyChain,
            height,
            counterpartyChainHeight
        })
    );

    counterpartyDatagrams = counterpartyDatagrams.concat(
        await updateLightClient({
            chain: counterpartyChain,
            counterpartyChain: chain,
            height: counterpartyChainHeight,
            counterpartyChainHeight: height
        })
    );

    // FIXME: We can't update light client upto the best block.
    height = height - 1;
    counterpartyChainHeight = counterpartyChainHeight - 1;

    const {
        localDatagrams: localDatagramsForConnection,
        counterpartyDatagrams: counterpartyDatagramsForConnection
    } = await buildConnection({
        chain,
        counterpartyChain,
        height,
        counterpartyChainHeight
    });

    localDatagrams = localDatagrams.concat(localDatagramsForConnection);
    counterpartyDatagrams = counterpartyDatagrams.concat(
        counterpartyDatagramsForConnection
    );

    return { localDatagrams, counterpartyDatagrams };
}

async function updateLightClient({
    chain,
    counterpartyChain,
    height,
    counterpartyChainHeight
}: {
    chain: Chain;
    counterpartyChain: Chain;
    height: number;
    counterpartyChainHeight: number;
}): Promise<Datagram[]> {
    const datagrams = [];
    const clientState = await chain.queryClient(height);

    if (clientState!.data == null) {
        throw new Error(
            `No client state found. Please create a light client with identifier: ${chain.counterpartyIdentifiers.client}`
        );
    }
    let currentBlockNumber = clientState!.data!.number;
    // FIXME: We can't get the best block's IBC header
    while (currentBlockNumber < counterpartyChainHeight - 1) {
        const header = (await counterpartyChain.queryIBCHeader(
            currentBlockNumber + 1
        ))!;
        assert.notEqual(header, null, "Composed header should not be null");
        datagrams.push(
            new UpdateClientDatagram({
                id: chain.counterpartyIdentifiers.client,
                header: Buffer.from(header, "hex")
            })
        );
        currentBlockNumber += 1;
    }

    return datagrams;
}

async function buildConnection({
    chain,
    counterpartyChain,
    height,
    counterpartyChainHeight
}: {
    chain: Chain;
    counterpartyChain: Chain;
    height: number;
    counterpartyChainHeight: number;
}) {
    const localDatagrams: Datagram[] = [];
    const counterpartyDatagrams = [];
    const connectionIdentifiers = await chain.queryClientConnections(height);

    assert.notEqual(connectionIdentifiers, null, "Client should be exist");
    for (const connectionIdentifier of connectionIdentifiers!.data || []) {
        const client = await chain.queryClient(height);
        const counterpartyClient = await counterpartyChain.queryClient(
            counterpartyChainHeight
        );

        assert.strictEqual(
            connectionIdentifier,
            chain.counterpartyIdentifiers.connection,
            "PoC supports only one connection"
        );
        const connectionEnd = await chain.queryConnection(height);
        const counterpartyConnectionEnd = await counterpartyChain.queryConnection(
            counterpartyChainHeight
        );
        assert.notEqual(
            connectionEnd!.data,
            null,
            "Connection exists because we acquired the identifier from RPC"
        );
        if (
            connectionEnd!.data!.state === "INIT" &&
            counterpartyConnectionEnd!.data == null
        ) {
            counterpartyDatagrams.push(
                new ConnOpenTryDatagram({
                    desiredIdentifier:
                        counterpartyChain.counterpartyIdentifiers.connection,
                    counterpartyConnectionIdentifier:
                        chain.counterpartyIdentifiers.connection,
                    counterpartyClientIdentifier:
                        chain.counterpartyIdentifiers.client,
                    clientIdentifier:
                        counterpartyChain.counterpartyIdentifiers.client,
                    proofInit: Buffer.from(connectionEnd!.proof, "hex"),
                    proofConsensus: Buffer.alloc(0),
                    proofHeight: height,
                    consensusHeight: client!.data!.number,
                    counterpartyPrefix: ""
                })
            );
        } else if (
            connectionEnd!.data!.state === "INIT" &&
            counterpartyConnectionEnd!.data!.state === "TRYOPEN"
        ) {
            localDatagrams.push(
                new ConnOpenAckDatagram({
                    identifier: chain.counterpartyIdentifiers.connection,
                    proofTry: Buffer.from(
                        counterpartyConnectionEnd!.proof,
                        "hex"
                    ),
                    proofConsensus: Buffer.alloc(0),
                    proofHeight: counterpartyChainHeight,
                    consensusHeight: counterpartyClient!.data!.number
                })
            );
        } else if (
            connectionEnd!.data!.state === "OPEN" &&
            counterpartyConnectionEnd!.data!.state === "TRYOPEN"
        ) {
            counterpartyDatagrams.push(
                new ConnOpenConfirmDatagram({
                    identifier:
                        counterpartyChain.counterpartyIdentifiers.connection,
                    proofAck: Buffer.from(connectionEnd!.proof, "hex"),
                    proofHeight: height
                })
            );
        }
    }

    return { localDatagrams, counterpartyDatagrams };
}

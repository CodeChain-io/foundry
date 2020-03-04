import Debug from "debug";
import { Chain } from "../common/chain";
import { Datagram } from "../common/datagram/index";
import { delay } from "../common/util";
import { getConfig } from "../common/config";
import { PlatformAddress } from "codechain-primitives/lib";
import { UpdateClientDatagram } from "../common/datagram/updateClient";
import { strict as assert } from "assert";

require("dotenv").config();

const debug = Debug("relayer:main");

async function main() {
    const config = getConfig();
    const chainA = new Chain({
        server: config.chainA.rpcURL,
        networkId: config.chainA.networkId,
        faucetAddress: PlatformAddress.fromString(config.chainA.faucetAddress),
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
        faucetAddress: PlatformAddress.fromString(config.chainB.faucetAddress),
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
    const height = await chain.latestHeight();
    const counterpartyChainHeight = await chain.latestHeight();
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
    while (currentBlockNumber < counterpartyChainHeight) {
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

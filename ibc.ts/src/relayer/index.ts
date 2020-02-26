import Debug from "debug";
import { Chain } from "../common/chain";
import { Datagram } from "../common/datagram/index";
import { delay } from "../common/util";

const debug = Debug("relayer:main");

async function main() {
    const chainA = new Chain();
    const chainB = new Chain();

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

    for (const localDiagram of localDatagrams) {
        await chain.submitDatagram(localDiagram);
    }

    for (const counterpartyDatagram of counterpartyDatagrams) {
        await counterpartyChain.submitDatagram(counterpartyDatagram);
    }
}

async function pendingDatagrams(
    args: any
): Promise<{ localDatagrams: Datagram[]; counterpartyDatagrams: Datagram[] }> {
    return { localDatagrams: [], counterpartyDatagrams: [] };
}

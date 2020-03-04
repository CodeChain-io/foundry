const RLP = require("rlp");

export class ConnOpenTryDatagram {
    private desiredIdentifier: string;
    private counterpartyConnectionIdentifier: string;
    private counterpartyPrefix: string;
    private counterpartyClientIdentifier: string;
    private clientIdentifier: string;
    private proofInit: Buffer;
    private proofConsensus: Buffer;
    private proofHeight: number;
    private consensusHeight: number;

    public constructor({
        desiredIdentifier,
        counterpartyConnectionIdentifier,
        counterpartyPrefix,
        counterpartyClientIdentifier,
        clientIdentifier,
        proofInit,
        proofConsensus,
        proofHeight,
        consensusHeight
    }: {
        desiredIdentifier: string;
        counterpartyConnectionIdentifier: string;
        counterpartyPrefix: string;
        counterpartyClientIdentifier: string;
        clientIdentifier: string;
        proofInit: Buffer;
        proofConsensus: Buffer;
        proofHeight: number;
        consensusHeight: number;
    }) {
        this.desiredIdentifier = desiredIdentifier;
        this.counterpartyConnectionIdentifier = counterpartyConnectionIdentifier;
        this.counterpartyPrefix = counterpartyPrefix;
        this.counterpartyClientIdentifier = counterpartyClientIdentifier;
        this.clientIdentifier = clientIdentifier;
        this.proofInit = proofInit;
        this.proofConsensus = proofConsensus;
        this.proofHeight = proofHeight;
        this.consensusHeight = consensusHeight;
    }

    public rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    public toEncodeObject(): any[] {
        return [
            4,
            this.desiredIdentifier,
            this.counterpartyConnectionIdentifier,
            this.counterpartyPrefix,
            this.counterpartyClientIdentifier,
            this.clientIdentifier,
            this.proofInit,
            this.proofConsensus,
            this.proofHeight,
            this.consensusHeight
        ];
    }
}

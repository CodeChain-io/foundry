const RLP = require("rlp");

export class ConnOpenInitDatagram {
    private id: string;
    private desiredCounterpartyConnectionIdentifier: string;
    private counterpartyPrefix: string;
    private clientIdentifier: string;
    private counterpartyClientIdentifier: string;

    public constructor({
        id,
        desiredCounterpartyConnectionIdentifier,
        counterpartyPrefix,
        clientIdentifier,
        counterpartyClientIdentifier
    }: {
        id: string;
        desiredCounterpartyConnectionIdentifier: string;
        counterpartyPrefix: string;
        clientIdentifier: string;
        counterpartyClientIdentifier: string;
    }) {
        this.id = id;
        this.desiredCounterpartyConnectionIdentifier = desiredCounterpartyConnectionIdentifier;
        this.counterpartyPrefix = counterpartyPrefix;
        this.clientIdentifier = clientIdentifier;
        this.counterpartyClientIdentifier = counterpartyClientIdentifier;
    }

    public rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    public toEncodeObject(): any[] {
        return [
            3,
            this.id,
            this.desiredCounterpartyConnectionIdentifier,
            this.counterpartyPrefix,
            this.clientIdentifier,
            this.counterpartyClientIdentifier
        ];
    }
}

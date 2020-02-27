const RLP = require("rlp");

export class CreateClientDatagram {
    private id: string;
    private kind: number;
    private consensusState: Buffer;
    private data: Buffer;

    public constructor({
        id,
        kind,
        consensusState,
        data
    }: {
        id: string;
        kind: number;
        consensusState: Buffer;
        data: Buffer;
    }) {
        this.id = id;
        this.kind = kind;
        this.consensusState = consensusState;
        this.data = data;
    }

    public rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    public toEncodeObject(): any[] {
        return [1, this.id, this.kind, this.consensusState, this.data];
    }
}

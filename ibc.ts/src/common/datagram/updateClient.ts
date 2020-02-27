const RLP = require("rlp");

export class UpdateClientDatagram {
    private id: string;
    private header: Buffer;

    public constructor({ id, header }: { id: string; header: Buffer }) {
        this.id = id;
        this.header = header;
    }

    public rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    public toEncodeObject(): any[] {
        return [2, this.id, this.header];
    }
}

export interface Datagram {
    rlpBytes(): Buffer;
    toEncodeObject(): any[];
}

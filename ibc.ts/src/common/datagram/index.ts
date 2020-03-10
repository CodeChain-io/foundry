export interface Datagram {
    rlpBytes(): Buffer;
    toEncodeObject(): any[];
}

export const ChannelOrdered = 0;
export const ChannelUnordered = 1;

export class NoopTransaction {
    private type = "noop";

    toEncodeObject() {
        return "";
    }
}

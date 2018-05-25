export class NoopTransaction {
    private type = "noop";

    toEncodeObject() {
        return "";
    }

    toJSON() {
        return "noop";
    }
}

export class CreateShard {
    public toEncodeObject(): any[] {
        return [4];
    }

    public toJSON() {
        return {
            action: "createShard"
        };
    }
}

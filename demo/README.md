# Foundry Demo

## How to Run

1. Build Foundry
2. Place the binary in this directory
3. Run

```
RUST_LOG=warn ./foundry  --app-desc-path app-desc.yml --config config0.toml --db-path ./db0

RUST_LOG=warn ./foundry  --app-desc-path app-desc.yml --config config1.toml --db-path ./db1

RUST_LOG=warn ./foundry  --app-desc-path app-desc.yml --config config2.toml --db-path ./db2

RUST_LOG=warn ./foundry  --app-desc-path app-desc.yml --config config3.toml --db-path ./db3
```

for each node.
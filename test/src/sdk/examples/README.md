To execute the example scripts without modifying `require("codechain-sdk")`, run the integration tests.

# How to test all examples

```
yarn test-int --testRegex examples.spec.ts
```

# How to test specific example
```
yarn test-int --testRegex examples.spec.ts -t mint-and-transfer
```


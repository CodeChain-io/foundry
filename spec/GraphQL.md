# GraphQL

The only interface to access to the running Foundry node is [GraphQL](https://graphql.org/).
There are `N+1` GraphQL endpoints opened in a single Foundry node, where `N` is the modules, and `1` is the consensus engine.
Each can be found in either `localhost:${port}/${module_name}/graphql` or `localhost:${port}/engine/graphql`.

Queries for modules don't specify the block number by themselves.
You should provide it in the HTTP header if you want to specify the block number.
Use `"number": _` for that. (This is because of federation - tentative)
Note that the latest block will be chosen if you don't specify any.

You can also use [GraphiQL](https://github.com/graphql/graphiql), an offical IDE for GraphQL.
To access to it, use `/__graphql` instead of `/graphql`.
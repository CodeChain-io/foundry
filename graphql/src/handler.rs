/// This will be used in both tests and
/// various GraphQL resolver for thehost level(chain, mempool, net...)
/// which will be implemented in this crate as well.
pub fn handle_gql_query<T: async_graphql::ObjectType + Send + Sync + 'static>(
    runtime: &tokio::runtime::Handle,
    root: T,
    query: &str,
    variables: &str,
) -> String {
    let variables = if let Ok(s) = (|| -> Result<_, ()> {
        Ok(async_graphql::context::Variables::from_json(serde_json::from_str(variables).map_err(|_| ())?))
    })() {
        s
    } else {
        return "Failed to parse JSON".to_owned()
    };

    let schema = async_graphql::Schema::new(root, async_graphql::EmptyMutation, async_graphql::EmptySubscription);
    let request = async_graphql::Request::new(query).variables(variables);
    let res = schema.execute(request);
    serde_json::to_string(&runtime.block_on(res)).unwrap()
}

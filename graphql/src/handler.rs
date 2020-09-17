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
        let json_variables = async_graphql::serde_json::from_str(variables).map_err(|_| ())?;
        let variables = async_graphql::Variables::parse_from_json(json_variables);
        Ok(variables)
    })() {
        s
    } else {
        return "Failed to parse JSON".to_owned()
    };

    let schema = async_graphql::Schema::new(root, async_graphql::EmptyMutation, async_graphql::EmptySubscription);
    let query = async_graphql::QueryBuilder::new(query).variables(variables);
    let res = query.execute(&schema);
    async_graphql::serde_json::to_string(&async_graphql::http::GQLResponse(runtime.block_on(res))).unwrap()
}

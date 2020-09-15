use coordinator::module::{HandleGraphQlRequest, SessionId};
use remote_trait_object::Service;
use std::collections::HashMap;

#[derive(Clone)]
struct Account {
    balance: u32,
}

#[derive(Clone)]
struct GraphQlRoot {
    accounts: HashMap<String, Account>,
}

#[async_graphql::Object]
impl GraphQlRoot {
    async fn account(&self, name: String) -> Option<Account> {
        self.accounts.get(&name).map(|x| x.clone())
    }
}

#[async_graphql::Object]
impl Account {
    async fn balance(&self) -> u32 {
        self.balance
    }
}

struct GraphQlRequestHandler {
    root: GraphQlRoot,

    /// A runtime to process the asynchronous result of the query
    tokio_runtime: Option<tokio::runtime::Runtime>,
}

impl GraphQlRequestHandler {
    fn new() -> Self {
        let accounts = vec![
            ("John".to_owned(), Account {
                balance: 10,
            }),
            ("Matthew".to_owned(), Account {
                balance: 20,
            }),
            ("James".to_owned(), Account {
                balance: 30,
            }),
        ]
        .into_iter()
        .collect();

        Self {
            root: GraphQlRoot {
                accounts,
            },
            tokio_runtime: Some(tokio::runtime::Runtime::new().unwrap()),
        }
    }
}

impl Drop for GraphQlRequestHandler {
    fn drop(&mut self) {
        self.tokio_runtime.take().unwrap().shutdown_background();
    }
}

impl Service for GraphQlRequestHandler {}

impl HandleGraphQlRequest for GraphQlRequestHandler {
    fn execute(&self, session: SessionId, query: &str, variables: &str) -> String {
        assert_eq!(session, 123);
        // We can't use tokio runtime inside another tokio. We spawn a new thread to avoid such restriciton.
        crossbeam::thread::scope(|s| {
            let j = s.spawn(|_| {
                fgql::handle_gql_query(
                    self.tokio_runtime.as_ref().unwrap().handle(),
                    self.root.clone(),
                    query,
                    variables,
                )
            });
            j.join().unwrap()
        })
        .unwrap()
    }
}

pub fn create_handler() -> Box<dyn HandleGraphQlRequest> {
    Box::new(GraphQlRequestHandler::new())
}

#[test]
fn query_directly() {
    let handler = create_handler();
    let result = handler.execute(123, r#"{account(name: "John"){ balance }}"#, "{}");
    assert_eq!(r#"{"data":{"account":{"balance":10}}}"#, result);

    let result =
        handler.execute(123, r#"query Test($name: String){account(name: $name){balance}}"#, r#"{"name": "John"}"#);
    assert_eq!(r#"{"data":{"account":{"balance":10}}}"#, result);
}

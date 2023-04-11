use crate::config::Config;

pub fn run_query(cfg: Config, query: String) -> String {
    let parsed: QueryNode = parse_query(&query);
    eval_query(parsed)
}

enum QueryNode {
    Latest,
}

fn parse_query(query: &str) -> QueryNode {
    QueryNode::Latest
}

fn eval_query(query: QueryNode) -> String {
    "output".to_string()
}
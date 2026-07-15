//! PromQL label injection (TASK-OBS-002 §1 #5, §3) via promql-parser@0.4.
//!
//! AST-based, never string concatenation (DEC-146): parse the query, push a `key="value"` matcher
//! onto every vector selector in the tree, and reserialise via the AST's Display. Because the parser
//! rejects malformed input, a bad query becomes a clean `ParseFailed` (400), never a 500.

use crate::error::{Backend, ProxyError};
use promql_parser::label::{MatchOp, Matcher};
use promql_parser::parser::{self, Expr, VectorSelector};

/// Inject `key="value"` into every vector selector and reserialise the query.
pub fn add_label(query: &str, key: &str, value: &str) -> Result<String, ProxyError> {
    let mut ast = parse(query)?;
    for_each_selector(&mut ast, &mut |vs| {
        if !vs.matchers.matchers.iter().any(|m| m.name == key) {
            vs.matchers
                .matchers
                .push(Matcher::new(MatchOp::Equal, key, value));
        }
    });
    Ok(ast.to_string())
}

/// True if any vector selector already carries a matcher named `key` (a bypass attempt - the proxy
/// checks this before injecting and refuses the query if so).
pub fn has_label(query: &str, key: &str) -> Result<bool, ProxyError> {
    let mut ast = parse(query)?;
    let mut found = false;
    for_each_selector(&mut ast, &mut |vs| {
        if vs.matchers.matchers.iter().any(|m| m.name == key) {
            found = true;
        }
    });
    Ok(found)
}

fn parse(query: &str) -> Result<Expr, ProxyError> {
    parser::parse(query).map_err(|reason| ProxyError::ParseFailed {
        backend: Backend::Prometheus,
        reason,
    })
}

/// Depth-first walk applying `f` to every vector selector in the expression tree.
fn for_each_selector(expr: &mut Expr, f: &mut dyn FnMut(&mut VectorSelector)) {
    match expr {
        Expr::VectorSelector(vs) => f(vs),
        Expr::MatrixSelector(ms) => f(&mut ms.vs),
        Expr::Aggregate(a) => {
            for_each_selector(&mut a.expr, f);
            if let Some(p) = a.param.as_mut() {
                for_each_selector(p, f);
            }
        }
        Expr::Unary(u) => for_each_selector(&mut u.expr, f),
        Expr::Binary(b) => {
            for_each_selector(&mut b.lhs, f);
            for_each_selector(&mut b.rhs, f);
        }
        Expr::Paren(p) => for_each_selector(&mut p.expr, f),
        Expr::Subquery(s) => for_each_selector(&mut s.expr, f),
        Expr::Call(c) => {
            for arg in c.args.args.iter_mut() {
                for_each_selector(arg, f);
            }
        }
        Expr::NumberLiteral(_) | Expr::StringLiteral(_) | Expr::Extension(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn injects_into_simple_selector() {
        let r = add_label("rate(foo[5m])", "tenant_id", "T").unwrap();
        assert!(r.contains("tenant_id=\"T\""));
        assert!(r.contains("foo"));
    }

    #[test]
    fn adds_to_existing_label_set() {
        let r = add_label("foo{x=\"y\"}", "tenant_id", "T").unwrap();
        assert!(r.contains("tenant_id=\"T\""));
        assert!(r.contains("x=\"y\""));
    }

    #[test]
    fn injects_into_every_selector_in_complex_query() {
        let r = add_label("sum(rate(foo[5m])) / sum(rate(bar[5m]))", "tenant_id", "T").unwrap();
        assert_eq!(r.matches("tenant_id=\"T\"").count(), 2);
    }

    #[test]
    fn detects_user_supplied_tenant_id() {
        assert!(has_label("foo{tenant_id=\"other\"}", "tenant_id").unwrap());
        assert!(!has_label("foo{x=\"y\"}", "tenant_id").unwrap());
    }

    #[test]
    fn malformed_promql_errors() {
        let e = add_label("rate(foo[bad", "tenant_id", "T").expect_err("parse must fail");
        assert!(matches!(e, ProxyError::ParseFailed { .. }));
    }
}

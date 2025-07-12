mod full;
mod semantic_report;

use crate::exe;

// utility methods to construct values faster

fn bool(b: bool) -> exe::Value {
    exe::Value::Bool(b)
}

fn int(n: i64) -> exe::Value {
    exe::Value::Int(n)
}

fn float(f: f64) -> exe::Value {
    exe::Value::Float(f)
}

fn str(s: &str) -> exe::Value {
    exe::Value::String(s.to_string())
}

fn bun<const N: usize>(items: [exe::Value; N]) -> exe::Value {
    exe::Value::Bundle(Box::new(items))
}

fn unit() -> exe::Value {
    bun([])
}

fn func<const N: usize>(captured: [exe::Value; N]) -> exe::Value {
    bun([exe::Value::Func, bun(captured)])
}

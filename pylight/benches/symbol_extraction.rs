use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use pylight::PythonParser;
use std::path::Path;

const SIMPLE_FUNCTION: &str = r#"
def hello():
    return "world"

def add(a, b):
    return a + b
"#;

const COMPLEX_FILE: &str = r#"
import os
import sys
from typing import List, Dict, Optional

class BaseClass:
    def __init__(self):
        self.value = 0
    
    def method1(self):
        return self.value

class DerivedClass(BaseClass):
    def __init__(self):
        super().__init__()
        self.extra = []
    
    def method2(self, x: int) -> int:
        def inner_func():
            return x * 2
        return inner_func()
    
    @property
    def computed(self):
        return self.value * 2
    
    @staticmethod
    def static_method():
        return 42

def global_function(param1: str, param2: Optional[int] = None):
    """A global function with type hints."""
    
    def nested_function():
        return param1.upper()
    
    class LocalClass:
        def local_method(self):
            pass
    
    return nested_function()

@decorator
def decorated_function():
    pass

async def async_function():
    pass
"#;

fn bench_simple_parse(c: &mut Criterion) {
    let mut parser = PythonParser::new().unwrap();
    
    c.bench_function("parse_simple_functions", |b| {
        b.iter(|| {
            let symbols = parser.parse_file(
                Path::new("test.py"),
                black_box(SIMPLE_FUNCTION)
            ).unwrap();
            black_box(symbols);
        });
    });
}

fn bench_complex_parse(c: &mut Criterion) {
    let mut parser = PythonParser::new().unwrap();
    
    c.bench_function("parse_complex_file", |b| {
        b.iter(|| {
            let symbols = parser.parse_file(
                Path::new("test.py"),
                black_box(COMPLEX_FILE)
            ).unwrap();
            black_box(symbols);
        });
    });
}

fn bench_file_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_by_size");
    let mut parser = PythonParser::new().unwrap();
    
    // Generate files of different sizes
    let sizes = vec![100, 500, 1000, 5000];
    
    for size in sizes {
        let content = generate_python_file(size);
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &content,
            |b, content| {
                b.iter(|| {
                    let symbols = parser.parse_file(
                        Path::new("test.py"),
                        black_box(content)
                    ).unwrap();
                    black_box(symbols);
                });
            }
        );
    }
    group.finish();
}

fn generate_python_file(num_functions: usize) -> String {
    let mut content = String::new();
    
    for i in 0..num_functions {
        content.push_str(&format!(
            r#"
def function_{}(x, y):
    """Function number {}"""
    return x + y + {}

"#,
            i, i, i
        ));
        
        if i % 10 == 0 {
            content.push_str(&format!(
                r#"
class Class_{}:
    def method_{}(self):
        return {}
        
"#,
                i, i, i
            ));
        }
    }
    
    content
}

criterion_group!(benches, bench_simple_parse, bench_complex_parse, bench_file_sizes);
criterion_main!(benches);
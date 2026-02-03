#!/usr/bin/env python3
"""
Helper script to systematically fix Arc<> compilation errors.
Patterns to fix:
1. Value::Str(x) where x is String -> Value::Str(Arc::new(x))
2. Value::Array(x) where x is Vec<Value> -> Value::Array(Arc::new(x))
3. Value::Dict(x) where x is HashMap -> Value::Dict(Arc::new(x))
4. Iteration over Arc<Vec> and Arc<HashMap> -> .iter()
5. String clone from Arc<String> -> .as_ref().clone() or .to_string()
"""

import re
import sys

def fix_value_constructions(content):
    """Fix Value::Str, Array, Dict constructions to use Arc::new"""
    
    # Pattern 1: Value::Str(something) but not Value::Str(Arc::new(...))
    # This is tricky because we need to avoid double-wrapping
    # For now, let's handle simple cases
    
    # Fix Value::Str(field.to_string()) -> Value::Str(Arc::new(field.to_string()))
    content = re.sub(
        r'Value::Str\(([^A][^)]+\.to_string\(\))\)',
        r'Value::Str(Arc::new(\1))',
        content
    )
    
    # Fix Value::Str(format!(...)) -> Value::Str(Arc::new(format!(...)))
    content = re.sub(
        r'Value::Str\((format!\([^)]+\))\)',
        r'Value::Str(Arc::new(\1))',
        content
    )
    
    # Fix Value::Str(String::new()) -> Value::Str(Arc::new(String::new()))
    content = re.sub(
        r'Value::Str\(String::new\(\)\)',
        r'Value::Str(Arc::new(String::new()))',
        content
    )
    
    # Fix Value::Array(something) where something doesn't start with Arc
    # Look for Value::Array(rows), Value::Array(result), etc
    content = re.sub(
        r'Value::Array\(([a-z_][a-z_0-9]*)\)([^.]|$)',
        r'Value::Array(Arc::new(\1))\2',
        content
    )
    
    # Fix Value::Dict(something) where something doesn't start with Arc
    content = re.sub(
        r'Value::Dict\(([a-z_][a-z_0-9]*)\)([^.]|$)',
        r'Value::Dict(Arc::new(\1))\2',
        content
    )
    
    return content

def fix_arc_string_clones(content):
    """Fix s.clone() where s is Arc<String> to s.as_ref().clone()"""
    # This is context-sensitive and hard to automate safely
    # For now, just add a comment
    return content

def fix_iterations(content):
    """Fix iterations over Arc<Vec> and Arc<HashMap>"""
    # Replace: for item in arr { -> for item in arr.iter() {
    # This needs context to know if arr is Arc<Vec>
    return content

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: fix_arc_errors.py <file>")
        sys.exit(1)
    
    filename = sys.argv[1]
    with open(filename, 'r') as f:
        content = f.read()
    
    content = fix_value_constructions(content)
    
    with open(filename, 'w') as f:
        f.write(content)
    
    print(f"Fixed {filename}")

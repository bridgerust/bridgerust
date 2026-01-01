# Migration Guide: PyO3/napi-rs to BridgeRust

This guide helps you migrate existing PyO3 or napi-rs projects to BridgeRust, allowing you to support both Python and Node.js from a single codebase.

## Why Migrate?

- **Single Codebase**: Write once, deploy to both Python and Node.js
- **Unified API**: One macro system instead of maintaining separate bindings
- **Reduced Maintenance**: Less code duplication, easier updates
- **Future-Proof**: Easy to add more targets later

## Migration Overview

The migration process involves:

1. Adding BridgeRust dependencies
2. Replacing `#[pyfunction]`/`#[napi]` with `#[bridgerust::export]`
3. Updating project structure
4. Testing both targets

## Step-by-Step Migration

### Step 1: Add BridgeRust Dependencies

Update your `Cargo.toml`:

```toml
[dependencies]
bridgerust = "0.1"

[features]
default = []
python = ["dep:pyo3", "bridgerust/python"]
nodejs = ["dep:napi", "dep:napi-derive", "bridgerust/nodejs"]

[dependencies.pyo3]
version = "0.27"
optional = true
features = ["extension-module"]

[dependencies.napi]
version = "3"
optional = true

[dependencies.napi-derive]
version = "3"
optional = true
```

### Step 2: Replace Function Macros

#### Before (PyO3 only):

```rust
use pyo3::prelude::*;

#[pyfunction]
pub fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

#[pymodule]
fn my_module(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(greet, m)?)?;
    Ok(())
}
```

#### After (BridgeRust):

```rust
use bridgerust::export;

#[export]
pub fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

// Python module (still needed for module registration)
#[cfg(feature = "python")]
use bridgerust::pyo3::prelude::*;

#[cfg(feature = "python")]
#[pymodule]
fn my_module(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(greet, m)?)?;
    Ok(())
}
```

#### Before (napi-rs only):

```rust
use napi_derive::napi;

#[napi]
pub fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}
```

#### After (BridgeRust):

```rust
use bridgerust::export;

#[export]
pub fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}
```

### Step 3: Replace Struct Macros

#### Before (PyO3):

```rust
use pyo3::prelude::*;

#[pyclass]
pub struct Point {
    #[pyo3(get, set)]
    pub x: f64,
    #[pyo3(get, set)]
    pub y: f64,
}

#[pymethods]
impl Point {
    #[new]
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    fn distance(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}
```

#### After (BridgeRust):

```rust
use bridgerust::export;

#[export]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

// Python methods
#[cfg(feature = "python")]
use bridgerust::pyo3::prelude::*;

#[cfg(feature = "python")]
#[pymethods]
impl Point {
    #[new]
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    fn distance(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

// Node.js methods
#[cfg(feature = "nodejs")]
use bridgerust::napi_derive::napi;

#[cfg(feature = "nodejs")]
#[napi]
impl Point {
    #[napi(constructor)]
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    #[napi]
    pub fn distance(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}
```

### Step 4: Error Handling Migration

#### Before (PyO3):

```rust
use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;

#[pyfunction]
fn divide(a: f64, b: f64) -> PyResult<f64> {
    if b == 0.0 {
        Err(PyValueError::new_err("Division by zero"))
    } else {
        Ok(a / b)
    }
}
```

#### After (BridgeRust):

```rust
use bridgerust::{export, error};
use std::fmt::{Display, Formatter};

#[error]
#[derive(Debug)]
pub enum MathError {
    DivisionByZero,
}

impl Display for MathError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MathError::DivisionByZero => write!(f, "Division by zero"),
        }
    }
}

#[export]
pub fn divide(a: f64, b: f64) -> Result<f64, MathError> {
    if b == 0.0 {
        Err(MathError::DivisionByZero)
    } else {
        Ok(a / b)
    }
}

// In Python binding code:
#[cfg(feature = "python")]
#[pyfunction]
fn divide_py(a: f64, b: f64) -> PyResult<f64> {
    divide(a, b).map_err(to_py_err)
}
```

### Step 5: Update Project Structure

If you only had Python bindings, add Node.js structure:

```bash
# Create Node.js directory
mkdir nodejs

# Create package.json
cat > nodejs/package.json << EOF
{
  "name": "@your-org/your-package",
  "version": "0.1.0",
  "main": "index.js",
  "types": "index.d.ts",
  "napi": {
    "name": "your_package",
    "triples": {
      "defaults": true
    }
  }
}
EOF
```

If you only had Node.js bindings, add Python structure:

```bash
# Create Python directory
mkdir python

# Create pyproject.toml
cat > python/pyproject.toml << EOF
[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[project]
name = "your-package"
version = "0.1.0"
requires-python = ">=3.8"

[tool.maturin]
features = ["python"]
module-name = "your_package"
EOF
```

### Step 6: Update Build Scripts

#### Before (PyO3 only):

```bash
# Build Python
maturin build --release
```

#### After (BridgeRust):

```bash
# Build both
bridge build --all --release

# Or build individually
bridge build --target python --release
bridge build --target nodejs --release
```

## Common Migration Patterns

### Pattern 1: Conditional Module Registration

You'll need separate module registration for Python and Node.js:

```rust
// Python module
#[cfg(feature = "python")]
#[pymodule]
fn my_module(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(greet, m)?)?;
    m.add_class::<Point>()?;
    Ok(())
}

// Node.js module (napi-rs handles this automatically via #[napi] on functions)
// No explicit registration needed
```

### Pattern 2: Type Differences

Some types work differently between Python and Node.js:

```rust
// Option<T> - works the same
#[export]
pub fn maybe_value() -> Option<i32> {
    Some(42)
}

// Vec<T> - works the same
#[export]
pub fn process_list(items: Vec<i32>) -> Vec<i32> {
    items.into_iter().map(|x| x * 2).collect()
}

// Result<T, E> - needs error conversion
#[export]
pub fn might_fail() -> Result<i32, MyError> {
    // ...
}
```

### Pattern 3: Method Naming

Python and Node.js have different naming conventions:

```rust
// Python: snake_case
#[cfg(feature = "python")]
#[pymethods]
impl MyStruct {
    fn get_value(&self) -> i32 { ... }
}

// Node.js: camelCase
#[cfg(feature = "nodejs")]
#[napi]
impl MyStruct {
    #[napi]
    pub fn get_value(&self) -> i32 { ... }
}
```

## Migration Checklist

- [ ] Add `bridgerust` dependency to `Cargo.toml`
- [ ] Update feature flags (add both `python` and `nodejs`)
- [ ] Replace `#[pyfunction]` with `#[bridgerust::export]`
- [ ] Replace `#[napi]` with `#[bridgerust::export]`
- [ ] Replace `#[pyclass]` with `#[bridgerust::export]`
- [ ] Update struct impl blocks (separate for Python/Node.js)
- [ ] Migrate error handling to `#[bridgerust::error]`
- [ ] Create missing project structure (python/ or nodejs/)
- [ ] Update build scripts to use `bridge build`
- [ ] Test Python bindings
- [ ] Test Node.js bindings
- [ ] Update CI/CD workflows
- [ ] Update documentation

## Testing Your Migration

### Test Python:

```bash
# Build
bridge build --target python

# Install
pip install target/wheels/*.whl

# Test
python -c "import your_module; print(your_module.greet('World'))"
```

### Test Node.js:

```bash
# Build
bridge build --target nodejs

# Test
cd nodejs
node -e "const m = require('./index.node'); console.log(m.greet('World'))"
```

## Troubleshooting

### "Function not found" errors

- Ensure `#[export]` is on the function
- Check that features are enabled in `Cargo.toml`
- Verify module registration includes the function

### Type conversion errors

- Check the [Type Mapping Reference](getting-started-bridgerust.md#type-mapping-reference)
- Some types may need explicit conversion
- Option and Vec should work automatically

### Build errors

- Run `bridge check` to validate project
- Ensure maturin/napi-rs CLI are installed
- Check feature flags are correct

## Example: Complete Migration

See `examples/bridgerust-example/` for a complete example of a migrated project with both Python and Node.js support.

## Getting Help

- [Troubleshooting Guide](TROUBLESHOOTING.md)
- [Getting Started Guide](getting-started-bridgerust.md)
- [Examples](EXAMPLES.md)
- [GitHub Issues](https://github.com/bridgerust/bridgerust/issues)

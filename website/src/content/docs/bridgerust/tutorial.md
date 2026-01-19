---
title: Tutorial
description: Build a complete project with BridgeRust.
---

# Tutorial: Building a Calculator

In this tutorial, we will build a cross-language Calculator library.

## Setup

```bash
bridge new calculator
cd calculator
```

## Implementation

We will create a specific `Calculator` struct.

```rust
use bridgerust::prelude::*;

#[bridge]
struct Calculator {
    value: f64
}

#[bridge_methods]
impl Calculator {
    #[constructor]
    fn new() -> Self {
        Self { value: 0.0 }
    }

    #[method]
    fn add(&mut self, val: f64) {
        self.value += val;
    }

    #[method]
    fn result(&self) -> f64 {
        self.value
    }
}
```

## Usage

### Python

```python
from calculator import Calculator
c = Calculator()
c.add(10.0)
print(c.result())
```

### Node.js

```javascript
import { Calculator } from "calculator";
const c = new Calculator();
c.add(10.0);
console.log(c.result());
```

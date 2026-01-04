// End-to-end tests for Node.js bindings

const {
  greet,
  add,
  multiply,
  is_even,
  divide,
  sum_numbers,
  might_fail,
  Point,
  Rectangle,
} = require("../bridgerust_e2e_test.node");

// Test functions
console.log("Testing functions...");

// Test greet
const greetResult = greet("World");
console.assert(
  greetResult === "Hello, World!",
  `Expected "Hello, World!", got "${greetResult}"`
);
console.log("✓ greet() works");

// Test add
const addResult = add(2, 3);
console.assert(addResult === 5, `Expected 5, got ${addResult}`);
console.log("✓ add() works");

// Test multiply
const multiplyResult = multiply(2.5, 4.0);
console.assert(multiplyResult === 10.0, `Expected 10.0, got ${multiplyResult}`);
console.log("✓ multiply() works");

// Test is_even
console.assert(is_even(2) === true, "Expected is_even(2) to be true");
console.assert(is_even(3) === false, "Expected is_even(3) to be false");
console.log("✓ is_even() works");

// Test divide
const divideResult = divide(10.0, 2.0);
console.assert(divideResult === 5.0, `Expected 5.0, got ${divideResult}`);
const divideNone = divide(10.0, 0.0);
console.assert(
  divideNone === null || divideNone === undefined,
  `Expected null/undefined, got ${divideNone}`
);
console.log("✓ divide() works");

// Test sum_numbers
const sumResult = sum_numbers([1, 2, 3, 4, 5]);
console.assert(sumResult === 15, `Expected 15, got ${sumResult}`);
console.log("✓ sum_numbers() works");

// Test might_fail
const mightFailSuccess = might_fail(5);
console.assert(mightFailSuccess === 10, `Expected 10, got ${mightFailSuccess}`);
try {
  might_fail(-1);
  console.assert(false, "Expected might_fail(-1) to throw");
} catch (e) {
  console.log("✓ might_fail() error handling works");
}

// Test structs
console.log("\nTesting structs...");

// Test Point
const point = new Point(3.0, 4.0);
console.assert(point.x === 3.0, `Expected x=3.0, got ${point.x}`);
console.assert(point.y === 4.0, `Expected y=4.0, got ${point.y}`);
const distance = point.distance();
console.assert(distance === 5.0, `Expected distance=5.0, got ${distance}`);
console.log("✓ Point works");

// Test Rectangle
const rect = new Rectangle(10.0, 20.0);
console.assert(rect.width === 10.0, `Expected width=10.0, got ${rect.width}`);
console.assert(
  rect.height === 20.0,
  `Expected height=20.0, got ${rect.height}`
);
const area = rect.area();
console.assert(area === 200.0, `Expected area=200.0, got ${area}`);
console.log("✓ Rectangle works");

console.log("\n✅ All tests passed!");

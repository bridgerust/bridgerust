// BridgeRust Example - Node.js Usage

const {
  greet,
  add,
  multiply,
  is_even,
  safe_divide,
  find_first_even,
  sum_numbers,
  filter_positive,
  double_all,
  safe_sqrt,
  safe_divide_result,
  Point,
  Rectangle,
  Calculator,
} = require("./index.node");

function main() {
  console.log("=".repeat(60));
  console.log("BridgeRust Example - Node.js");
  console.log("=".repeat(60));

  // Basic functions
  console.log("\n📦 Basic Functions:");
  console.log(`  greet('World'): ${greet("World")}`);
  console.log(`  add(2, 3): ${add(2, 3)}`);
  console.log(`  multiply(2.5, 4.0): ${multiply(2.5, 4.0)}`);
  console.log(`  is_even(4): ${is_even(4)}`);

  // Option handling
  console.log("\n🔀 Option Handling:");
  let result = safe_divide(10.0, 2.0);
  console.log(`  safe_divide(10.0, 2.0): ${result}`);
  result = safe_divide(10.0, 0.0);
  console.log(`  safe_divide(10.0, 0.0): ${result}`);
  result = find_first_even([1, 3, 4, 5]);
  console.log(`  find_first_even([1, 3, 4, 5]): ${result}`);

  // Vec handling
  console.log("\n📊 Vec Handling:");
  console.log(
    `  sum_numbers([1, 2, 3, 4, 5]): ${sum_numbers([1, 2, 3, 4, 5])}`
  );
  console.log(
    `  filter_positive([-2, -1, 0, 1, 2]): ${JSON.stringify(
      filter_positive([-2, -1, 0, 1, 2])
    )}`
  );
  console.log(
    `  double_all([1, 2, 3]): ${JSON.stringify(double_all([1, 2, 3]))}`
  );

  // Result handling
  console.log("\n✅ Result Handling:");
  try {
    result = safe_sqrt(16.0);
    console.log(`  safe_sqrt(16.0): ${result}`);
  } catch (e) {
    console.log(`  safe_sqrt(16.0): Error - ${e.message}`);
  }

  try {
    result = safe_sqrt(-1.0);
    console.log(`  safe_sqrt(-1.0): ${result}`);
  } catch (e) {
    console.log(`  safe_sqrt(-1.0): Error - ${e.message}`);
  }

  // Structs
  console.log("\n🏗️  Structs:");
  const point = new Point(3.0, 4.0);
  console.log(`  new Point(3.0, 4.0): Point(${point.x}, ${point.y})`);
  console.log(`  point.distance(): ${point.distance()}`);

  const point2 = new Point(0.0, 0.0);
  console.log(
    `  point.distance_to(new Point(0, 0)): ${point.distance_to(point2)}`
  );

  const rect = new Rectangle(10.0, 20.0);
  console.log(
    `  new Rectangle(10.0, 20.0): Rectangle(${rect.width}x${rect.height})`
  );
  console.log(`  rect.area(): ${rect.area()}`);
  console.log(`  rect.perimeter(): ${rect.perimeter()}`);

  // Mutable struct
  const calc = new Calculator(10.0);
  console.log(`\n  new Calculator(10.0):`);
  console.log(`    calc.add(5.0): ${calc.add(5.0)}`);
  console.log(`    calc.multiply(2.0): ${calc.multiply(2.0)}`);
  console.log(`    calc.get_value(): ${calc.get_value()}`);
  calc.reset();
  console.log(`    calc.reset(), get_value(): ${calc.get_value()}`);

  console.log("\n" + "=".repeat(60));
  console.log("✅ All examples completed successfully!");
  console.log("=".repeat(60));
}

main();

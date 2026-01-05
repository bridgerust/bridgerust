"""BridgeRust Example - Python Usage"""

import bridgerust_example

def main():
    print("=" * 60)
    print("BridgeRust Example - Python")
    print("=" * 60)
    
    # Basic functions
    print("\n📦 Basic Functions:")
    print(f"  greet('World'): {bridgerust_example.greet('World')}")
    print(f"  add(2, 3): {bridgerust_example.add(2, 3)}")
    print(f"  multiply(2.5, 4.0): {bridgerust_example.multiply(2.5, 4.0)}")
    print(f"  is_even(4): {bridgerust_example.is_even(4)}")
    
    # Option handling
    print("\n🔀 Option Handling:")
    result = bridgerust_example.safe_divide(10.0, 2.0)
    print(f"  safe_divide(10.0, 2.0): {result}")
    result = bridgerust_example.safe_divide(10.0, 0.0)
    print(f"  safe_divide(10.0, 0.0): {result}")
    result = bridgerust_example.find_first_even([1, 3, 4, 5])
    print(f"  find_first_even([1, 3, 4, 5]): {result}")
    
    # Vec handling
    print("\n📊 Vec Handling:")
    print(f"  sum_numbers([1, 2, 3, 4, 5]): {bridgerust_example.sum_numbers([1, 2, 3, 4, 5])}")
    print(f"  filter_positive([-2, -1, 0, 1, 2]): {bridgerust_example.filter_positive([-2, -1, 0, 1, 2])}")
    print(f"  double_all([1, 2, 3]): {bridgerust_example.double_all([1, 2, 3])}")
    
    # Result handling
    print("\n✅ Result Handling:")
    try:
        result = bridgerust_example.safe_sqrt(16.0)
        print(f"  safe_sqrt(16.0): {result}")
    except Exception as e:
        print(f"  safe_sqrt(16.0): Error - {e}")
    
    try:
        result = bridgerust_example.safe_sqrt(-1.0)
        print(f"  safe_sqrt(-1.0): {result}")
    except Exception as e:
        print(f"  safe_sqrt(-1.0): Error - {e}")
    
    # Structs
    print("\n🏗️  Structs:")
    point = bridgerust_example.Point(3.0, 4.0)
    print(f"  Point(3.0, 4.0): {point}")
    print(f"  point.distance(): {point.distance()}")
    
    point2 = bridgerust_example.Point(0.0, 0.0)
    print(f"  point.distance_to(Point(0, 0)): {point.distance_to(point2)}")
    
    # Operator overloading (Python)
    point3 = point + point2
    print(f"  point + Point(0, 0): {point3}")
    
    rect = bridgerust_example.Rectangle(10.0, 20.0)
    print(f"  Rectangle(10.0, 20.0): {rect}")
    print(f"  rect.area(): {rect.area()}")
    print(f"  rect.perimeter(): {rect.perimeter()}")
    
    # Mutable struct
    calc = bridgerust_example.Calculator(10.0)
    print("\n  Calculator(10.0):")
    print(f"    calc.add(5.0): {calc.add(5.0)}")
    print(f"    calc.multiply(2.0): {calc.multiply(2.0)}")
    print(f"    calc.get_value(): {calc.get_value()}")
    calc.reset()
    print(f"    calc.reset(), get_value(): {calc.get_value()}")
    
    print("\n" + "=" * 60)
    print("✅ All examples completed successfully!")
    print("=" * 60)

if __name__ == "__main__":
    main()


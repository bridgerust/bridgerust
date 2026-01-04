"""End-to-end tests for Python bindings"""

import pytest
import sys
from pathlib import Path

# Add the built module to path
sys.path.insert(0, str(Path(__file__).parent.parent / "target" / "wheels"))

try:
    import bridgerust_e2e_test
except ImportError:
    pytest.skip("Python bindings not built", allow_module_level=True)


class TestFunctions:
    """Test exported functions"""
    
    def test_greet(self):
        result = bridgerust_e2e_test.greet("World")
        assert result == "Hello, World!"
    
    def test_add(self):
        result = bridgerust_e2e_test.add(2, 3)
        assert result == 5
    
    def test_multiply(self):
        result = bridgerust_e2e_test.multiply(2.5, 4.0)
        assert result == 10.0
    
    def test_is_even(self):
        assert bridgerust_e2e_test.is_even(2) is True
        assert bridgerust_e2e_test.is_even(3) is False
    
    def test_divide(self):
        result = bridgerust_e2e_test.divide(10.0, 2.0)
        assert result == 5.0
        
        result = bridgerust_e2e_test.divide(10.0, 0.0)
        assert result is None
    
    def test_sum_numbers(self):
        result = bridgerust_e2e_test.sum_numbers([1, 2, 3, 4, 5])
        assert result == 15
    
    def test_might_fail(self):
        result = bridgerust_e2e_test.might_fail(5)
        assert result == 10
        
        with pytest.raises(Exception):
            bridgerust_e2e_test.might_fail(-1)


class TestStructs:
    """Test exported structs"""
    
    def test_point(self):
        point = bridgerust_e2e_test.Point(3.0, 4.0)
        assert point.x == 3.0
        assert point.y == 4.0
        assert point.distance() == 5.0
        assert "Point" in repr(point)
    
    def test_rectangle(self):
        rect = bridgerust_e2e_test.Rectangle(10.0, 20.0)
        assert rect.width == 10.0
        assert rect.height == 20.0
        assert rect.area() == 200.0
        assert "Rectangle" in repr(rect)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])


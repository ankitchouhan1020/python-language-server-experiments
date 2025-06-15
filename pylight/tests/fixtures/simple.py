"""Simple Python file for testing symbol extraction."""

def simple_function():
    """A simple function."""
    pass

def function_with_args(arg1, arg2):
    """Function with arguments."""
    return arg1 + arg2

class SimpleClass:
    """A simple class."""
    
    def __init__(self):
        self.value = 0
    
    def method(self):
        """A simple method."""
        return self.value
    
    def method_with_args(self, x, y):
        """Method with arguments."""
        return x + y

class AnotherClass:
    """Another class for testing."""
    pass

@decorator
def decorated_function():
    """A decorated function."""
    pass

@property
def property_function():
    """A property function."""
    pass
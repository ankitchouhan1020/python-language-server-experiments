"""Test file for nested structures."""

def outer_function():
    """Outer function."""
    
    def inner_function():
        """Inner function."""
        
        def deeply_nested():
            """Deeply nested function."""
            pass
        
        return deeply_nested
    
    return inner_function

class OuterClass:
    """Outer class."""
    
    class InnerClass:
        """Inner class."""
        
        def inner_method(self):
            """Method in inner class."""
            pass
    
    def outer_method(self):
        """Method in outer class."""
        
        def method_inner_function():
            """Function inside method."""
            pass
        
        return method_inner_function
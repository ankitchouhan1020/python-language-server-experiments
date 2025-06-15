"""Test file for various decorator patterns."""

from functools import wraps

def my_decorator(func):
    @wraps(func)
    def wrapper(*args, **kwargs):
        return func(*args, **kwargs)
    return wrapper

@my_decorator
def decorated_function():
    pass

@property
def property_method():
    pass

@staticmethod
def static_method():
    pass

@classmethod
def class_method(cls):
    pass

class DecoratedClass:
    @property
    def value(self):
        return self._value
    
    @value.setter
    def value(self, val):
        self._value = val
    
    @staticmethod
    def static_in_class():
        pass
    
    @classmethod
    def class_in_class(cls):
        pass

@my_decorator
@another_decorator
def multi_decorated():
    pass
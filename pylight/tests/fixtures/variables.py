"""Test file for global variable extraction."""

# Simple assignments
MY_CONST = 42
config = {}
name = "hello"

# Annotated assignments
MY_VAR: int = 100
server_name: str

# Multi-target assignment
a = b = 10

# Tuple unpacking
x, y = 1, 2

# Type alias (old-style, appears as regular assignment)
from typing import Union
MyType = Union[int, str]

# Augmented assignment (should NOT be extracted)
MY_CONST += 1

# Attribute assignment (should NOT be extracted)
import os
os.environ["KEY"] = "value"

# Variable inside a function (should NOT be extracted)
def some_function():
    local_var = "not extracted"
    return local_var

# Variable inside a class (should NOT be extracted)
class SomeClass:
    class_attr = "not extracted"

    def method(self):
        self.instance_attr = "not extracted"

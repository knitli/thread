# Sample Python code for benchmarking
def test_function():
    print("Hello World")
    print('test string')
    print(f"template {variable}")

class TestClass:
    def __init__(self):
        self.value = 42

    def method(self):
        print(self.value)

variable = "test"
constant = 123
old_var = True

import os
from typing import List, Dict
import asyncio

async def async_function():
    result = await fetch_data()
    return result

def recursion():
    recursion()

recursion2 = lambda: recursion2()

if __name__ == "__main__":
    test_function()

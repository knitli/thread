#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
# SPDX-FileCopyrightText: 2026 Knitli Inc.
#
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Sample Python code for testing ThreadParse functionality"""

import os
import sys
from typing import List, Dict, Optional
from dataclasses import dataclass


@dataclass
class User:
    """Represents a user in the system"""
    id: int
    name: str
    email: str


class UserManager:
    """Manages user operations"""

    def __init__(self):
        self.users: Dict[int, User] = {}

    def add_user(self, user: User) -> None:
        """Add a user to the manager"""
        if not user.name:
            raise ValueError("Name cannot be empty")
        self.users[user.id] = user

    def get_user(self, user_id: int) -> Optional[User]:
        """Retrieve a user by ID"""
        return self.users.get(user_id)

    def calculate_total(self, values: List[int]) -> int:
        """Calculate sum of values"""
        return sum(values)


def process_user(user: User) -> str:
    """Process user data and return formatted string"""
    if not user.name:
        raise ValueError("Name cannot be empty")
    return f"User: {user.name} ({user.email})"


def main():
    """Main entry point"""
    user = User(id=1, name="Alice", email="alice@example.com")
    manager = UserManager()

    manager.add_user(user)
    result = process_user(user)
    print(result)

    numbers = [1, 2, 3, 4, 5]
    total = manager.calculate_total(numbers)
    print(f"Total: {total}")


if __name__ == "__main__":
    main()

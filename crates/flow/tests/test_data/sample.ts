// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

/**
 * Sample TypeScript code for testing ThreadParse functionality
 */

import { EventEmitter } from 'events';
import * as path from 'path';

/**
 * User interface representing a system user
 */
export interface User {
    id: number;
    name: string;
    email: string;
}

/**
 * Role enum for user permissions
 */
export enum Role {
    Admin = "admin",
    User = "user",
    Guest = "guest",
}

/**
 * User manager class for handling user operations
 */
export class UserManager extends EventEmitter {
    private users: Map<number, User>;

    constructor() {
        super();
        this.users = new Map();
    }

    /**
     * Add a user to the manager
     */
    addUser(user: User): void {
        if (!user.name) {
            throw new Error("Name cannot be empty");
        }
        this.users.set(user.id, user);
        this.emit('userAdded', user);
    }

    /**
     * Get a user by ID
     */
    getUser(userId: number): User | undefined {
        return this.users.get(userId);
    }

    /**
     * Calculate total from array of numbers
     */
    calculateTotal(values: number[]): number {
        return values.reduce((sum, val) => sum + val, 0);
    }
}

/**
 * Process user data and return formatted string
 */
export function processUser(user: User): string {
    if (!user.name) {
        throw new Error("Name cannot be empty");
    }
    return `User: ${user.name} (${user.email})`;
}

/**
 * Main function demonstrating usage
 */
function main(): void {
    const user: User = {
        id: 1,
        name: "Alice",
        email: "alice@example.com",
    };

    const manager = new UserManager();
    manager.addUser(user);

    const result = processUser(user);
    console.log(result);

    const numbers = [1, 2, 3, 4, 5];
    const total = manager.calculateTotal(numbers);
    console.log(`Total: ${total}`);
}

main();

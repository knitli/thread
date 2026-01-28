// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

// Sample Go code for testing ThreadParse functionality
package main

import (
	"errors"
	"fmt"
	"log"
)

// User represents a system user
type User struct {
	ID    uint64
	Name  string
	Email string
}

// Role represents user permissions
type Role int

const (
	Admin Role = iota
	UserRole
	Guest
)

// UserManager manages user operations
type UserManager struct {
	users map[uint64]*User
}

// NewUserManager creates a new user manager
func NewUserManager() *UserManager {
	return &UserManager{
		users: make(map[uint64]*User),
	}
}

// AddUser adds a user to the manager
func (m *UserManager) AddUser(user *User) error {
	if user.Name == "" {
		return errors.New("name cannot be empty")
	}
	m.users[user.ID] = user
	return nil
}

// GetUser retrieves a user by ID
func (m *UserManager) GetUser(userID uint64) (*User, bool) {
	user, ok := m.users[userID]
	return user, ok
}

// CalculateTotal calculates sum of values
func (m *UserManager) CalculateTotal(values []int) int {
	total := 0
	for _, v := range values {
		total += v
	}
	return total
}

// ProcessUser processes user data and returns formatted string
func ProcessUser(user *User) (string, error) {
	if user.Name == "" {
		return "", errors.New("name cannot be empty")
	}
	return fmt.Sprintf("User: %s (%s)", user.Name, user.Email), nil
}

func main() {
	user := &User{
		ID:    1,
		Name:  "Alice",
		Email: "alice@example.com",
	}

	manager := NewUserManager()
	if err := manager.AddUser(user); err != nil {
		log.Fatal(err)
	}

	result, err := ProcessUser(user)
	if err != nil {
		log.Fatal(err)
	}
	fmt.Println(result)

	numbers := []int{1, 2, 3, 4, 5}
	total := manager.CalculateTotal(numbers)
	fmt.Printf("Total: %d\n", total)
}

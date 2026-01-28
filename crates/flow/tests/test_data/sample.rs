// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Sample Rust code for testing ThreadParse functionality

use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// A sample struct representing a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
}

/// A sample enum for user roles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Admin,
    User,
    Guest,
}

/// Process user data and return a result
pub fn process_user(user: &User) -> Result<String, String> {
    if user.name.is_empty() {
        return Err("Name cannot be empty".to_string());
    }

    let formatted = format!("User: {} ({})", user.name, user.email);
    Ok(formatted)
}

/// Calculate total from a list of values
pub fn calculate_total(values: &[i32]) -> i32 {
    values.iter().sum()
}

/// Main function with multiple calls
pub fn main() {
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    match process_user(&user) {
        Ok(result) => println!("{}", result),
        Err(e) => eprintln!("Error: {}", e),
    }

    let numbers = vec![1, 2, 3, 4, 5];
    let total = calculate_total(&numbers);
    println!("Total: {}", total);
}

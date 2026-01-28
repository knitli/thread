/// Simple calculator with basic arithmetic operations
pub struct Calculator {
    result: f64,
}

impl Calculator {
    /// Create a new calculator with initial value
    pub fn new(initial: f64) -> Self {
        Self { result: initial }
    }

    /// Add a value to the current result
    pub fn add(&mut self, value: f64) -> &mut Self {
        self.result += value;
        self
    }

    /// Subtract a value from the current result
    pub fn subtract(&mut self, value: f64) -> &mut Self {
        self.result -= value;
        self
    }

    /// Multiply the current result by a value
    pub fn multiply(&mut self, value: f64) -> &mut Self {
        self.result *= value;
        self
    }

    /// Divide the current result by a value
    pub fn divide(&mut self, value: f64) -> Result<&mut Self, &'static str> {
        if value == 0.0 {
            Err("Division by zero")
        } else {
            self.result /= value;
            Ok(self)
        }
    }

    /// Get the current result
    pub fn get(&self) -> f64 {
        self.result
    }

    /// Reset to zero
    pub fn reset(&mut self) {
        self.result = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let mut calc = Calculator::new(10.0);
        calc.add(5.0).multiply(2.0);
        assert_eq!(calc.get(), 30.0);
    }

    #[test]
    fn test_division_by_zero() {
        let mut calc = Calculator::new(10.0);
        assert!(calc.divide(0.0).is_err());
    }
}

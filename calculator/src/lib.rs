#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CalcOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CalcError {
    DivideByZero,
    Overflow,
}

pub fn calculate(op: CalcOp, a: f64, b: f64) -> Result<f64, CalcError> {
    match op {
        CalcOp::Add => {
            let result = a + b;
            if result.is_infinite() {
                Err(CalcError::Overflow)
            } else {
                Ok(result)
            }
        }
        CalcOp::Subtract => {
            let result = a - b;
            if result.is_infinite() {
                Err(CalcError::Overflow)
            } else {
                Ok(result)
            }
        }
        CalcOp::Multiply => {
            let result = a * b;
            if result.is_infinite() {
                Err(CalcError::Overflow)
            } else {
                Ok(result)
            }
        }
        CalcOp::Divide => {
            if b == 0.0 {
                return Err(CalcError::DivideByZero);
            }
            let result = a / b;
            if result.is_infinite() {
                Err(CalcError::Overflow)
            } else {
                Ok(result)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Calculator;

impl Calculator {
    pub fn new() -> Self {
        Self
    }

    pub fn eval(&self, input: &str) -> Result<f64, String> {
        let input = input.trim();
        if input.is_empty() {
            return Err("Empty input".to_string());
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.len() != 3 {
            return Err("Expected format: <a> <op> <b> (e.g. 2 + 3)".to_string());
        }

        let a: f64 = parts[0]
            .parse()
            .map_err(|_| format!("Invalid number: {}", parts[0]))?;
        let op = match parts[1] {
            "+" => CalcOp::Add,
            "-" => CalcOp::Subtract,
            "*" => CalcOp::Multiply,
            "/" => CalcOp::Divide,
            other => return Err(format!("Unknown operator: {}", other)),
        };
        let b: f64 = parts[2]
            .parse()
            .map_err(|_| format!("Invalid number: {}", parts[2]))?;

        calculate(op, a, b).map_err(|e| match e {
            CalcError::DivideByZero => "Division by zero".to_string(),
            CalcError::Overflow => "Numeric overflow".to_string(),
        })
    }
}

impl Default for Calculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(calculate(CalcOp::Add, 2.0, 3.0).unwrap(), 5.0);
    }

    #[test]
    fn test_subtract() {
        assert_eq!(calculate(CalcOp::Subtract, 10.0, 4.0).unwrap(), 6.0);
    }

    #[test]
    fn test_multiply() {
        assert_eq!(calculate(CalcOp::Multiply, 3.0, 7.0).unwrap(), 21.0);
    }

    #[test]
    fn test_divide() {
        assert_eq!(calculate(CalcOp::Divide, 15.0, 3.0).unwrap(), 5.0);
    }

    #[test]
    fn test_divide_by_zero() {
        assert_eq!(
            calculate(CalcOp::Divide, 5.0, 0.0).unwrap_err(),
            CalcError::DivideByZero
        );
    }

    #[test]
    fn test_negative_numbers() {
        assert_eq!(calculate(CalcOp::Add, -5.0, 3.0).unwrap(), -2.0);
        assert_eq!(calculate(CalcOp::Subtract, -5.0, -3.0).unwrap(), -2.0);
        assert_eq!(calculate(CalcOp::Multiply, -2.0, 4.0).unwrap(), -8.0);
        assert_eq!(calculate(CalcOp::Divide, -10.0, 2.0).unwrap(), -5.0);
    }

    #[test]
    fn test_floating_point() {
        let result = calculate(CalcOp::Add, 0.1, 0.2).unwrap();
        assert!((result - 0.3).abs() < 1e-10);
    }

    #[test]
    fn test_eval_addition() {
        let calc = Calculator::new();
        assert_eq!(calc.eval("2 + 3").unwrap(), 5.0);
    }

    #[test]
    fn test_eval_subtraction() {
        let calc = Calculator::new();
        assert_eq!(calc.eval("10 - 4").unwrap(), 6.0);
    }

    #[test]
    fn test_eval_multiplication() {
        let calc = Calculator::new();
        assert_eq!(calc.eval("3 * 7").unwrap(), 21.0);
    }

    #[test]
    fn test_eval_division() {
        let calc = Calculator::new();
        assert_eq!(calc.eval("15 / 3").unwrap(), 5.0);
    }

    #[test]
    fn test_eval_divide_by_zero() {
        let calc = Calculator::new();
        assert_eq!(calc.eval("5 / 0").unwrap_err(), "Division by zero");
    }

    #[test]
    fn test_eval_empty_input() {
        let calc = Calculator::new();
        assert_eq!(calc.eval("").unwrap_err(), "Empty input");
    }

    #[test]
    fn test_eval_invalid_number() {
        let calc = Calculator::new();
        assert!(calc.eval("abc + 3").unwrap_err().contains("Invalid number"));
    }

    #[test]
    fn test_eval_unknown_operator() {
        let calc = Calculator::new();
        assert_eq!(calc.eval("2 ^ 3").unwrap_err(), "Unknown operator: ^");
    }

    #[test]
    fn test_eval_wrong_arg_count() {
        let calc = Calculator::new();
        assert!(calc.eval("2 +").unwrap_err().contains("Expected format"));
    }

    #[test]
    fn test_calculator_default() {
        let calc: Calculator = Default::default();
        assert_eq!(calc.eval("2 + 2").unwrap(), 4.0);
    }
}

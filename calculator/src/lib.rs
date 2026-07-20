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

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    Eof,
}

struct Lexer {
    chars: Vec<char>,
    pos: usize,
}

impl Lexer {
    fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn next_token(&mut self) -> Result<Token, String> {
        self.skip_whitespace();

        if self.pos >= self.chars.len() {
            return Ok(Token::Eof);
        }

        let ch = self.chars[self.pos];
        match ch {
            '+' => {
                self.pos += 1;
                Ok(Token::Plus)
            }
            '-' => {
                self.pos += 1;
                Ok(Token::Minus)
            }
            '*' => {
                self.pos += 1;
                Ok(Token::Star)
            }
            '/' => {
                self.pos += 1;
                Ok(Token::Slash)
            }
            '(' => {
                self.pos += 1;
                Ok(Token::LParen)
            }
            ')' => {
                self.pos += 1;
                Ok(Token::RParen)
            }
            _ if ch.is_ascii_digit() || ch == '.' => {
                let start = self.pos;
                while self.pos < self.chars.len()
                    && (self.chars[self.pos].is_ascii_digit() || self.chars[self.pos] == '.')
                {
                    self.pos += 1;
                }
                let s: String = self.chars[start..self.pos].iter().collect();
                s.parse::<f64>()
                    .map(Token::Number)
                    .map_err(|_| format!("Invalid number: {}", s))
            }
            _ if ch.is_ascii_alphabetic() => {
                let start = self.pos;
                while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_alphabetic() {
                    self.pos += 1;
                }
                let s: String = self.chars[start..self.pos].iter().collect();
                Err(format!("Invalid number: {}", s))
            }
            _ => {
                let s = ch.to_string();
                self.pos += 1;
                Err(format!("Unknown operator: {}", s))
            }
        }
    }
}

struct Parser {
    lexer: Lexer,
    current_token: Token,
}

impl Parser {
    fn new(input: &str) -> Result<Self, String> {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.next_token()?;
        Ok(Self {
            lexer,
            current_token,
        })
    }

    fn consume(&mut self) -> Result<Token, String> {
        let token = self.current_token.clone();
        self.current_token = self.lexer.next_token()?;
        Ok(token)
    }

    fn parse_expr(&mut self) -> Result<f64, String> {
        let mut left = self.parse_term()?;
        loop {
            match &self.current_token {
                Token::Plus => {
                    self.consume()?;
                    let right = self.parse_term()?;
                    left = calculate(CalcOp::Add, left, right).map_err(|e| match e {
                        CalcError::DivideByZero => "Division by zero".to_string(),
                        CalcError::Overflow => "Numeric overflow".to_string(),
                    })?;
                }
                Token::Minus => {
                    self.consume()?;
                    let right = self.parse_term()?;
                    left = calculate(CalcOp::Subtract, left, right).map_err(|e| match e {
                        CalcError::DivideByZero => "Division by zero".to_string(),
                        CalcError::Overflow => "Numeric overflow".to_string(),
                    })?;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_term(&mut self) -> Result<f64, String> {
        let mut left = self.parse_factor()?;
        loop {
            match &self.current_token {
                Token::Star => {
                    self.consume()?;
                    let right = self.parse_factor()?;
                    left = calculate(CalcOp::Multiply, left, right).map_err(|e| match e {
                        CalcError::DivideByZero => "Division by zero".to_string(),
                        CalcError::Overflow => "Numeric overflow".to_string(),
                    })?;
                }
                Token::Slash => {
                    self.consume()?;
                    let right = self.parse_factor()?;
                    left = calculate(CalcOp::Divide, left, right).map_err(|e| match e {
                        CalcError::DivideByZero => "Division by zero".to_string(),
                        CalcError::Overflow => "Numeric overflow".to_string(),
                    })?;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<f64, String> {
        match &self.current_token {
            Token::Number(n) => {
                let val = *n;
                self.consume()?;
                Ok(val)
            }
            Token::LParen => {
                self.consume()?;
                let val = self.parse_expr()?;
                if self.current_token != Token::RParen {
                    return Err("Mismatched parentheses".to_string());
                }
                self.consume()?;
                Ok(val)
            }
            Token::Minus => {
                self.consume()?;
                let val = self.parse_factor()?;
                Ok(-val)
            }
            Token::Eof => Err("Unexpected end of expression".to_string()),
            _ => Err(format!("Unexpected token: {:?}", self.current_token)),
        }
    }

    fn parse(mut self) -> Result<f64, String> {
        let result = self.parse_expr()?;
        if self.current_token != Token::Eof {
            return Err("Unexpected input after expression".to_string());
        }
        Ok(result)
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

        let parser = Parser::new(input)?;
        parser.parse()
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
        assert_eq!(
            calc.eval("2 +").unwrap_err(),
            "Unexpected end of expression"
        );
    }

    #[test]
    fn test_calculator_default() {
        let calc: Calculator = Default::default();
        assert_eq!(calc.eval("2 + 2").unwrap(), 4.0);
    }

    #[test]
    fn test_precedence_multiplication_over_addition() {
        let calc = Calculator::new();
        assert_eq!(calc.eval("2 + 3 * 4").unwrap(), 14.0);
    }

    #[test]
    fn test_precedence_division_over_subtraction() {
        let calc = Calculator::new();
        assert_eq!(calc.eval("10 - 6 / 3").unwrap(), 8.0);
    }

    #[test]
    fn test_parentheses_override_precedence() {
        let calc = Calculator::new();
        assert_eq!(calc.eval("(2 + 3) * 4").unwrap(), 20.0);
    }

    #[test]
    fn test_nested_parentheses() {
        let calc = Calculator::new();
        assert_eq!(calc.eval("((2 + 3) * 2)").unwrap(), 10.0);
    }

    #[test]
    fn test_chained_addition() {
        let calc = Calculator::new();
        assert_eq!(calc.eval("1 + 2 + 3 + 4").unwrap(), 10.0);
    }

    #[test]
    fn test_chained_multiplication() {
        let calc = Calculator::new();
        assert_eq!(calc.eval("2 * 3 * 4").unwrap(), 24.0);
    }

    #[test]
    fn test_whitespace_tolerance() {
        let calc = Calculator::new();
        assert_eq!(calc.eval("2+3").unwrap(), 5.0);
        assert_eq!(calc.eval("2   +   3").unwrap(), 5.0);
        assert_eq!(calc.eval("2+3*4").unwrap(), 14.0);
    }

    #[test]
    fn test_single_number() {
        let calc = Calculator::new();
        assert_eq!(calc.eval("42").unwrap(), 42.0);
    }

    #[test]
    fn test_unary_minus() {
        let calc = Calculator::new();
        assert_eq!(calc.eval("-5 + 3").unwrap(), -2.0);
    }

    #[test]
    fn test_unary_minus_with_parens() {
        let calc = Calculator::new();
        assert_eq!(calc.eval("-(2 + 3)").unwrap(), -5.0);
    }

    #[test]
    fn test_expression_divide_by_zero() {
        let calc = Calculator::new();
        assert_eq!(
            calc.eval("5 / (2 - 2)").unwrap_err(),
            "Division by zero"
        );
    }

    #[test]
    fn test_mismatched_parentheses() {
        let calc = Calculator::new();
        assert_eq!(
            calc.eval("(2 + 3").unwrap_err(),
            "Mismatched parentheses"
        );
    }
}

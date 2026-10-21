//! Recursive-descent parser.
//!
//! Grammar, lowest precedence first:
//! `equiv` → `implies` (right assoc) → `or` → `xor` → `and` → unary `not` → primary.

use std::fmt;

use crate::ast::Expr;
use crate::value::Ternary;

/// Parse error with a character offset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// Character offset where parsing failed.
    pub pos: usize,
    /// Human-readable message.
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "解析失败（位置 {}）：{}", self.pos, self.message)
    }
}

impl std::error::Error for ParseError {}

/// Parse an expression.
pub fn parse(src: &str) -> Result<Expr, ParseError> {
    let mut parser = Parser {
        chars: src.chars().collect(),
        pos: 0,
    };
    let expr = parser.expr()?;
    parser.skip_ws();
    if parser.peek().is_some() {
        return parser.err("表达式末尾有多余字符");
    }
    Ok(expr)
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(c) if c.is_whitespace()) {
            self.pos += 1;
        }
    }

    fn eat(&mut self, c: char) -> bool {
        self.skip_ws();
        if self.peek() == Some(c) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn eat_str(&mut self, s: &str) -> bool {
        self.skip_ws();
        let mut i = self.pos;
        for c in s.chars() {
            if self.chars.get(i) != Some(&c) {
                return false;
            }
            i += 1;
        }
        self.pos = i;
        true
    }

    fn err<T>(&self, message: impl Into<String>) -> Result<T, ParseError> {
        Err(ParseError {
            pos: self.pos,
            message: message.into(),
        })
    }

    fn expr(&mut self) -> Result<Expr, ParseError> {
        self.equiv()
    }

    fn equiv(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.implies()?;
        loop {
            if self.eat_str("<=>") || self.eat_str("==") || self.eat('≡') {
                let right = self.implies()?;
                left = Expr::equiv(left, right);
            } else {
                return Ok(left);
            }
        }
    }

    fn implies(&mut self) -> Result<Expr, ParseError> {
        let left = self.or()?;
        if self.eat_str("=>") || self.eat('→') {
            let right = self.implies()?;
            Ok(Expr::implies(left, right))
        } else {
            Ok(left)
        }
    }

    fn or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.xor()?;
        loop {
            if self.eat_str("||") || self.eat('|') || self.eat('∨') {
                let right = self.xor()?;
                left = Expr::or(left, right);
            } else {
                return Ok(left);
            }
        }
    }

    fn xor(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.and()?;
        loop {
            if self.eat('^') || self.eat('⊕') {
                let right = self.and()?;
                left = Expr::xor(left, right);
            } else {
                return Ok(left);
            }
        }
    }

    fn and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.unary()?;
        loop {
            if self.eat_str("&&") || self.eat('&') || self.eat('∧') {
                let right = self.unary()?;
                left = Expr::and(left, right);
            } else {
                return Ok(left);
            }
        }
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.eat('!') || self.eat('~') || self.eat('¬') {
            Ok(Expr::logical_not(self.unary()?))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        self.skip_ws();
        if self.eat('(') {
            let expr = self.expr()?;
            if !self.eat(')') {
                return self.err("缺少右括号");
            }
            return Ok(expr);
        }
        let Some(c) = self.peek() else {
            return self.err("缺少表达式");
        };
        if c == '?' {
            self.pos += 1;
            return Ok(Expr::Const(Ternary::Unknown));
        }
        if c.is_ascii_digit() {
            let mut digits = String::new();
            while matches!(self.peek(), Some(d) if d.is_ascii_digit()) {
                digits.push(self.peek().expect("peeked"));
                self.pos += 1;
            }
            return match digits.as_str() {
                "0" => Ok(Expr::Const(Ternary::False)),
                "1" => Ok(Expr::Const(Ternary::True)),
                _ => self.err(format!("非法常量 {digits}")),
            };
        }
        if c.is_alphabetic() || c == '_' {
            let mut name = String::new();
            while matches!(self.peek(), Some(d) if d.is_alphanumeric() || d == '_') {
                name.push(self.peek().expect("peeked"));
                self.pos += 1;
            }
            return Ok(match name.as_str() {
                "X" | "x" => Expr::Const(Ternary::Unknown),
                "True" | "true" => Expr::Const(Ternary::True),
                "False" | "false" => Expr::Const(Ternary::False),
                _ => Expr::Var(name),
            });
        }
        self.err(format!("无法识别的字符 {c:?}"))
    }
}

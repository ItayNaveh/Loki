use crate::lexer::Token;

/*
For now i think that the root can only be const assignments

Root = (const_assignmnet)*
// TODO: maybe const assignments don't need a ';'
const_assignment = ident "::" expr;

expr = func | ident | literal
func = "fn" "(" (ident ":" ident)* ")" ("->" ident)? "{" (statement)* "}"
*/

#[derive(Debug)]
pub struct AstRoot(pub Vec<ConstAssignment>);

#[derive(Debug)]
pub struct ConstAssignment(pub String, pub ConstAssignmentVal);

#[derive(Debug)]
pub enum ConstAssignmentVal {
	Function { args: Vec<(String, String)>, return_type: Option<String>, body: Vec<Statement> },
	Expression(Expression),
}

#[derive(Debug)]
pub enum Expression {
	NumberLiteral(i64),
	StringLiteral(String),
	Ident(String),

	// FIXME: op shouldn't be a Token
	BinaryOperator { op: Token, left: Box<Expression>, right: Box<Expression> },

	FunctionCall(String, Vec<Expression>),
}

#[derive(Debug)]
pub enum Statement {
	Return(Expression),
	Expression(Expression),
}

pub fn parse(tokens: Vec<Token>) -> AstRoot {
	Parser { tokens: &tokens, pos: 0 }.parse()
}

struct Parser<'a> {
	tokens: &'a [Token],
	pos: usize,
}

impl<'a> Parser<'a> {
	fn parse(mut self) -> AstRoot {
		let mut root = Vec::new();
		while self.pos < self.tokens.len() {
			root.push(self.parse_const_assignment());
		}

		return AstRoot(root);
	}

	#[inline(always)]
	const fn at(&self) -> &Token { &self.tokens[self.pos] }

	// fn expect_ident(&self) -> Option<String> { match self.at() {
	// 	Token::Ident(ident) => Some(ident.clone()),
	// 	_ => None,
	// } }

	#[must_use]
	fn consume_ident(&mut self) -> Option<String> {
		// self.expect_ident().and_then(|x| { self.pos += 1; Some(x) })
		self.at().ident().and_then(|x| { self.pos += 1; Some(x) })
	}

	#[must_use]
	fn consume(&mut self, token: Token) -> Option<()> {
		if std::mem::discriminant(self.at()) == std::mem::discriminant(&token) {
			self.pos += 1;
			Some(())
		} else {
			None
		}
	}

	fn parse_const_assignment(&mut self) -> ConstAssignment {
		let ident = self.consume_ident().unwrap();

		self.consume(Token::ColonColon).unwrap();

		let val = match self.at() {
			Token::Fn => {
				self.pos += 1;

				self.consume(Token::ParenOpen).unwrap();

				let mut args = Vec::new();
				while self.pos < self.tokens.len() {
					if *self.at() == Token::ParenClose { break }

					let name = self.consume_ident().unwrap();

					self.consume(Token::Colon).unwrap();

					let type_ = self.consume_ident().unwrap();

					args.push((name.clone(), type_.clone()));
					if *self.at() == Token::ParenClose { break }

					self.consume(Token::Comma).unwrap();
				}

				self.consume(Token::ParenClose).unwrap();

				let return_type = if *self.at() == Token::Arrow {
					self.pos += 1;

					let return_type = self.consume_ident().unwrap();
					Some(return_type.clone())
				} else {
					None
				};

				self.consume(Token::BraceOpen).unwrap();

				let mut body = Vec::new();
				while *self.at() != Token::BraceClose {
					body.push(self.parse_statement());
				}

				self.consume(Token::BraceClose).unwrap();

				ConstAssignmentVal::Function { args, return_type, body }
			},
			_ => ConstAssignmentVal::Expression(self.parse_expr()),
		};

		self.consume(Token::Semicolon).unwrap();

		ConstAssignment(ident, val)
	}

	fn parse_expr(&mut self) -> Expression {
		self.parse_additive()
	}

	fn parse_multiplicative(&mut self) -> Expression {
		let mut left = self.parse_primary_expr();

		while matches!(self.at(), Token::Star) {
			let op = self.at().clone();
			self.pos += 1;

			let right = self.parse_primary_expr();
			left = Expression::BinaryOperator { op, left: Box::new(left), right: Box::new(right) };
		}

		left
	}

	fn parse_additive(&mut self) -> Expression {
		let mut left = self.parse_multiplicative();

		while matches!(self.at(), Token::Plus) {
			let op = self.at().clone();
			self.pos += 1;

			let right = self.parse_multiplicative();
			left = Expression::BinaryOperator { op, left: Box::new(left), right: Box::new(right) };
		}

		left
	}

	fn parse_primary_expr(&mut self) -> Expression {
		match self.tokens[self.pos] {
			Token::NumberLiteral(n) => { self.pos += 1; Expression::NumberLiteral(n) },
			Token::StringLiteral(ref s) => { self.pos += 1; Expression::StringLiteral(s.clone()) },

			Token::Ident(ref ident) if self.tokens[self.pos + 1] == Token::ParenOpen => {
				self.pos += 2; // ident + (
				let mut args = Vec::new();
				while *self.at() != Token::ParenClose {
					args.push(self.parse_expr());

					if *self.at() != Token::Comma { break }
					self.pos += 1;
				}

				self.consume(Token::ParenClose).unwrap();

				Expression::FunctionCall(ident.clone(), args)
			},

			Token::Ident(ref ident) => { self.pos += 1; Expression::Ident(ident.clone()) },

			ref t => panic!("Unexpected token while parsing expression: {t:?}"),
		}
	}

	fn parse_statement(&mut self) -> Statement {
		match self.at() {
			Token::Return => {
				self.pos += 1;

				let expr = self.parse_expr();
				self.consume(Token::Semicolon).unwrap();

				Statement::Return(expr)
			},

			_ => {
				let expr = self.parse_expr();
				self.consume(Token::Semicolon).unwrap();

				Statement::Expression(expr)
			},

			// ref t => panic!("Unexpected token while parsing statement: {t:?}"),
		}
	}
}

// fn is_ident(token: &Token) -> bool { matches!(token, Token::Ident(_)) }
// fn expect_ident(token: &Token) -> Option<&String> { match token { Token::Ident(s) => Some(s), _ => None } }

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
pub enum Operator {
	Assign,
	Add,
	Multiply,
	Deref,
	IsEqual,
	UnaryPlus,
	IsLessThan,
	IsGreaterThan,
}

#[derive(Debug)]
pub enum Expression {
	NumberLiteral(i64),
	StringLiteral(String),
	Ident(String),

	BinaryOperator { op: Operator, left: Box<Expression>, right: Box<Expression> },
	UnaryOperator { op: Operator, operand: Box<Expression> },

	FunctionCall(String, Vec<Expression>),
}

#[derive(Debug)]
pub enum Statement {
	Return(Expression),
	Let(String, String, Expression),
	// TODO: make if an expression
	If(Expression, Box<Statement>),
	Expression(Expression),
}

pub fn parse(tokens: Vec<Token>) -> AstRoot {
	Parser { tokens: &tokens, pos: 0 }.parse()
}

impl Operator {
	fn to_binary_op(token: &Token) -> Option<Self> {
		match token {
			Token::Equals => Some(Self::Assign),
			Token::Plus => Some(Self::Add),
			Token::Star => Some(Self::Multiply),
			Token::EqualsEquals => Some(Self::IsEqual),
			Token::AngleBracketOpen => Some(Self::IsLessThan),
			Token::AngleBracketClose => Some(Self::IsGreaterThan),
			_ => None,
		}
	}

	fn to_unary_op(token: &Token) -> Option<Self> {
		match token {
			Token::Plus => Some(Self::UnaryPlus),
			Token::Star => Some(Self::Deref),
			_ => None,
		}
	}
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
		let at = self.at().ident();
		if at.is_some() {
			self.pos += 1;
		}

		at
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
					Some(return_type)
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
		self.parse_expr_p0()
	}

	fn parse_statement(&mut self) -> Statement {
		match self.at() {
			Token::Return => {
				self.pos += 1;

				let expr = self.parse_expr();
				self.consume(Token::Semicolon).unwrap();

				Statement::Return(expr)
			},

			Token::Let => {
				self.pos += 1;

				let name = self.consume_ident().unwrap();
				self.consume(Token::Colon).unwrap();
				
				let mut type_ = self.consume_ident().unwrap();
				// TODO: this is a bit of a hack, need to properly parse types
				while *self.at() == Token::Star {
					type_.push('*');
					self.pos += 1;
				}

				self.consume(Token::Equals).unwrap();

				let val = self.parse_expr();
				self.consume(Token::Semicolon).unwrap();

				Statement::Let(name, type_, val)
			},

			Token::If => {
				self.pos += 1;

				self.consume(Token::ParenOpen).unwrap();
				let cond = self.parse_expr();
				self.consume(Token::ParenClose).unwrap();

				let body = self.parse_statement();

				Statement::If(cond, Box::new(body))
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

macro_rules! parse_expr_pn {
	($name:ident, $higher_name:ident, $( $pattern:pat_param )|+) => {
		fn $name(&mut self) -> Expression {
			let mut left = self.$higher_name();

			while matches!(self.at(), $($pattern)|+) {
				let op = Operator::to_binary_op(self.at()).expect(&format!("Could not convert {:?} into a binary operator", self.at()));
				self.pos += 1;

				let right = self.$higher_name();
				left = Expression::BinaryOperator { op, left: Box::new(left), right: Box::new(right) };
			}

			return left;
		}
	};
}

impl<'a> Parser<'a> {
	// FIXME: this is supposed to be rtl
	parse_expr_pn!(parse_expr_p0, parse_expr_p1, Token::Equals); // , +=, ...

	parse_expr_pn!(parse_expr_p1, parse_expr_p2, Token::FIXME_DELETE(_)); // ||
	parse_expr_pn!(parse_expr_p2, parse_expr_p3, Token::FIXME_DELETE(_)); // &&
	parse_expr_pn!(parse_expr_p3, parse_expr_p4, Token::FIXME_DELETE(_)); // |
	parse_expr_pn!(parse_expr_p4, parse_expr_p5, Token::FIXME_DELETE(_)); // ^
	parse_expr_pn!(parse_expr_p5, parse_expr_p6, Token::FIXME_DELETE(_)); // &

	parse_expr_pn!(parse_expr_p6, parse_expr_p7, Token::EqualsEquals); // , !=

	parse_expr_pn!(parse_expr_p7, parse_expr_p8, Token::AngleBracketOpen | Token::AngleBracketClose); // <= >=
	parse_expr_pn!(parse_expr_p8, parse_expr_p9, Token::FIXME_DELETE(_)); // << >>

	parse_expr_pn!(parse_expr_p9, parse_expr_p10, Token::Plus); // , -

	parse_expr_pn!(parse_expr_p10, parse_unary_rtl, Token::Star);

	fn parse_unary_rtl(&mut self) -> Expression {
		if matches!(self.at(), Token::Star | Token::Plus) {
			let op = Operator::to_unary_op(self.at()).unwrap();
			self.pos += 1;

			// AA: should this be a parse_unary_rtl or parse_expr?
			return Expression::UnaryOperator { op, operand: Box::new(self.parse_unary_rtl()) };
		}

		self.parse_primary_expr()
	}

	fn parse_primary_expr(&mut self) -> Expression {
		match self.tokens[self.pos] {
			Token::NumberLiteral(n) => { self.pos += 1; Expression::NumberLiteral(n) },
			Token::StringLiteral(ref s) => { self.pos += 1; Expression::StringLiteral(s.clone()) },

			Token::ParenOpen => {
				self.pos += 1;
				let expr = self.parse_expr();
				self.consume(Token::ParenClose).unwrap();

				expr
			},

			// TODO: this needs to be an actual operator
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
}

// fn is_ident(token: &Token) -> bool { matches!(token, Token::Ident(_)) }
// fn expect_ident(token: &Token) -> Option<&String> { match token { Token::Ident(s) => Some(s), _ => None } }

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
	Struct(Vec<(String, String)>),
	Expression(Expression),
}

#[derive(Debug)]
pub enum Operator {
	Assign,
	Add,
	Subtract,
	Multiply,
	Deref,
	IsEqual,
	UnaryPlus,
	IsLessThan,
	IsGreaterThan,
	MemberAccess,
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
	Let(String, String, Option<Expression>),
	// TODO: make if and while an expression
	If(Expression, Box<Statement>),
	While(Expression, Box<Statement>),

	// TODO: make this an expr
	Compound(Vec<Statement>),
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
			Token::Hyphen => Some(Self::Subtract),
			Token::Star => Some(Self::Multiply),
			Token::EqualsEquals => Some(Self::IsEqual),
			Token::AngleBracketOpen => Some(Self::IsLessThan),
			Token::AngleBracketClose => Some(Self::IsGreaterThan),
			Token::Period => Some(Self::MemberAccess),
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

macro_rules! consume_unwrap {
	($self:ident, $token:expr) => { $self.consume(&$token).expect(&format!("Expected {:?} but found {:?}", $token, $self.at())) };
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
	fn consume(&mut self, token: &Token) -> Option<()> {
		if std::mem::discriminant(self.at()) == std::mem::discriminant(token) {
			self.pos += 1;
			Some(())
		} else {
			None
		}
	}

	// fn consume_unwrap(&mut self, token: Token) {
	// 	self.consume(&token).expect(&format!("Expected {token:?} but found {:?}", self.at()))
	// }

	fn parse_const_assignment(&mut self) -> ConstAssignment {
		let ident = self.consume_ident().unwrap();

		consume_unwrap!(self, Token::ColonColon);

		let val = match self.at() {
			Token::Fn => {
				self.pos += 1;

				consume_unwrap!(self, Token::ParenOpen);

				let mut args = Vec::new();
				while self.pos < self.tokens.len() {
					if *self.at() == Token::ParenClose { break }

					let name = self.consume_ident().unwrap();

					consume_unwrap!(self, Token::Colon);

					let type_ = self.parse_type();

					args.push((name.clone(), type_.clone()));
					if *self.at() == Token::ParenClose { break }

					self.consume(&Token::Comma).expect("Missing comma between function parameters");
				}

				consume_unwrap!(self, Token::ParenClose);

				let return_type = if *self.at() == Token::Arrow {
					self.pos += 1;

					let return_type = self.parse_type();
					Some(return_type)
				} else {
					None
				};

				consume_unwrap!(self, Token::BraceOpen);

				// AA: mayhaps use parse_statement?
				let mut body = Vec::new();
				while *self.at() != Token::BraceClose {
					body.push(self.parse_statement());
				}

				consume_unwrap!(self, Token::BraceClose);

				ConstAssignmentVal::Function { args, return_type, body }
			},

			Token::Struct => {
				self.pos += 1;
				consume_unwrap!(self, Token::BraceOpen);

				let mut members = Vec::new();
				while self.pos < self.tokens.len() {
					if *self.at() == Token::BraceClose { break }

					let name = self.consume_ident().unwrap();

					consume_unwrap!(self, Token::Colon);

					let type_ = self.parse_type();

					members.push((name.clone(), type_.clone()));

					if *self.at() == Token::BraceClose { break }
					self.consume(&Token::Comma).expect("Missing comma between struct members");
				}

				consume_unwrap!(self, Token::BraceClose);

				ConstAssignmentVal::Struct(members)
			},

			_ => ConstAssignmentVal::Expression(self.parse_expr()),
		};

		self.consume(&Token::Semicolon).expect("Missing ; after constant assignment");

		ConstAssignment(ident, val)
	}

	// FIXME: this is a bit of a hack, need to properly parse types
	fn parse_type(&mut self) -> String {
		let mut type_ = self.consume_ident().unwrap();
		while *self.at() == Token::Star {
			type_.push('*');
			self.pos += 1;
		}

		type_
	}

	fn parse_expr(&mut self) -> Expression {
		self.parse_expr_p0()
	}

	fn parse_statement(&mut self) -> Statement {
		match self.at() {
			Token::Return => {
				self.pos += 1;

				let expr = self.parse_expr();
				consume_unwrap!(self, Token::Semicolon);

				Statement::Return(expr)
			},

			Token::Let => {
				self.pos += 1;

				let name = self.consume_ident().unwrap();
				self.consume(&Token::Colon).expect("No explicit type hint, type inference isn't implemented (yet)");
				
				let type_ = self.parse_type();

				let val = if *self.at() == Token::Equals {
					self.pos += 1;
					Some(self.parse_expr())
				} else {
					None
				};
				
				consume_unwrap!(self, Token::Semicolon);

				Statement::Let(name, type_, val)
			},

			Token::If => {
				self.pos += 1;

				consume_unwrap!(self, Token::ParenOpen);
				let cond = self.parse_expr();
				consume_unwrap!(self, Token::ParenClose);

				let body = self.parse_statement();

				Statement::If(cond, Box::new(body))
			},

			Token::While => {
				self.pos += 1;

				consume_unwrap!(self, Token::ParenOpen);
				let cond = self.parse_expr();
				consume_unwrap!(self, Token::ParenClose);

				let body = self.parse_statement();

				Statement::While(cond, Box::new(body))
			},

			Token::BraceOpen => {
				self.pos += 1;

				let mut body = Vec::new();
				while *self.at() != Token::BraceClose {
					body.push(self.parse_statement());
				}

				consume_unwrap!(self, Token::BraceClose);

				Statement::Compound(body)
			},

			_ => {
				let expr = self.parse_expr();
				consume_unwrap!(self, Token::Semicolon);

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

	parse_expr_pn!(parse_expr_p9, parse_expr_p10, Token::Plus | Token::Hyphen);
	parse_expr_pn!(parse_expr_p10, parse_unary_rtl, Token::Star);

	parse_expr_pn!(parse_expr_p11, parse_primary_expr, Token::Period);

	fn parse_unary_rtl(&mut self) -> Expression {
		if matches!(self.at(), Token::Star | Token::Plus) {
			let op = Operator::to_unary_op(self.at()).expect(&format!("Could not convert {:?} into a unary operator", self.at()));
			self.pos += 1;

			// AA: should this be a parse_unary_rtl or parse_expr?
			return Expression::UnaryOperator { op, operand: Box::new(self.parse_unary_rtl()) };
		}

		self.parse_expr_p11()
	}

	fn parse_primary_expr(&mut self) -> Expression {
		match self.tokens[self.pos] {
			Token::NumberLiteral(n) => { self.pos += 1; Expression::NumberLiteral(n) },
			Token::StringLiteral(ref s) => { self.pos += 1; Expression::StringLiteral(s.clone()) },

			Token::ParenOpen => {
				self.pos += 1;
				let expr = self.parse_expr();
				consume_unwrap!(self, Token::ParenClose);

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

				consume_unwrap!(self, Token::ParenClose);

				Expression::FunctionCall(ident.clone(), args)
			},

			Token::Ident(ref ident) => { self.pos += 1; Expression::Ident(ident.clone()) },

			ref t => panic!("Unexpected token while parsing expression: {t:?}"),
		}
	}
}

// fn is_ident(token: &Token) -> bool { matches!(token, Token::Ident(_)) }
// fn expect_ident(token: &Token) -> Option<&String> { match token { Token::Ident(s) => Some(s), _ => None } }

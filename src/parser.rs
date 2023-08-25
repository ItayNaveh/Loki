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
	Number(i64),
	Ident(String),

	// FIXME: op shouldn't be a Token
	BinaryOperator { op: Token, left: Box<Expression>, right: Box<Expression> },

	FunctionCall(String, Vec<Expression>),
}

#[derive(Debug)]
pub enum Statement {
	Return(Expression),
}

pub fn parse(tokens: Vec<Token>) -> AstRoot {
	let mut pos = 0;
	let mut root = Vec::new();

	while pos < tokens.len() {
		root.push(parse_const_assignment(&tokens, &mut pos));
	}

	return AstRoot(root);
}

fn parse_const_assignment(tokens: &[Token], pos: &mut usize) -> ConstAssignment {
	assert!(is_ident(&tokens[*pos]));
	let ident = expect_ident(&tokens[*pos]).unwrap();
	*pos += 1;
	
	assert_eq!(tokens[*pos], Token::ColonColon);
	*pos += 1;
	
	let val = match tokens[*pos] {
		Token::Fn => {
			*pos += 1;
			assert_eq!(tokens[*pos], Token::ParenOpen);
			*pos += 1;
			
			let mut args = Vec::new();
			while *pos < tokens.len() {
				if tokens[*pos] == Token::ParenClose { break }

				assert!(is_ident(&tokens[*pos]));
				let name = expect_ident(&tokens[*pos]).unwrap();
				*pos += 1;
				
				assert_eq!(tokens[*pos], Token::Colon);
				*pos += 1;
				
				assert!(is_ident(&tokens[*pos]));
				let type_ = expect_ident(&tokens[*pos]).unwrap();
				*pos += 1;
				
				args.push((name.clone(), type_.clone()));
				if tokens[*pos] == Token::ParenClose { break }

				assert_eq!(tokens[*pos], Token::Comma);
				*pos += 1;
			}
			
			assert_eq!(tokens[*pos], Token::ParenClose);
			*pos += 1;
			
			let return_type = if tokens[*pos] == Token::Arrow {
				*pos += 1;
				assert!(is_ident(&tokens[*pos]));
				let return_type = expect_ident(&tokens[*pos]).unwrap();
				*pos += 1;

				Some(return_type.clone())
			} else {
				None
			};

			assert_eq!(tokens[*pos], Token::BraceOpen);
			*pos += 1;

			let mut body = Vec::new();
			while *pos < tokens.len() && tokens[*pos] != Token::BraceClose {
				body.push(parse_statement(tokens, pos));
			}

			assert_eq!(tokens[*pos], Token::BraceClose);
			*pos += 1;

			ConstAssignmentVal::Function { args, return_type, body }
		},
		_ => ConstAssignmentVal::Expression(parse_expr(tokens, pos)),
	};

	assert_eq!(tokens[*pos], Token::Semicolon);
	*pos += 1;

	ConstAssignment(ident.clone(), val)
}

fn parse_expr(tokens: &[Token], pos: &mut usize) -> Expression {
	parse_additive(tokens, pos)
}

fn parse_multiplicative(tokens: &[Token], pos: &mut usize) -> Expression {
	let mut left = parse_primary_expr(tokens, pos);
	
	while *pos < tokens.len() && matches!(tokens[*pos], Token::Star) {
		let op = tokens[*pos].clone();
		*pos += 1;
	
		let right = parse_primary_expr(tokens, pos);
		left = Expression::BinaryOperator { op, left: Box::new(left), right: Box::new(right) };
	}
	
	left
}

fn parse_additive(tokens: &[Token], pos: &mut usize) -> Expression {
	let mut left = parse_multiplicative(tokens, pos);

	while *pos < tokens.len() && matches!(tokens[*pos], Token::Plus) {
		let op = tokens[*pos].clone();
		*pos += 1;

		let right = parse_multiplicative(tokens, pos);
		left = Expression::BinaryOperator { op, left: Box::new(left), right: Box::new(right) };
	}

	left
}

fn parse_primary_expr(tokens: &[Token], pos: &mut usize) -> Expression {
	match tokens[*pos] {
		Token::Number(n) => { *pos += 1; Expression::Number(n) },

		Token::Ident(ref ident) if tokens[*pos + 1] == Token::ParenOpen => {
			*pos += 2; // ident + (
			let mut args = Vec::new();
			while *pos < tokens.len() && tokens[*pos] != Token::ParenClose {
				args.push(parse_expr(tokens, pos));

				if tokens[*pos] != Token::Comma { break }
				*pos += 1;
			}

			assert_eq!(tokens[*pos], Token::ParenClose);
			*pos += 1;

			Expression::FunctionCall(ident.clone(), args)
		},

		Token::Ident(ref ident) => { *pos += 1; Expression::Ident(ident.clone()) },

		ref t => panic!("Unexpected token while parsing expression: {t:?}"),
	}
}

fn parse_statement(tokens: &[Token], pos: &mut usize) -> Statement {
	match tokens[*pos] {
		Token::Return => {
			*pos += 1;
			let expr = parse_expr(tokens, pos);
			
			assert_eq!(tokens[*pos], Token::Semicolon);
			*pos += 1;

			Statement::Return(expr)
		},

		ref t => panic!("Unexpected token while parsing statement: {t:?}"),
	}
}

fn is_ident(token: &Token) -> bool { matches!(token, Token::Ident(_)) }
fn expect_ident(token: &Token) -> Option<&String> { match token { Token::Ident(s) => Some(s), _ => None } }

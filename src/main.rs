#![allow(clippy::needless_return)]

mod lexer;
mod parser;

use std::fmt::Write;
use parser::{ Statement, Expression, ConstAssignmentVal };

macro_rules! println_if {
	($cond:expr, $($arg:tt)*) => { if $cond { println!($($arg)*) } };
}

fn main() {
	let mut filename = None;
	let mut output_filename = None;

	let mut running_test = false;

	if let Ok(var_running_tests) = std::env::var("LOKI_RUNNING_TESTS") {
		if var_running_tests == "yes" {
			running_test = true;
			
			if let Ok(file) = std::env::var("LOKI_FILE") {
				filename = Some(file);
			}

			if let Ok(file) = std::env::var("LOKI_OUTPUT_FILE") {
				output_filename = Some(file);
			}
		}
	}

	let filename = filename.unwrap_or_else(|| "stuff.loki".to_string());
	let output_filename = output_filename.unwrap_or_else(|| filename.clone() + ".c");

	let input = std::fs::read_to_string(&filename).expect(&("Failed to open file ".to_string() + &filename));
	let tokens = lexer::lex(&input);
	// println_if!(!running_test, "{tokens:#?}");

	let ast = parser::parse(tokens);
	// println_if!(!running_test, "{ast:#?}");

	let mut program = "".to_string();
	for const_assignment in ast.0 {
		match const_assignment.1 {
			ConstAssignmentVal::Function { args, return_type, body } => {
				let args = args.into_iter().map(|(name, type_)| type_ + " " + &name).collect::<Vec<String>>().join(",");
				let body = body.into_iter().map(serialize_statement).collect::<String>();
				write!(program,
					"{ret} {name} ({args}) {{ {body} }}",
					ret = return_type.unwrap_or_else(|| "void".to_string()),
					name = const_assignment.0,
					args = args,
					body = body,
				).unwrap();
			},

			ConstAssignmentVal::Expression(expr) if matches!(expr, Expression::NumberLiteral(_)) && const_assignment.0.starts_with("__t_") => {
				if let Expression::NumberLiteral(n) = expr {
					println_if!(running_test, "{}={}", const_assignment.0, n);
				} else {
					panic!();
				}
			},

			e => unimplemented!("{e:?}"),
		}
	}

	std::fs::write(output_filename, program).unwrap();
}

fn serialize_statement(statement: Statement) -> String {
	match statement {
		Statement::Return(expr) => format!("return {};", serialize_expression(expr)),
		Statement::Expression(expr) => serialize_expression(expr) + ";",
	}
}

fn serialize_expression(expr: Expression) -> String {
	match expr {
		Expression::NumberLiteral(n) => n.to_string(),
		Expression::StringLiteral(s) => '"'.to_string() + &s + "\"",
		Expression::Ident(ident) => ident,
		Expression::BinaryOperator { op, left, right } => format!("({}) {} ({})", serialize_expression(*left), serialize_token(op), serialize_expression(*right)),
		Expression::FunctionCall(name, args) => {
			let args = args.into_iter().map(serialize_expression).collect::<Vec<String>>().join(",");
			format!("{name}({})", args)
		},
	}
}

// FIXME: this souldn't need to exist
fn serialize_token(token: lexer::Token) -> String {
	match token {
		lexer::Token::Plus => "+".to_string(),
		lexer::Token::Star => "*".to_string(),
		_ => panic!(),
	}
}

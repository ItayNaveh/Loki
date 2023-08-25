#![allow(clippy::needless_return)]

mod lexer;
mod parser;

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

	let filename = filename.unwrap_or("start.loki".to_string());
	let output_filename = output_filename.unwrap_or_else(|| filename.clone() + ".c");

	let input = std::fs::read_to_string(&filename).unwrap();
	let tokens = lexer::lex(&input);
	println_if!(!running_test, "{tokens:#?}");

	let ast = parser::parse(tokens);
	println_if!(!running_test, "{ast:#?}");

	let mut program = "".to_string();
	for const_assignment in ast.0 {
		match const_assignment.1 {
			ConstAssignmentVal::Function { return_type, body } => {
				let body = body.into_iter().map(serialize_statement).collect::<String>();

				program += &format!(
					"{ret} {name} () {{ {body} }}",
					ret = return_type.unwrap_or("void".to_string()),
					name = const_assignment.0,
					body = body,
				);
			},

			ConstAssignmentVal::Expression(expr) if matches!(expr, Expression::Number(_)) && const_assignment.0.starts_with("__t_") => {
				if let Expression::Number(n) = expr {
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
	}
}

fn serialize_expression(expr: Expression) -> String {
	match expr {
		Expression::Number(n) => n.to_string(),
		Expression::BinaryOperator { op, left, right } => format!("({}) {} ({})", serialize_expression(*left), serialize_token(op), serialize_expression(*right)),
		_ => unimplemented!()
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

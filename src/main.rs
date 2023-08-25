#![allow(clippy::needless_return)]

mod lexer;
mod parser;

use parser::{ Statement, Expression, ConstAssignmentVal};

fn main() {
	let input = std::fs::read_to_string("start.loki").unwrap();
	let tokens = lexer::lex(&input);
	println!("{tokens:#?}");

	let ast = parser::parse(tokens);
	println!("{ast:#?}");

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

			e => unimplemented!("{e:?}"),
		}
	}

	std::fs::write("out.c", program).unwrap();
}

fn serialize_statement(statement: Statement) -> String {
	match statement {
		Statement::Return(expr) => format!("return {};", serialize_expression(expr)),
	}
}

fn serialize_expression(expr: Expression) -> String {
	match expr {
		Expression::Number(n) => n.to_string(),
		_ => unimplemented!()
	}
}

#![allow(clippy::needless_return)]

mod lexer;
mod parser;

use std::fmt::Write;
use parser::{ Statement, Expression, ConstAssignmentVal, Operator };

fn main() {
	let mut args = std::env::args().into_iter().skip(1);

	let mut input_file = None;
	let mut output_file = None;

	#[derive(Debug)] enum Emit {
		C, BinClang,
	}
	let mut emit = None;

	while let Some(arg) = args.next() {
		match arg.as_str() {
			"--emit" => {
				if emit.is_some() {
					panic!("Already specified emit")
				}

				let to = args.next().unwrap();
				emit = Some(match to.as_str() {
					"c" => Emit::C,
					"bin-clang" => Emit::BinClang,
					t => panic!("Unknown emit type {t}"),
				});
			},

			"-o" => {
				if output_file.is_some() {
					panic!("Already specified output file")
				}

				let out = args.next().unwrap();
				output_file = Some(out);
			},

			f if !f.starts_with("-") && input_file.is_none() => input_file = Some(arg),
			_ => panic!("Unknown argument {arg}"),
		}
	}

	let input_file = input_file.unwrap();
	let emit = emit.unwrap_or(Emit::BinClang);
	let output_file = output_file.unwrap_or_else(|| {
		std::path::Path::new(&input_file).file_stem().unwrap().to_str().unwrap().to_string() + match emit {
			Emit::C => ".c",
			Emit::BinClang => ".exe",
		}
	});

	// println!("Compiling {input_file} to {output_file} as {emit:?}");
	let compiler_output = compile(&input_file);
	match emit {
		Emit::C => std::fs::write(output_file, compiler_output).unwrap(),

		Emit::BinClang => {
			use std::process::*;
			use std::io::Write;

			let mut clang = Command::new("clang")
				.arg("-x").arg("c")
				.arg("-")
				.arg("-o").arg(output_file)
				.args(["-Wall", "-Wextra", "-pedantic"])
				.stdin(Stdio::piped()).spawn().unwrap();
			let mut stdin = clang.stdin.take().unwrap();
			std::thread::spawn(move || {
				stdin.write_all(compiler_output.as_bytes()).unwrap();
			});

			if !clang.wait().unwrap().success() {
				panic!("Clang errored");
			}
		},
	}
}

fn compile(input_file: &str) -> String {
	let mut running_test = false;
	if let Ok(var_running_tests) = std::env::var("LOKI_RUNNING_TESTS") {
		running_test = var_running_tests == "yes";
	}

	let input = std::fs::read_to_string(input_file).expect(&("Failed to open file ".to_string() + input_file));
	let tokens = lexer::lex(&input);
	// println!("{tokens:#?}");

	let ast = parser::parse(tokens);
	// println!("{ast:#?}");

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
					if running_test { println!("{}={}", const_assignment.0, n) }
				} else {
					panic!();
				}
			},

			e => unimplemented!("{e:?}"),
		}
	}

	program.write_char('\n').unwrap();

	return program;
}

fn serialize_statement(statement: Statement) -> String {
	match statement {
		Statement::Return(expr) => format!("return {};", serialize_expression(expr)),
		Statement::Let(name, type_, val) => format!("{type_} {name} = {};", serialize_expression(val)),
		Statement::Expression(expr) => serialize_expression(expr) + ";",
	}
}

fn serialize_expression(expr: Expression) -> String {
	match expr {
		Expression::NumberLiteral(n) => n.to_string(),
		Expression::StringLiteral(s) => '"'.to_string() + &s + "\"",
		Expression::Ident(ident) => ident,

		Expression::BinaryOperator { op, left, right } => format!("({} {} {})", serialize_expression(*left), serialize_operator(op), serialize_expression(*right)),
		Expression::UnaryOperator { op, operand } => format!("({}{})", serialize_operator(op), serialize_expression(*operand)),

		Expression::FunctionCall(name, args) => {
			let args = args.into_iter().map(serialize_expression).collect::<Vec<String>>().join(",");
			format!("{name}({})", args)
		},
	}
}

fn serialize_operator(op: Operator) -> String {
	match op {
		Operator::Equals => '='.to_string(),
		Operator::Plus => '+'.to_string(),
		Operator::Multiply | Operator::Deref => '*'.to_string(),
	}
}

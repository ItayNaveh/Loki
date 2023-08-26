// FIXME: instead of String use like a unique string table (FlyString? in serenity)
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
	Fn,
	Return,
	Let,

	// FIXME: maybe it should be 2 Colon tokens
	// https://odin-lang.org/docs/faq/#what-does--mean-1
	ColonColon,
	Arrow,
	Colon,
	Semicolon,
	Comma,

	Plus,
	Star, // FIXME: maybe asterisk?
	Equals,

	ParenOpen,
	ParenClose,
	BraceOpen,
	BraceClose,

	Ident(String),
	NumberLiteral(i64),
	StringLiteral(String),
}

impl Token {
	pub fn ident(&self) -> Option<String> {
		match self {
			Token::Ident(ident) => Some(ident.clone()),
			_ => None,
		}
	}
}

pub fn lex(input: &str) -> Vec<Token> {
	let mut pos = 0;
	let mut tokens = Vec::new();
	// FIXME: maybe it can be an iterator
	let input: Vec<char> = input.chars().collect();

	while pos < input.len() {
		match input[pos] {
			' ' | '\t' | '\n' => pos += 1,
			'/' if input[pos + 1] == '/' => {
				while pos < input.len() && input[pos] != '\n' { pos += 1 }
				pos += 1;
			},

			':' if input[pos + 1] == ':' => {
				tokens.push(Token::ColonColon);
				pos += 2;
			},

			'-' if input[pos + 1] == '>' => {
				tokens.push(Token::Arrow);
				pos += 2;
			},

			':' => { tokens.push(Token::Colon); pos += 1 },
			';' => { tokens.push(Token::Semicolon); pos += 1 },
			',' => { tokens.push(Token::Comma); pos += 1 },
			
			'+' => { tokens.push(Token::Plus); pos += 1 },
			'*' => { tokens.push(Token::Star); pos += 1 },
			'=' => { tokens.push(Token::Equals); pos += 1 },

			'(' => { tokens.push(Token::ParenOpen); pos += 1 },
			')' => { tokens.push(Token::ParenClose); pos += 1 },
			'{' => { tokens.push(Token::BraceOpen); pos += 1 },
			'}' => { tokens.push(Token::BraceClose); pos += 1 },

			// TODO: handle things like \n, \"
			'"' => {
				pos += 1;
				let start = pos;
				while pos < input.len() && input[pos] != '"' { pos += 1 }
				assert_eq!(input[pos], '"');
				let str = String::from_iter(&input[start..pos]);
				pos += 1;
				tokens.push(Token::StringLiteral(str));
			},

			c if is_ident_start(c) => {
				let start = pos;
				pos += 1;
				while pos < input.len() && is_ident_anywhere(input[pos]) { pos += 1 }
				let ident = String::from_iter(&input[start..pos]);
				tokens.push(match ident.as_str() {
					"fn" => Token::Fn,
					"return" => Token::Return,
					"let" => Token::Let,
					_ => Token::Ident(ident),
				});
			},

			// FIXME: handle different bases (and also like U / L suffixes)
			'0'..='9' => {
				let start = pos;
				pos += 1;
				while pos < input.len() && matches!(input[pos], '0'..='9') { pos += 1 }
				tokens.push(Token::NumberLiteral(String::from_iter(&input[start..pos]).parse().unwrap()));
			},

			c => unimplemented!("Unexpected character: {c}"),
		}
	}

	return tokens;
}

#[inline(always)]
const fn is_ident_start(c: char) -> bool { matches!(c, 'a'..='z' | 'A'..='Z' | '_') }

#[inline(always)]
const fn is_ident_anywhere(c: char) -> bool { is_ident_start(c) || matches!(c, '0'..='9') }

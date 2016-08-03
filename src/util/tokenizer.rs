#[derive(Debug, Copy, Clone)]
pub struct CharIndex
{
	pub linear: usize,
	pub line: usize,
	pub column: usize
}


#[derive(Debug, Copy, Clone)]
pub struct Span
{
	pub start: CharIndex,
	pub end: CharIndex
}


#[derive(Debug, Clone)]
pub struct Token
{
	pub span: Span,
	pub kind: TokenKind
}


#[derive(Debug, Clone)]
pub enum TokenKind
{
	Error(char),
	LineBreak,
	Identifier(String),
	Number(usize, String),
	String(String),
	Operator(&'static str),
	End
}


pub fn tokenize(src: &[char]) -> Vec<Token>
{
	let mut tokens = Vec::new();
	let mut index = CharIndex::new();
	let mut last_was_linebreak = true;
	
	while index.linear < src.len()
	{
		if is_whitespace(src[index.linear])
		{
			index.advance();
			continue;
		}
		else if last_was_linebreak && src[index.linear] == '\n'
		{
			index.advance_line();
			continue;
		}
	
		let token =
			try_read_identifier(src, &mut index).unwrap_or_else(||
			try_read_integer(src, &mut index).unwrap_or_else(||
			try_read_string(src, &mut index).unwrap_or_else(||
			try_read_linebreak(src, &mut index).unwrap_or_else(||
			try_read_operator(src, &mut index).unwrap_or_else(||
			read_error(src, &mut index))))));
		
		last_was_linebreak = match token.kind
		{
			TokenKind::LineBreak => true,
			_ => false
		};
		
		tokens.push(token);
	}
	
	tokens
}


impl CharIndex
{
	pub fn new() -> CharIndex
	{
		CharIndex
		{
			linear: 0,
			line: 1,
			column: 1
		}
	}
	
	
	pub fn advance(&mut self)
	{
		self.linear += 1;
		self.column += 1;
	}
	
	
	pub fn advance_by(&mut self, columns: usize)
	{
		self.linear += columns;
		self.column += columns;
	}
	
	
	pub fn advance_line(&mut self)
	{
		self.linear += 1;
		self.line += 1;
		self.column = 1;
	}
}


impl Token
{
	pub fn is_linebreak(&self) -> bool
	{
		match self.kind
		{
			TokenKind::LineBreak => true,
			_ => false
		}
	}
	
	
	pub fn is_linebreak_or_end(&self) -> bool
	{
		match self.kind
		{
			TokenKind::End => true,
			TokenKind::LineBreak => true,
			_ => false
		}
	}
	
	
	pub fn is_identifier(&self) -> bool
	{
		match self.kind
		{
			TokenKind::Identifier(..) => true,
			_ => false
		}
	}
	
	
	pub fn identifier(&self) -> &String
	{
		match self.kind
		{
			TokenKind::Identifier(ref ident) => &ident,
			_ => panic!("not an identifier")
		}
	}
	
	
	pub fn is_number(&self) -> bool
	{
		match self.kind
		{
			TokenKind::Number(..) => true,
			_ => false
		}
	}
	
	
	pub fn number(&self) -> (usize, &String)
	{
		match self.kind
		{
			TokenKind::Number(radix, ref value) => (radix, &value),
			_ => panic!("not a number")
		}
	}
	
	
	pub fn number_usize(&self) -> usize
	{
		match self.kind
		{
			TokenKind::Number(radix, ref value) => usize::from_str_radix(&value, radix as _).unwrap(),
			_ => panic!("not a number")
		}
	}
	
	
	pub fn is_string(&self) -> bool
	{
		match self.kind
		{
			TokenKind::String(..) => true,
			_ => false
		}
	}
	
	
	pub fn string(&self) -> &String
	{
		match self.kind
		{
			TokenKind::String(ref s) => &s,
			_ => panic!("not a string")
		}
	}
	
	
	pub fn is_any_operator(&self) -> bool
	{
		match self.kind
		{
			TokenKind::Operator(..) => true,
			_ => false
		}
	}
	
	
	pub fn is_operator(&self, op: &str) -> bool
	{
		match self.kind
		{
			TokenKind::Operator(token_op) => token_op == op,
			_ => false
		}
	}
	
	
	pub fn operator(&self) -> &'static str
	{
		match self.kind
		{
			TokenKind::Operator(op) => op,
			_ => panic!("not an operator")
		}
	}
}


fn try_read_identifier(src: &[char], index: &mut CharIndex) -> Option<Token>
{
	let index_before = *index;
	
	if !is_identifier_start(src[index.linear])
		{ return None; }

	let mut identifier = String::new();
	while index.linear < src.len() && is_identifier_mid(src[index.linear])
	{
		identifier.push(src[index.linear]);
		index.advance();
	}
	
	Some(Token
	{
		span: Span { start: index_before, end: *index },
		kind: TokenKind::Identifier(identifier)
	})
}


fn try_read_integer(src: &[char], index: &mut CharIndex) -> Option<Token>
{
	let index_before = *index;
	
	if !src[index.linear].is_digit(10)
		{ return None; }

	let radix =
		if src[index.linear] == '0' && index.linear + 1 < src.len()
		{
			match src[index.linear + 1]
			{
				'b' => { index.advance_by(2); 2 }
				'x' => { index.advance_by(2); 16 }
				_ => 10
			}
		}
		else
			{ 10 };
	
	let mut digits = String::new();
	while index.linear < src.len()
	{
		if !src[index.linear].is_digit(radix) && src[index.linear] != '_'
			{ break; }
		
		digits.push(src[index.linear]);
		index.advance();
	}
	
	Some(Token
	{
		span: Span { start: index_before, end: *index },
		kind: TokenKind::Number(radix as usize, digits)
	})
}


fn try_read_string(src: &[char], index: &mut CharIndex) -> Option<Token>
{
	let index_before = *index;
	
	if src[index.linear] != '\"' // "
		{ return None; }
		
	index.advance();

	let mut s = String::new();
	while index.linear < src.len() && src[index.linear] != '\"' // "
	{
		s.push(src[index.linear]);
		
		if src[index.linear] == '\n'
			{ index.advance_line(); }
		else
			{ index.advance(); }
	}
	
	if src[index.linear] == '\"' // "
		{ index.advance(); }
	
	Some(Token
	{
		span: Span { start: index_before, end: *index },
		kind: TokenKind::String(s)
	})
}


fn try_read_operator(src: &[char], index: &mut CharIndex) -> Option<Token>
{
	let operators =
	[
		".", "->", ":", ";", ",",
		"(", ")", "[", "]", "{", "}",
		"'",
		"#", "$"
	];
	
	let maybe_match = operators.iter().find(|op|
	{
		for (i, c) in op.chars().enumerate()
		{
			if index.linear + i >= src.len() || src[index.linear + i] != c
				{ return false; }
		}
		true
	});
	
	match maybe_match
	{
		Some(s) =>
		{
			let index_before = *index;
			index.advance_by(s.chars().count());
			Some(Token
			{
				span: Span { start: index_before, end: *index },
				kind: TokenKind::Operator(s)
			})
		}
		None => None
	}
}


fn try_read_linebreak(src: &[char], index: &mut CharIndex) -> Option<Token>
{
	let index_before = *index;
	
	if src[index.linear] != '\n'
		{ return None; }
	
	index.advance_line();
	Some(Token
	{
		span: Span { start: index_before, end: *index },
		kind: TokenKind::LineBreak
	})
}


fn read_error(src: &[char], index: &mut CharIndex) -> Token
{
	let index_before = *index;
	
	index.advance();
	Token
	{
		span: Span { start: index_before, end: *index },
		kind: TokenKind::Error(src[index.linear - 1])
	}
}


fn is_whitespace(c: char) -> bool
{
	c == ' ' ||
	c == '\t' ||
	c == '\r'
}


fn is_identifier_start(c: char) -> bool
{
	(c >= 'a' && c <= 'z') ||
	(c >= 'A' && c <= 'Z') ||
	c == '_'
}


fn is_identifier_mid(c: char) -> bool
{
	(c >= 'a' && c <= 'z') ||
	(c >= 'A' && c <= 'Z') ||
	(c >= '0' && c <= '9') ||
	c == '_'
}
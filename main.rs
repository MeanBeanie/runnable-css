use std::fs::File;
use std::io::Read;
use std::env;

#[derive(PartialEq, Clone)]
enum TokenType {
	Keyword,
	Number,
	Newline,
}

#[derive(Clone)]
struct Token {
	t: TokenType,
	string: String,
	value: i32
}

#[derive(PartialEq, Clone)]
enum Expr {
	SelectVar(i32, i32), // which var to select
	SetVar(i32), // value to set selected var to
	// NOTE totally ignoring the loop type for now
	Loop(i32, char, i32), // stores the start, type, end
	LoopEnd,
	Math(i32, char, i32), // stores lhs, op, rhs
	AddToPrint(i32), // Which var to add
	AddCharToPrint(i32),
	AddVarCharPrint(i32),
	Print(i32), // the amount of vars to print
}

#[derive(Clone)]
enum Var {
	Integer(i32),
	Str(String)
}

impl Var {
	fn as_int(self) -> Option<i32> {
		match self {
			Self::Integer(a) => Some(a),
			_ => None
		}
	}

	fn as_string(self) -> Option<String> {
		match self {
			Self::Str(s) => Some(s),
			_ => None
		}
	}
}

fn get_number(num: Vec<u8>) -> i32 {
	if num[0] == b'#' {
		let mut res: i32 = 0;
		for i in 1..num.len() {
			let p16 = (num.len()-i-1) as u32;
			let mut mult: i32 = 16;
			mult = mult.pow(p16);
			match num[i] {
				b'0'..b'9' => {
					let val = (num[i] - b'0') as i32;
					res += val * mult;
				}
				b'a'..b'g' => {
					let mut val = (num[i] - b'a') as i32;
					val += 10;
					res += val * mult;
				}
				_ => {}
			}
		}
		return res;
	}

	let mut res: i32 = 0;
	for i in 0..num.len() {
		if num[i] == b'%' {
			break;
		}
		let p10 = (num.len()-i-1) as u32;
		let mut mult: i32 = 10;
		mult = mult.pow(p10);
		let val = (num[i] - b'0') as i32;
		res += val * mult;
	}

	return res;
}

fn main() -> std::io::Result<()> {
	let args: Vec<String> = env::args().collect();
	if args.len() < 2 {
		return Ok(());
	}

	let mut f = File::open(args[1].clone())?;
	let mut buffer = vec![];
	f.read_to_end(&mut buffer)?;

	let mut tokens: Vec<Token> = vec![];
	
	let mut size = 0;
	let mut start = 0;
	for byte in &buffer {
		let c: char = *byte as char;
		match c {
			' ' | '\n' | '\t' | ';' => {

				let mut tok: Token = Token {
					string: String::new(),
					t: TokenType::Newline,
					value: 0
				};

				let word: Vec<u8> = buffer[start..start+size].to_vec();
				let temp: String = String::from_utf8(word.clone()).unwrap();
				tok.string = temp;

				if word.len() != 0 && (word[0] == b'#' || (word[0] >= b'0' && word[0] <= b'9')) {
					tok.value = get_number(word);
					tok.t = TokenType::Number;
				}
				else{
					tok.t = TokenType::Keyword;
				}

				if !tok.string.is_empty() {
					tokens.push(tok.clone());
				}
				
				if c == '\n' {
					tok.t = TokenType::Newline;
					tok.string = String::from("\n");
					tok.value = 0;
					tokens.push(tok);
				}

				start += size;
				start += 1;
				size = 0;
			}
			_ => size += 1,
		}
	}

	println!("Found {} tokens", tokens.len());

	let mut exprs: Vec<Expr> = vec![];

	let mut args: Vec<Token> = vec![];
	let mut command: i32 = -1;

	for tok in &tokens {
		if tok.t == TokenType::Keyword {
			let bytes = tok.string.as_bytes();
			if bytes[bytes.len()-1] == b':' {
				match tok.string.as_str() {
					"background-size:" => command = 0, // NOTE deprecated
					"background-position:" => command = 1,
					"background-color:" => command = 2,
					"outline:" => command = 3,
					"border:" => command = 4,
					"padding-right:" => command = 5,
					"padding-left:" => command = 8,
					"padding-bottom:" => command = 9,
					"padding-top:" => command = 6,
					"overflow:" => command = 7,
					_ => {}
				}
			}
			else {
				args.push(tok.clone());
			}
		}
		else if tok.t == TokenType::Newline {
			match command {
				1 => exprs.push(Expr::SelectVar(args[0].value, args[1].value)),
				2 => exprs.push(Expr::SetVar(args[0].value)),
				3 => {
					let mut c: char = ' ';
					match args[1].string.as_str() {
						"solid" => c = '+', // step of +1
						_ => {}
					}
					exprs.push(Expr::Loop(args[0].value, c, args[2].value));
				}
				4 => {
					let mut c: char = ' ';
					match args[1].string.as_str() {
						"solid" => c = '+', // add with 2 vars
						"none" => c = '_', // sub with rhs as literal
						_ => {}
					}
					exprs.push(Expr::Math(args[0].value, c, args[2].value));
				}
				5 => exprs.push(Expr::AddToPrint(args[0].value)),
				6 => exprs.push(Expr::Print(args[0].value)),
				7 => exprs.push(Expr::LoopEnd),
				8 => exprs.push(Expr::AddCharToPrint(args[0].value)),
				9 => exprs.push(Expr::AddVarCharPrint(args[0].value)),
				_ => {}
			}
			command = -1;
			args.clear();
		}
		else {
			args.push(tok.clone());
		}
	}

	println!("Found {} expressions", exprs.len());

	println!("PROG START:");

	let mut vars: Vec<Var> = vec![];
	let mut selected_var: i32 = 0;
	let mut rewrite: i32 = 0;
	let mut loop_index: i32 = -1;
	let mut iterator: i32 = 0;
	let mut printing: Vec<i32> = vec![];
	let mut index: i32 = 0;

	while index < exprs.len() as i32 {
		match exprs[index as usize] {
			Expr::SelectVar(var_index, r) => {
				selected_var = var_index;
				rewrite = r;
				if selected_var as usize >= vars.len() {
					for _i in vars.len()..(selected_var+1) as usize {
						vars.push(Var::Integer(0));
					}
				}
			}
			Expr::SetVar(value) => {
				if rewrite == 0 {
					if vars[selected_var as usize].clone().as_int() != None {
						// have an integer
						vars[selected_var as usize] = Var::Integer(value);
					}
					else {
						// have a string
						let mut old: String = vars[selected_var as usize].clone().as_string().unwrap();
						old.push(value as u8 as char);
						vars[selected_var as usize] = Var::Str(old);
					}
				}
				else{
					match rewrite {
						1 => vars[selected_var as usize] = Var::Integer(value),
						2 => vars[selected_var as usize] = Var::Str(String::from(value as u8 as char)),
						_ => {}
					}
				}
				rewrite = 0;
			}
			Expr::Loop(start, t, end) => {
				if loop_index < 0 {
					loop_index = index;
					iterator = start;
				}
				else {
					match t {
						'+' => {
							iterator += 1;
							if iterator >= end {
								loop_index = -1;
							}
						}
						_ => loop_index = -1,
					}
				}
			}
			Expr::LoopEnd => {
				if loop_index > 0 {
					index = loop_index;
					index -= 1;
				}
			}
			Expr::Math(lhs, op, rhs) => {
				match op {
					'+' => {
						if vars[lhs as usize].clone().as_int() != None && vars[rhs as usize].clone().as_int() != None {
							vars[selected_var as usize] = Var::Integer(vars[lhs as usize].clone().as_int().unwrap() + vars[rhs as usize].clone().as_int().unwrap());
						}
					}
					'_' => {
						if vars[lhs as usize].clone().as_int() != None {
							vars[selected_var as usize] = Var::Integer(vars[lhs as usize].clone().as_int().unwrap() - rhs);
						}
					}
					_ => {}
				}
			}
			Expr::AddToPrint(var_index) => printing.push(var_index),
			Expr::AddCharToPrint(char_value) => printing.push(-char_value),
			Expr::AddVarCharPrint(var_index) => {
				if vars[var_index as usize].clone().as_int() != None {
					printing.push(-vars[var_index as usize].clone().as_int().unwrap());
				}
			}
			Expr::Print(count) => {
				let mut end = count as usize;
				if count == 0 || count > printing.len() as i32 {
					end = printing.len();
				}
				for i in 0..end {
					if printing[i] >= 0 {
						if vars[printing[i] as usize].clone().as_int() != None {
							print!("{} ", vars[printing[i] as usize].clone().as_int().unwrap());
						}
						else{
							print!("{} ", vars[printing[i] as usize].clone().as_string().unwrap());
						}
					}
					else {
						print!("{}", -printing[i] as u8 as char);
					}
				}
				println!("");
				printing.clear();
			}
		}
		index += 1;
	}
	
	Ok(())
}

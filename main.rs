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
	CreateVarSpace(i32), // how many vars to create
	SelectVar(i32), // which var to select
	SetVar(i32), // value to set selected var to
	// NOTE totally ignoring the loop type for now
	Loop(i32, i32), // stores the start and end value
	LoopEnd,
	Math(i32, char, i32), // stores lhs, op, rhs
	AddToPrint(i32), // Which var to add
	Print(i32), // the amount of vars to print
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
				b'a'..b'f' => {
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
					"background-size:" => command = 0,
					"background-position:" => command = 1,
					"background-color:" => command = 2,
					"outline:" => command = 3,
					"border:" => command = 4,
					"padding-right:" => command = 5,
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
				0 => exprs.push(Expr::CreateVarSpace(args[0].value)),
				1 => exprs.push(Expr::SelectVar(args[0].value)),
				2 => exprs.push(Expr::SetVar(args[0].value)),
				3 => exprs.push(Expr::Loop(args[0].value, args[2].value)),
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

	let mut vars: Vec<i32> = vec![];
	let mut selected_var: i32 = 0;
	let mut loop_index: i32 = -1;
	let mut iterator: i32 = 0;
	let mut printing: Vec<i32> = vec![];
	let mut index: i32 = 0;

	while index < exprs.len() as i32 {
		match exprs[index as usize] {
			Expr::CreateVarSpace(var_count) => {
				for _i in 0..var_count {
					vars.push(0);
				}
			}
			Expr::SelectVar(var_index) => selected_var = var_index,
			Expr::SetVar(value) => vars[selected_var as usize] = value,
			Expr::Loop(start, end) => {
				if loop_index < 0 {
					loop_index = index;
					iterator = start;
				}
				else {
					if iterator >= end {
						loop_index = -1;
					}
				}
			}
			Expr::LoopEnd => {
				iterator += 1;
				if loop_index > 0 {
					index = loop_index;
					index -= 1;
				}
			}
			Expr::Math(lhs, op, rhs) => {
				match op {
					'+' => vars[selected_var as usize] = vars[lhs as usize] + vars[rhs as usize],
					'_' => vars[selected_var as usize] = vars[lhs as usize] - rhs,
					_ => {}
				}
			}
			Expr::AddToPrint(var_index) => printing.push(var_index),
			Expr::Print(count) => {
				if count == 0 {
					for i in &printing {
						print!("{} ", vars[*i as usize]);
					}
					println!("");
				}
				else{
					for i in 0..count {
						print!("{} ", vars[i as usize]);
					}
					println!("");
				}
				printing.clear();
			}
		}
		index += 1;
	}
	
	Ok(())
}

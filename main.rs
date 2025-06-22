use std::fs::File;
use std::process::exit;
use std::io::{Read, Write};
use std::io;
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
	SelectVar(i32, i32), // which var to select and the rewrite mode
	SetVar(i32), // value to set selected var to
	Loop(i32, char, i32), // stores the start, type, end
	LoopEnd, // used to jump back to the start
	Math(i32, char, i32), // stores lhs, op, rhs
	AddToPrint(i32), // which var to add
	AddCharToPrint(i32), // which char to add
	AddVarCharPrint(i32), // which var to treat a char when added
	Print(i32), // the amount of vars to print
	Conditional(i32, i32, i32, i32),
	Label(i32),
	Exit,
	UserInput(bool, i32),
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

fn print_variable(var: Var) {
	if var.clone().as_int() != None {
		print!("{}", var.clone().as_int().unwrap());
	}
	else{
		print!("{}", var.clone().as_string().unwrap());
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
					"margin:" => command = 10,
					"opacity:" => command = 11,
					"word-wrap:" => command = 12,
					"transition:" => command = 13,
					_ => {}
				}
			}
			else {
				args.push(tok.clone());
			}
		}
		else if tok.t == TokenType::Newline {
			match command {
				1 => {
					if args.len() < 2 {
						eprintln!("Expected two args for a select var");
						exit(1);
					}
					exprs.push(Expr::SelectVar(args[0].value, args[1].value));
				}
				2 => {
					if args.len() < 1 {
						eprintln!("Expected an arg for a set var");
						exit(1);
					}
					exprs.push(Expr::SetVar(args[0].value));
				}
				3 => {
					if args.len() < 3 {
						eprintln!("Expected 3 args for loop");
						exit(1);
					}
					let mut c: char = ' ';
					match args[1].string.as_str() {
						"solid" => c = '+', // step of +1
						_ => {}
					}
					exprs.push(Expr::Loop(args[0].value, c, args[2].value));
				}
				4 => {
					if args.len() < 3 {
						eprintln!("Expected 3 args for math operation");
						exit(1);
					}
					let mut c: char = ' ';
					match args[1].string.as_str() {
						"none" => c = '_', // sub with rhs as literal
						"hidden" => c = '~', // sub with lhs as literal
						"dotted" => c = '-', // sub with 2 vars
						"dashed" => c = '=', // add with rhs as literal
						"solid" => c = '+', // add with 2 vars
						"double" => c = '8', // mult with rhs as literal
						"groove" => c = '*', // mult with 2 vars
						"ridge" => c = '\\', // divide with rhs as literal
						"inset" => c = '|', // divide with lhs as literal,
						"outset" => c = '/', // divide with 2 vars
						_ => {}
					}
					exprs.push(Expr::Math(args[0].value, c, args[2].value));
				}
				5 => {
					if args.len() < 1 {
						eprintln!("Expected 1 arg for add var print");
						exit(1);
					}
					exprs.push(Expr::AddToPrint(args[0].value));
				}
				6 => {
					if args.len() < 1 {
						eprintln!("Expected 1 arg for print");
						exit(1);
					}
					exprs.push(Expr::Print(args[0].value));
				}
				7 => exprs.push(Expr::LoopEnd),
				8 => {
					if args.len() < 1 {
						eprintln!("Expected 1 arg for add char print");
						exit(1);
					}
					exprs.push(Expr::AddCharToPrint(args[0].value));
				}
				9 => {
					if args.len() < 1 {
						eprintln!("Expected 1 arg for add var as char print");
						exit(1);
					}
					exprs.push(Expr::AddVarCharPrint(args[0].value));
				}
				10 => {
					if args.len() < 4 {
						eprintln!("Expected 4 args for conditional");
						exit(1);
					}
					exprs.push(Expr::Conditional(args[0].value, args[1].value, args[2].value, args[3].value));
				}
				11 => {
					if args.len() < 1 {
						eprintln!("Expected 1 arg for label");
						exit(1);
					}
					exprs.push(Expr::Label(args[0].value));
				}
				12 => exprs.push(Expr::Exit),
				13 => {
					if args.len() < 2 {
						eprintln!("Expected 2 args for user input");
						exit(1);
					}
					exprs.push(Expr::UserInput(args[0].string == "all", args[1].value));
				}
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
	let mut labels: Vec<i32> = vec![];
	let mut selected_var: i32 = 0;
	let mut rewrite: i32 = 0;
	let mut loop_index: i32 = -1;
	let mut iterator: i32 = 0;
	let mut printing: Vec<i32> = vec![];
	let mut index: i32 = 0;

	for expr in &exprs {
		match expr {
			Expr::Label(label_index) => {
				if *label_index >= labels.len() as i32 {
					for _i in labels.len()..(*label_index+1) as usize {
						labels.push(exprs.len() as i32); // unset labels default to the end of script
					}
				}

				labels[*label_index as usize] = index;
			}
			_ => {}
		}
		index += 1;
	}

	index = 0;

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
						_ => {
							eprintln!("Loop requires valid loop type");
							exit(1);
						}
					}
				}
			}
			Expr::LoopEnd => {
				if loop_index >= 0 {
					index = loop_index;
					index -= 1;
				}
			}
			Expr::Math(lhs, op, rhs) => {
				let mut res: i32 = 0;
				let mut failed: bool = false;
				match op {
					'_' => {
						if vars[lhs as usize].clone().as_int() != None {
							res = vars[lhs as usize].clone().as_int().unwrap() - rhs;
						}
						else{
							eprintln!("Cannot do math with string variable");
							exit(1);
						}
					}
					'~' => {
						if vars[rhs as usize].clone().as_int() != None {
							res = lhs - vars[rhs as usize].clone().as_int().unwrap();
						}
						else{
							eprintln!("Cannot do math with string variable");
							exit(1);
						}
					}
					'-' => {
						if vars[lhs as usize].clone().as_int() != None && vars[rhs as usize].clone().as_int() != None {
							res = vars[lhs as usize].clone().as_int().unwrap() - vars[rhs as usize].clone().as_int().unwrap();
						}
						else{
							eprintln!("Cannot do math with string variable");
							exit(1);
						}
					}
					'=' => {
						if vars[lhs as usize].clone().as_int() != None {
							res = vars[lhs as usize].clone().as_int().unwrap() + rhs;
						}
						else{
							eprintln!("Cannot do math with string variable");
							exit(1);
						}
					}
					'+' => {
						if vars[lhs as usize].clone().as_int() != None && vars[rhs as usize].clone().as_int() != None {
							res = vars[lhs as usize].clone().as_int().unwrap() + vars[rhs as usize].clone().as_int().unwrap();
						}
						else{
							eprintln!("Cannot do math with string variable");
							exit(1);
						}
					}
					'8' => {
						if vars[lhs as usize].clone().as_int() != None {
							res = vars[lhs as usize].clone().as_int().unwrap() * rhs;
						}
						else{
							eprintln!("Cannot do math with string variable");
							exit(1);
						}
					}
					'*' => {
						if vars[lhs as usize].clone().as_int() != None && vars[rhs as usize].clone().as_int() != None {
							res = vars[lhs as usize].clone().as_int().unwrap() * vars[rhs as usize].clone().as_int().unwrap();
						}
						else{
							eprintln!("Cannot do math with string variable");
							exit(1);
						}
					}
					'\\' => {
						if vars[rhs as usize].clone().as_int() != None {
							res = lhs / vars[rhs as usize].clone().as_int().unwrap();
						}
						else{
							eprintln!("Cannot do math with string variable");
							exit(1);
						}
					}
					'|' => {
						if vars[lhs as usize].clone().as_int() != None {
							res = vars[lhs as usize].clone().as_int().unwrap() / rhs;
						}
						else{
							eprintln!("Cannot do math with string variable");
							exit(1);
						}
					}
					'/' => {
						if vars[lhs as usize].clone().as_int() != None && vars[rhs as usize].clone().as_int() != None {
							res = vars[lhs as usize].clone().as_int().unwrap() / vars[rhs as usize].clone().as_int().unwrap();
						}
						else{
							eprintln!("Cannot do math with string variable");
							exit(1);
						}
					}
					_ => failed = true
				}
				if !failed {
					vars[selected_var as usize] = Var::Integer(res);
				}
				else {
					eprintln!("Found unknown math operator");
					exit(1);
				}
			}
			Expr::AddToPrint(var_index) => printing.push(var_index),
			Expr::AddCharToPrint(char_value) => printing.push(-char_value),
			Expr::AddVarCharPrint(var_index) => {
				if vars[var_index as usize].clone().as_int() != None {
					printing.push(-vars[var_index as usize].clone().as_int().unwrap());
				}
				else {
					eprintln!("Cannot convert string variable to character");
					exit(1);
				}
			}
			Expr::Print(count) => {
				let mut end = count as usize;
				if count == 0 || count > printing.len() as i32 {
					end = printing.len();
				}
				for i in 0..end {
					if printing[i] >= 0 {
						print_variable(vars[printing[i] as usize].clone());
						print!(" ");
					}
					else {
						print!("{}", -printing[i] as u8 as char);
					}
				}
				println!("");
				printing.clear();
			}
			Expr::Conditional(lhs, op, rhs, jump) => {
				let lhs_val: i32;
				let rhs_val: i32;

				if (op & 0b1000) == 8 {
					// literal
					lhs_val = lhs;
				}
				else{
					if vars[lhs as usize].clone().as_int() != None {
						lhs_val = vars[lhs as usize].clone().as_int().unwrap();
					}
					else {
						eprintln!("Cannot do conditional with string");
						exit(1);
					}
				}

				if (op & 0b0100) == 4 {
					// literal
					rhs_val = rhs;
				}
				else{
					if vars[rhs as usize].clone().as_int() != None {
						rhs_val = vars[rhs as usize].clone().as_int().unwrap();
					}
					else{
						eprintln!("Cannot do conditional with string");
						exit(1);
					}
				}

				match op & 0b0011 {
					0 => {
						if lhs_val != rhs_val {
							index = labels[jump as usize];
						}
					}
					1 => {
						if lhs_val == rhs_val {
							index = labels[jump as usize];
						}
					}
					2 => {
						if lhs_val < rhs_val {
							index = labels[jump as usize];
						}
					}
					3 => {
						if lhs_val > rhs_val {
							index = labels[jump as usize];
						}
					}
					_ => {
						eprintln!("Found unknown conditional operator");
						exit(1);
					}
				}
			}
			Expr::Exit => {
				index = exprs.len() as i32;
			}
			Expr::UserInput(should_prompt, prompt_var) => {
				if should_prompt {
					if prompt_var >= vars.len() as i32 {
						eprintln!("Prompt must be a valid variable");
						exit(1);
					}
					print_variable(vars[prompt_var as usize].clone());
				}
				let _ = io::stdout().flush();
				let mut inp = String::new();
				let _ = io::stdin().read_line(&mut inp);
				let _ = inp.pop();
				vars[selected_var as usize] = Var::Str(inp);
			}
			_ => {}
		}
		index += 1;
	}
	
	Ok(())
}

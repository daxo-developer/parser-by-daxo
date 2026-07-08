use std::fs;
use std::env;
use std::collections::HashMap;

// ==================== Токены ====================

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Fn,
    Let,
    If,
    Else,
    While,
    Identifier(String),
    Number(i32),
    Assign,
    Plus,
    Minus,
    Less,
    Equal,
    NotEqual,     // !=
    LessEqual,    // <=
    GreaterEqual, // >=
    Greater,      // >
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    Semicolon,
    Comma,
}

// ==================== Лексер ====================

pub fn tokenize(code: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = code.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        match c {
            ' ' | '\t' | '\n' | '\r' => { i += 1; continue; }
            '+' => { tokens.push(Token::Plus); i += 1; }
            '-' => { tokens.push(Token::Minus); i += 1; }
            '<' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::LessEqual);
                    i += 2;
                } else {
                    tokens.push(Token::Less);
                    i += 1;
                }
            }
            '>' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::GreaterEqual);
                    i += 2;
                } else {
                    tokens.push(Token::Greater);
                    i += 1;
                }
            }
            '=' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::Equal);
                    i += 2;
                } else {
                    tokens.push(Token::Assign);
                    i += 1;
                }
            }
            '!' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::NotEqual);
                    i += 2;
                } else {
                    panic!("Lexer Error: Unexpected character '!' without '='");
                }
            }
            '{' => { tokens.push(Token::OpenBrace); i += 1; }
            '}' => { tokens.push(Token::CloseBrace); i += 1; }
            '(' => { tokens.push(Token::OpenParen); i += 1; }
            ')' => { tokens.push(Token::CloseParen); i += 1; }
            ';' => { tokens.push(Token::Semicolon); i += 1; }
            ',' => { tokens.push(Token::Comma); i += 1; }
            _ if c.is_alphabetic() || c == '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                match word.as_str() {
                    "fn" => tokens.push(Token::Fn),
                    "let" => tokens.push(Token::Let),
                    "if" => tokens.push(Token::If),
                    "else" => tokens.push(Token::Else),
                    "while" => tokens.push(Token::While),
                    _ => tokens.push(Token::Identifier(word)),
                }
            }
            _ if c.is_digit(10) => {
                let start = i;
                while i < chars.len() && chars[i].is_digit(10) {
                    i += 1;
                }
                let num: i32 = chars[start..i].iter().collect::<String>().parse().unwrap();
                tokens.push(Token::Number(num));
            }
            _ => { i += 1; }
        }
    }
    tokens
}

// ==================== AST ====================

#[derive(Debug, Clone)]
pub enum Expr {
    Number(i32),
    Variable(String),
    BinaryOp {
        left: Box<Expr>,
        op: String,
        right: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
}

#[derive(Debug, Clone)]
pub enum Statement {
    Let { name: String, value: Expr },
    Assign { name: String, value: Expr },
    Expression(Expr),
    Function { name: String, body: Vec<Statement> },
    If {
        condition: Expr,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    },
    While {
        condition: Expr,
        body: Vec<Statement>,
    },
}

// ==================== Парсер ====================

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let tok = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(tok)
        } else {
            None
        }
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        if let Some(tok) = self.next() {
            if tok == expected {
                Ok(())
            } else {
                Err(format!("Expected {:?}, found {:?}", expected, tok))
            }
        } else {
            Err("Unexpected end of input stream".to_string())
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Statement>, String> {
        let mut stmts = Vec::new();
        while self.peek().is_some() {
            stmts.push(self.parse_statement()?);
        }
        Ok(stmts)
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.peek() {
            Some(Token::Fn) => self.parse_function(),
            Some(Token::Let) => self.parse_let(),
            Some(Token::If) => self.parse_if(),
            Some(Token::While) => self.parse_while(),
            Some(Token::Identifier(_)) => {
                let save_pos = self.pos;
                let name = match self.next() {
                    Some(Token::Identifier(id)) => id,
                    _ => {
                        self.pos = save_pos;
                        return Err("Expected variable name".to_string());
                    }
                };
                if let Some(Token::Assign) = self.peek() {
                    self.next();
                    let value = self.parse_expr()?;
                    self.expect(Token::Semicolon)?;
                    Ok(Statement::Assign { name, value })
                } else if let Some(Token::OpenParen) = self.peek() {
                    self.next();
                    let mut args = Vec::new();
                    if let Some(Token::CloseParen) = self.peek() {
                        self.next();
                    } else {
                        loop {
                            args.push(self.parse_expr()?);
                            match self.peek() {
                                Some(Token::CloseParen) => { self.next(); break; }
                                Some(Token::Comma) => { self.next(); continue; }
                                _ => return Err("Expected ')' or ','".to_string()),
                            }
                        }
                    }
                    self.expect(Token::Semicolon)?;
                    Ok(Statement::Expression(Expr::Call { name, args }))
                } else {
                    Err(format!("Unexpected token after identifier: {:?}", self.peek()))
                }
            }
            _ => Err(format!("Unknown instruction syntax: {:?}", self.peek())),
        }
    }

    fn parse_function(&mut self) -> Result<Statement, String> {
        self.expect(Token::Fn)?;
        let name = match self.next() {
            Some(Token::Identifier(id)) => id,
            _ => return Err("Expected function name".to_string()),
        };
        self.expect(Token::OpenParen)?;
        self.expect(Token::CloseParen)?;
        self.expect(Token::OpenBrace)?;

        let mut body = Vec::new();
        while let Some(tok) = self.peek() {
            if *tok == Token::CloseBrace {
                break;
            }
            body.push(self.parse_statement()?);
        }
        self.expect(Token::CloseBrace)?;
        Ok(Statement::Function { name, body })
    }

    fn parse_let(&mut self) -> Result<Statement, String> {
        self.expect(Token::Let)?;
        let name = match self.next() {
            Some(Token::Identifier(id)) => id,
            _ => return Err("Expected variable name".to_string()),
        };
        self.expect(Token::Assign)?;
        let expr = self.parse_expr()?;
        self.expect(Token::Semicolon)?;
        Ok(Statement::Let { name, value: expr })
    }

    fn parse_if(&mut self) -> Result<Statement, String> {
        self.expect(Token::If)?;
        self.expect(Token::OpenParen)?;
        let condition = self.parse_expr()?;
        self.expect(Token::CloseParen)?;
        self.expect(Token::OpenBrace)?;

        let mut then_branch = Vec::new();
        while let Some(tok) = self.peek() {
            if *tok == Token::CloseBrace { break; }
            then_branch.push(self.parse_statement()?);
        }
        self.expect(Token::CloseBrace)?;

        let mut else_branch = None;
        if let Some(Token::Else) = self.peek() {
            self.next();
            self.expect(Token::OpenBrace)?;
            let mut else_body = Vec::new();
            while let Some(tok) = self.peek() {
                if *tok == Token::CloseBrace { break; }
                else_body.push(self.parse_statement()?);
            }
            self.expect(Token::CloseBrace)?;
            else_branch = Some(else_body);
        }

        Ok(Statement::If { condition, then_branch, else_branch })
    }

    fn parse_while(&mut self) -> Result<Statement, String> {
        self.expect(Token::While)?;
        self.expect(Token::OpenParen)?;
        let condition = self.parse_expr()?;
        self.expect(Token::CloseParen)?;
        self.expect(Token::OpenBrace)?;

        let mut body = Vec::new();
        while let Some(tok) = self.peek() {
            if *tok == Token::CloseBrace { break; }
            body.push(self.parse_statement()?);
        }
        self.expect(Token::CloseBrace)?;
        Ok(Statement::While { condition, body })
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_primary()?;
        while let Some(tok) = self.peek() {
            match tok {
                Token::Plus | Token::Minus | Token::Less | Token::Equal |
                Token::NotEqual | Token::LessEqual | Token::GreaterEqual | Token::Greater => {
                    let op = match self.next().unwrap() {
                        Token::Plus => "+",
                        Token::Minus => "-",
                        Token::Less => "<",
                        Token::Equal => "==",
                        Token::NotEqual => "!=",
                        Token::LessEqual => "<=",
                        Token::GreaterEqual => ">=",
                        Token::Greater => ">",
                        _ => unreachable!(),
                    };
                    let right = self.parse_primary()?;
                    left = Expr::BinaryOp {
                        left: Box::new(left),
                        op: op.to_string(),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Some(Token::Identifier(id)) => {
                let name = id.clone();
                self.next();
                if let Some(Token::OpenParen) = self.peek() {
                    self.next();
                    let mut args = Vec::new();
                    if let Some(Token::CloseParen) = self.peek() {
                        self.next();
                    } else {
                        loop {
                            args.push(self.parse_expr()?);
                            match self.peek() {
                                Some(Token::CloseParen) => { self.next(); break; }
                                Some(Token::Comma) => { self.next(); continue; }
                                _ => return Err("Expected ')' or ','".to_string()),
                            }
                        }
                    }
                    Ok(Expr::Call { name, args })
                } else {
                    Ok(Expr::Variable(name))
                }
            }
            Some(Token::Number(n)) => {
                let val = *n;
                self.next();
                Ok(Expr::Number(val))
            }
            Some(Token::OpenParen) => {
                self.next();
                let expr = self.parse_expr()?;
                self.expect(Token::CloseParen)?;
                Ok(expr)
            }
            _ => Err(format!("Expected expression components, found: {:?}", self.peek())),
        }
    }
}

// ==================== Линейное представление для логов (IR) ====================

#[derive(Debug, Clone)]
pub enum LogIR {
    Push(i32),
    Load(String),
    Store(String),
    Add,
    Sub,
    Less,
    Equal,
    NotEqual,
    LessEqual,
    GreaterEqual,
    Greater,
    Call(String),
    LoadByte,
    StoreByte,
    IfStart,
    ElseStart,
    WhileStart,
    LoopEnd,
}

pub fn generate_log_ir(stmts: &[Statement], list: &mut Vec<LogIR>) {
    for s in stmts {
        match s {
            Statement::Function { body, .. } => generate_log_ir(body, list),
            Statement::Let { name, value } | Statement::Assign { name, value } => {
                generate_expr_log_ir(value, list);
                list.push(LogIR::Store(name.clone()));
            }
            Statement::Expression(e) => generate_expr_log_ir(e, list),
            Statement::If { condition, then_branch, else_branch } => {
                generate_expr_log_ir(condition, list);
                list.push(LogIR::IfStart);
                generate_log_ir(then_branch, list);
                if let Some(eb) = else_branch {
                    list.push(LogIR::ElseStart);
                    generate_log_ir(eb, list);
                }
                list.push(LogIR::LoopEnd);
            }
            Statement::While { condition, body } => {
                list.push(LogIR::WhileStart);
                generate_expr_log_ir(condition, list);
                generate_log_ir(body, list);
                list.push(LogIR::LoopEnd);
            }
        }
    }
}

fn generate_expr_log_ir(expr: &Expr, list: &mut Vec<LogIR>) {
    match expr {
        Expr::Number(n) => list.push(LogIR::Push(*n)),
        Expr::Variable(v) => list.push(LogIR::Load(v.clone())),
        Expr::BinaryOp { left, op, right } => {
            generate_expr_log_ir(left, list);
            generate_expr_log_ir(right, list);
            match op.as_str() {
                "+" => list.push(LogIR::Add),
                "-" => list.push(LogIR::Sub),
                "<" => list.push(LogIR::Less),
                "==" => list.push(LogIR::Equal),
                "!=" => list.push(LogIR::NotEqual),
                "<=" => list.push(LogIR::LessEqual),
                ">=" => list.push(LogIR::GreaterEqual),
                ">" => list.push(LogIR::Greater),
                _ => {}
            }
        }
        Expr::Call { name, args } => {
            if name == "load_byte" {
                generate_expr_log_ir(&args[0], list);
                list.push(LogIR::LoadByte);
            } else if name == "store_byte" {
                generate_expr_log_ir(&args[0], list);
                generate_expr_log_ir(&args[1], list);
                list.push(LogIR::StoreByte);
            } else {
                for arg in args { generate_expr_log_ir(arg, list); }
                list.push(LogIR::Call(name.clone()));
            }
        }
    }
}

// ==================== Эмиттер WebAssembly ====================

pub struct WasmEmitter;

impl WasmEmitter {
    pub fn emit(stmts: &[Statement]) -> Vec<u8> {
        let mut locals = HashMap::new();
        let mut local_idx = 0;
        
        fn find_locals(stmts: &[Statement], locals: &mut HashMap<String, u8>, local_idx: &mut u8) {
            for stmt in stmts {
                match stmt {
                    Statement::Let { name, .. } => {
                        if !locals.contains_key(name) {
                            locals.insert(name.clone(), *local_idx);
                            *local_idx += 1;
                        }
                    }
                    Statement::If { then_branch, else_branch, .. } => {
                        find_locals(then_branch, locals, local_idx);
                        if let Some(eb) = else_branch { find_locals(eb, locals, local_idx); }
                    }
                    Statement::While { body, .. } => {
                        find_locals(body, locals, local_idx);
                    }
                    _ => {}
                }
            }
        }
        find_locals(stmts, &mut locals, &mut local_idx);

        let mut body = Vec::new();
        if local_idx > 0 {
            body.push(1); 
            body.push(local_idx); 
            body.push(0x7F); 
        } else {
            body.push(0);
        }

        fn compile_stmts(stmts: &[Statement], body: &mut Vec<u8>, locals: &HashMap<String, u8>) {
            for stmt in stmts {
                match stmt {
                    Statement::Let { name, value } | Statement::Assign { name, value } => {
                        compile_expr(value, body, locals);
                        let idx = *locals.get(name).unwrap();
                        body.push(0x21); 
                        body.push(idx);
                    }
                    Statement::Expression(expr) => {
                        compile_expr(expr, body, locals);
                        if let Expr::Call { name, .. } = expr {
                            if name == "read_file" || name == "alloc" || name == "load_byte" {
                                body.push(0x1A); 
                            }
                        }
                    }
                    Statement::If { condition, then_branch, else_branch } => {
                        compile_expr(condition, body, locals);
                        body.push(0x04); 
                        body.push(0x40); 
                        compile_stmts(then_branch, body, locals);
                        if let Some(eb) = else_branch {
                            body.push(0x05); 
                            compile_stmts(eb, body, locals);
                        }
                        body.push(0x0B); 
                    }
                    Statement::While { condition, body: w_body } => {
                        body.push(0x02); 
                        body.push(0x40); 
                        body.push(0x03); 
                        body.push(0x40); 
                        
                        compile_expr(condition, body, locals);
                        body.push(0x45); 
                        body.push(0x0D); 
                        body.push(0x01); 
                        
                        compile_stmts(w_body, body, locals);
                        
                        body.push(0x0C); 
                        body.push(0x00); 
                        
                        body.push(0x0B); 
                        body.push(0x0B); 
                    }
                    _ => {}
                }
            }
        }

        fn compile_expr(expr: &Expr, body: &mut Vec<u8>, locals: &HashMap<String, u8>) {
            match expr {
                Expr::Number(n) => {
                    body.push(0x41); 
                    write_sleb128(body, *n);
                }
                Expr::Variable(name) => {
                    let idx = *locals.get(name).unwrap();
                    body.push(0x20); 
                    body.push(idx);
                }
                Expr::BinaryOp { left, op, right } => {
                    compile_expr(left, body, locals);
                    compile_expr(right, body, locals);
                    match op.as_str() {
                        "+" => body.push(0x6A),
                        "-" => body.push(0x6B),
                        "<" => body.push(0x48),  // i32.lt_s
                        "==" => body.push(0x46), // i32.eq
                        "!=" => body.push(0x47), // i32.ne
                        ">" => body.push(0x4A),  // i32.gt_s
                        "<=" => body.push(0x4C), // i32.le_s
                        ">=" => body.push(0x4E), // i32.ge_s
                        _ => {}
                    }
                }
                Expr::Call { name, args } => {
                    if name == "load_byte" {
                        compile_expr(&args[0], body, locals);
                        body.push(0x2D); body.push(0x00); body.push(0x00);
                    } else if name == "store_byte" {
                        compile_expr(&args[0], body, locals);
                        compile_expr(&args[1], body, locals);
                        body.push(0x3A); body.push(0x00); body.push(0x00);
                    } else {
                        for arg in args { compile_expr(arg, body, locals); }
                        let idx = match name.as_str() {
                            "print" => 0,
                            "read_file" => 1,
                            "write_file" => 2,
                            "alloc" => 3,
                            _ => 0,
                        };
                        body.push(0x10); 
                        body.push(idx);
                    }
                }
            }
        }

        compile_stmts(stmts, &mut body, &locals);
        body.push(0x0B); 

        let mut wasm = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];

        let mut type_sec = vec![4]; 
        type_sec.extend_from_slice(&[0x60, 0x01, 0x7F, 0x00]); 
        type_sec.extend_from_slice(&[0x60, 0x01, 0x7F, 0x01, 0x7F]); 
        type_sec.extend_from_slice(&[0x60, 0x03, 0x7F, 0x7F, 0x7F, 0x01, 0x7F]); 
        type_sec.extend_from_slice(&[0x60, 0x00, 0x00]); 

        wasm.push(1); wasm.push(type_sec.len() as u8); wasm.extend(type_sec);

        let mut imp_sec = vec![4]; 
        imp_sec.extend_from_slice(&[3, b'e', b'n', b'v', 5, b'p', b'r', b'i', b'n', b't', 0, 0]);
        imp_sec.extend_from_slice(&[3, b'e', b'n', b'v', 9, b'r', b'e', b'a', b'd', b'_', b'f', b'i', b'l', b'e', 0, 1]);
        imp_sec.extend_from_slice(&[3, b'e', b'n', b'v', 10, b'w', b'r', b'i', b't', b'e', b'_', b'f', b'i', b'l', b'e', 0, 2]);
        imp_sec.extend_from_slice(&[3, b'e', b'n', b'v', 5, b'a', b'l', b'l', b'o', b'c', 0, 1]);

        wasm.push(2); wasm.push(imp_sec.len() as u8); wasm.extend(imp_sec);

        wasm.push(3); wasm.push(2); wasm.push(1); wasm.push(3);

        wasm.push(5); wasm.push(3); wasm.push(1); wasm.push(0); wasm.push(1);

        let mut exp_sec = vec![2]; 
        exp_sec.extend_from_slice(&[6, b'm', b'e', b'm', b'o', b'r', b'y', 2, 0]);
        exp_sec.extend_from_slice(&[4, b'm', b'a', b'i', b'n', 0, 4]); 

        wasm.push(7); wasm.push(exp_sec.len() as u8); wasm.extend(exp_sec);

        let mut code_sec = vec![1]; 
        code_sec.push(body.len() as u8); code_sec.extend(body);

        wasm.push(10); wasm.push(code_sec.len() as u8); wasm.extend(code_sec);

        wasm
    }
}

fn write_sleb128(vec: &mut Vec<u8>, mut val: i32) {
    loop {
        let byte = (val & 0x7F) as u8;
        val >>= 7;
        if (val == 0 && (byte & 0x40) == 0) || (val == -1 && (byte & 0x40) != 0) {
            vec.push(byte);
            break;
        } else {
            vec.push(byte | 0x80);
        }
    }
}

// ==================== Точка входа ====================

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let filename = if args.len() > 1 { args[1].clone() } else { "test.dx".to_string() };

    let code = fs::read_to_string(&filename)
        .map_err(|e| format!("IO Error: Cannot read '{}': {}", filename, e))?;

    println!("--- [1] Source High-Level Code ---\n{}", code);

    let tokens = tokenize(&code);
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;
    println!("--- [2] Generated Abstract Syntax Tree (AST) ---\n{:#?}", ast);

    let mut log_ir = Vec::new();
    generate_log_ir(&ast, &mut log_ir);
    println!("--- [3] Linear Stack-Based Intermediate Representation (IR) ---");
    for (idx, ins) in log_ir.iter().enumerate() {
        println!("  {:02}: {:?}", idx, ins);
    }

    let wasm_bytes = WasmEmitter::emit(&ast);
    fs::write("output.wasm", &wasm_bytes)
        .map_err(|e| format!("Write Error: {}", e))?;

    println!("--- [4] Success! Dynamic Binary 'output.wasm' Generated and Ready to Load ---");
    Ok(())
}

use std::io::{self, Read};
use jocky_lexer::Lexer;
use jocky_parser::Parser;
use serde::Serialize;

#[derive(Serialize)]
struct CompilerResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    ast: Option<jocky_ast::Program>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

fn main() {
    let mut source = String::new();
    if let Err(e) = io::stdin().read_to_string(&mut source) {
        let resp = CompilerResponse {
            success: false,
            ast: None,
            error: Some(format!("Failed to read stdin: {}", e)),
        };
        println!("{}", serde_json::to_string(&resp).unwrap());
        std::process::exit(1);
    }

    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            let resp = CompilerResponse {
                success: false,
                ast: None,
                error: Some(format!("Lexer error: {:?}", e)),
            };
            println!("{}", serde_json::to_string(&resp).unwrap());
            std::process::exit(1);
        }
    };

    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(program) => {
            let resp = CompilerResponse {
                success: true,
                ast: Some(program),
                error: None,
            };
            println!("{}", serde_json::to_string(&resp).unwrap());
        }
        Err(e) => {
            let resp = CompilerResponse {
                success: false,
                ast: None,
                error: Some(format!("Parser error: {:?}", e)),
            };
            println!("{}", serde_json::to_string(&resp).unwrap());
            std::process::exit(1);
        }
    }
}

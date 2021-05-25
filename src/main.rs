mod lex;

fn main() {
    let input ="IF+-123 foo*THEN/";
    let mut lexer = lex::Lexer::new(input);

    let mut token = lexer.get_token();
    while token.kind != lex::TokenType::EOF {
	println!("{:?}",token.kind);
	token = lexer.get_token();
    }
}



// https://craftinginterpreters.com/scanning.html

enum Position {

}

enum Target {

}


enum ItemCost {
    Free,
    Gold(usize),
}
enum TokenType {
    Action,
    EffectCondition,
    Number(usize),
    Position(Position),
    Target(Target),
    Attack(usize),
    Health(usize),
    Damage(usize),
    Item { cost: ItemCost, name: String },
    For,
    From,
    To,
    Comma,
    Period,
    End,
}

struct Token<'src> {
    ttype: TokenType,
    text: &'src str,
    literal: Option<char>,
    line: usize
}

#[derive(Default)]
struct Scanner<'src> {
    source: String,
    tokens: Vec<Token<'src>>,
    /// Start character index of lexeme.
    start: usize,
    /// Current character index of lexeme.
    current: usize,
    /// Curent line
    line: usize,
}

impl<'src> Scanner<'src> {
    fn new(source: String) -> Self {
        Scanner {
            source,
            line: 1,
            ..Default::default()
        }
    }

    fn scan_tokens(&mut self) -> &mut Self {
        loop {
            self.start = self.current;

            if self.scan_token().is_none() {
                break;
            }
        }
        self.tokens.push(Token { ttype: TokenType::End, text: "", literal: None, line: self.line });
        self
    }

    /// Advance to next character.
    fn advance(&mut self) -> Option<char> {
        // Assume ASCII.
        // TODO: Replace Option with Error to allow multiple states (non-valid char, eof, etc.).
        if let Some(char) = self
            .source
            .as_bytes()
            .get(self.current)
            .filter(|byte| byte.is_ascii())
            .map(|byte| *byte as char)
        {
            self.current += 1;
            Some(char)
        } else {
            None
        }
    }

    fn add_token(&'src mut self, ttype: TokenType, literal: Option<char>) {
        let Some(text) = self.source.get(self.start..self.current) else {
            panic!("Invalid start {} and current {} indices.", self.start, self.current)
        };
        self.tokens.push(Token { ttype, text, literal, line: self.line });
    }

    fn scan_token(&mut self) -> Option<()>{
        let Some(c) = self.advance() else {
            return None;
        };
        match c {
            _ => todo!()
        }
        Some(())
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}

fn main() {

}

mod test {
    #[test]
    fn test_lex() {}
}

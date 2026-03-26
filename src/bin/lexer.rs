use std::error::Error;

use shiperon::{Lexer, TokenRegistry};

fn main() -> Result<(), Box<dyn Error>> {
    let mut lexer = Lexer::of_str(
        "
class Program is
    this() is
        var sum: Integer(0)
        var limit: Integer(10)
        var i: 0
        while i.Less(limit) loop
            if i.Rem(2).Equal(0) then
                sum := sum.Plus(i)
            end
        end
        return sum
    end
end

            ",
    );
    loop {
        let token = lexer.yylex();
        println!("{token}");
        let token_type = token.token_type;
        if token_type == TokenRegistry::YYEOF {
            break;
        }
    }
    Ok(())
}

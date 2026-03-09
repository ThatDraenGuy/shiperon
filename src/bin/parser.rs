use std::error::Error;

use ron::ser::PrettyConfig;
use shiperon::{Lexer, Parser};

fn main() -> Result<(), Box<dyn Error>> {
    let mut parser = Parser::new(
        Lexer::of_str(
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
        ),
        true,
    );
    parser.parse();
    let str_result = ron::ser::to_string_pretty(&parser.result, PrettyConfig::default())?;
    println!("{str_result}");
    // println!("{:?}", parser.result);
    Ok(())
}

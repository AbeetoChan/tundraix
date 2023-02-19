use tundraix_src::compiler::Parser;
use tundraix_src::vm::VM;
use tundraix_src::error::ErrorResult;

fn print_fn(text: String) -> ErrorResult<()> {
    print!("{}", text);
    Ok(())
}

fn main() -> ErrorResult<()> {
    let mut parser = Parser::new(r#"
        var a = 3;
        var b = 4 + 2 * a;
        print b;
        b = 4;
        print b;
    "#);

    let chunk = parser.parse()?;
    let mut vm = VM::new(print_fn);
    vm.interpret(chunk)?;
    Ok(())
}
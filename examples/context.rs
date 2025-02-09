use libphp::exec::Context;

fn main() {
    let mut context = Context::new();

    let true_ = context.result_of("true", false);
    let false_ = context.result_of("false", false);
    let integer = context.result_of("100_000_000", false);
    let float = context.result_of("100.525", false);
    let null = context.result_of("null", false);
    let string = context.result_of("'Hello, world!'", false);
    let array = context.result_of("['Hello', 'world!']", false);

    println!("Converting between PHP and Rust values:");
    println!("true = {true_:?}");
    println!("false = {false_:?}");
    println!("integer = {integer:?}");
    println!("float = {float:?}");
    println!("null = {null:?}");
    println!("string = {string:?}");
    println!("array = {array:?}");
}

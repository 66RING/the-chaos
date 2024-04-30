
use prompt_manager::{make_prompt_template, make_func};

make_prompt_template!{
    hello_prompt,
    "\t  Hello {name}"
}

// generate a function named test_fn
make_func!(fn);

fn main() {
    println!("{}", hello_prompt("66ring"));
    test_fn();
}

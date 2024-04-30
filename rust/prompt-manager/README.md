# Prompt-manager

prompt template manager copy from [prompt-organizer](https://github.com/m1guelpf/prompt-organizer)

## Usage

Create template

```rust
prompt!{my_example_prompt, "
    You are {name}, an AI assistant.
"}
```

Use template

```rust
assert_eq!(my_example_prompt("some name"), "You are some name, an AI assistant.");
```


## impl

1. use `#[proc_macro]` to generate prompt template function
2. implement `Parse` trait to parse `input: TokenStream` to helpful structure
    - func name(`func_name`) and corresponding arguments(`prompt`)
        * `macro!(name, "prompt")`
        * first token into `func_name: ExprPath`, ParseStream::parse
        * consume `,`
        * rest of all tokens into `prompt: LitStr`, ParseStream::parse
3. generate with `#[proc_macro]`
    - pick out all named argument
    - compose `::std::format!(#prompt, #(#args = #args),*)`


### dev log

[tutorial](https://doc.rust-lang.org/reference/procedural-macros.html)

本质: 使用过程宏根据用户指定的名字创建函数, 函数都是字符串格式化, 而名字不同以区分不同的template。

启用过程宏

``` toml
# Cargo.toml
[lib]
proc-macro = true
```

使用过程宏, 相当于动态生成代码。

需要写在`lib.rs`中

```rust
// lib.rs
extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;

#[proc_macro]
pub fn make_func(ident: TokenStream) -> TokenStream {
    let func_name = format!("test_{}", ident.to_string());
    let ident_func_name = Ident::new(&func_name, Span::call_site());
    let exptened = quote! {
        pub fn #ident_func_name() {
            println!("call from {}", #func_name);
        }
    };

    exptened.into()
}
```

使用

```rust
// main.rs
use lib_name::{make_prompt_template, make_func};

// generate a function named test_fn
make_func!(fn);

fn main() {
    test_fn();
}
```

## rust tip

- `proc_macro`
    * `#var`插入作用域的变量, 变量需要是`Ident(name, scope)`类型
    * 可变参数的扩展: `#(#args: &str),*`, "*相当于重复"
- `parse_format`解析字符串format相关的东西
    * ParseMode, Parser, Piece, Position
    * 纯文本, `parse_format::Piece::String(_)`
    * format参数, `parse_format::Piece::NextArgument(arg)`
    * format参数的类型, `!matches!(arg.position, Position::ArgumentNamed(_)))`
- `parser`
    * `let func_name: ExprPath = parse()?`会根据目标类型自动消耗token并返回结果
    * 抽象语法树`func_name.path`




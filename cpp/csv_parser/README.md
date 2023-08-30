# CSV解析

- 带列名, with headers
    * 手动排除第一行
- 不带列名, without headers
- 统计最大行数和列数, 开辟空间
- 使用`getline(sstream, line)`读取每行
- 使用`getline(line_stream, token, split_char)`读取每个token
- 使用`stof(token)`转换成float


## API

- 额为向量成员的初始化``
- `getline(input_stream, output_value, split_char)`
    * 读取一行的中的一个token, `split_chat`变量分隔
- `std::stof(string)`
    * string转float


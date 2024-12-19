use proc_macro::*;

#[proc_macro_attribute]
pub fn rim_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    
    let mut ignore = false;

    for rule in split_attr(attr) {
        match rule.as_str() {
            // 插入不同版本进行测试
            "should_ignore" => {
                ignore = true;
            }
            _ => panic!("unknown rule: {:?}", rule),
        }
    }

    // 获取当前调用位置的 Span
    let span = Span::call_site();
    let mut ret = TokenStream::new();
    // 闭包：创建 #[attr_name(attr_input)]
    let add_attr = |ret: &mut TokenStream, attr_name: &str, attr_input: Option<TokenStream>| {
        ret.extend(Some(TokenTree::from(Punct::new('#', Spacing::Alone))));
        let attr = TokenTree::from(Ident::new(attr_name, span));
        let mut attr_stream: TokenStream = attr.into();
        if let Some(input) = attr_input {
            attr_stream.extend(input)
        }
        ret.extend(Some(TokenTree::from(Group::new(
            Delimiter::Bracket,
            attr_stream 
        ))));
    };
    
    // 添加`#[test]`
    add_attr(&mut ret, "test", None);
    
    if ignore {
        // 添加 `#[ignore]` reason 暂时不急。
        add_attr(&mut ret, "ignore", None);
    }

    // 找到原函数的Body
    for token in item {
        let group = match token {
            // 直到找到 `{}`
            TokenTree::Group(g) => {
                if g.delimiter() == Delimiter::Brace {
                    g
                } else {
                    ret.extend(Some(TokenTree::Group(g)));
                    continue;
                }
            }
            other => {
                ret.extend(Some(other));
                continue;
            }
        };

        // 初始化一个临时目录
        let mut init_resource = parse_to_token_stream(
            r#"let _init_test = {
                let tmp_dir = option_env!("RIM_TARGET_TMPDIR");
                rim_test_support::paths::init_root(tmp_dir)
            };"#,
        );

        init_resource.extend(group.stream());
        ret.extend(Some(TokenTree::from(Group::new(
            group.delimiter(), 
            init_resource,
        ))));
    }
    
    ret
}

fn split_attr(attr: TokenStream) -> Vec<String> {
    let attrs: Vec<_> = attr.into_iter().collect();
    attrs.split(|tt| match tt {
        TokenTree::Punct(p) => p.as_char() == ',',
        _ => false,
    })
    .filter(|tt| !tt.is_empty())
    .map(|tt| {
        tt.into_iter()
            .map(|p| p.to_string())
            .collect::<String>()
    })
    .collect()
}

fn parse_to_token_stream(code: &str) -> TokenStream {
    code.parse().unwrap()
}

#[macro_export]
macro_rules! regex_color_replace {
    ($str:expr, $($reg:literal, $rep:literal, $col:ident)*) => {
        $(
            let re = regex::Regex::new($reg).unwrap();
            $str = re
                .replace_all(
                    &$str,
                    ansi_term::Color::$col
                        .bold()
                        .paint($rep)
                        .to_string(),
                )
                .into();
        )*
    };
}

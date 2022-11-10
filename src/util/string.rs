#[derive(Debug, Default, Clone)]
enum UnindentLine {
    #[default]
    Empty,
    Content(String, usize),
}

pub fn unindent(input: String, remove_surrounding: bool, add_eof_newline: bool) -> String {
    let mut unindent_lines: Vec<UnindentLine> = input.lines()
        .map(|s| {
            let trimmed = s.trim_start();
            if trimmed.len() > 0 {
                UnindentLine::Content(s.to_string(), s.len() - trimmed.len())
            } else {
                UnindentLine::Empty
            }
        })
        .collect();
    
    if remove_surrounding {
        unindent_lines = unindent_lines
            .iter()
            .skip_while(|v| matches!(v, UnindentLine::Empty))
            .collect::<Vec<&UnindentLine>>()
            .iter()
            .rev()
            .skip_while(|v| matches!(v, UnindentLine::Empty))
            .map(|&v| v.clone())
            .collect();
        unindent_lines.reverse();
    }

    let last_item = unindent_lines.last().unwrap_or(&UnindentLine::Empty);
    if add_eof_newline && matches!(last_item, &UnindentLine::Content(_, _)) {
        unindent_lines.push(UnindentLine::Empty);
    }

    let min = unindent_lines
        .iter()
        .filter(|l| match l {
            UnindentLine::Empty => false,
            UnindentLine::Content(_, _) => true,
        })
        .map(|l| {
            match l {
                UnindentLine::Content(_, w) => w,
                _ => unreachable!(),
            }
        })
        .min()
        .unwrap_or(&0);

    let processed = unindent_lines
        .iter()
        .map(|l| match l {
            UnindentLine::Empty => "".to_string(),
            UnindentLine::Content(string, _) => string.chars().skip(*min).collect::<String>(),
        })
        .collect::<Vec<String>>();

    processed.join("\n")
}

#[cfg(test)]
use super::formatter::format_nwpython;

#[test]
fn test_basic_formatting() {
    let input = "def main(){\nprint(\"hi\");\n}\n";
    let expected = "def main(){\n    print(\"hi\");\n}\n";
    let output = format_nwpython(input);
    assert_eq!(output, expected);
}

#[test]
fn test_multiline_comment() {
    let input = "/*\nthis is a comment\n*/\ndef foo(){\nbar();\n}\n";
    let expected = "/*\nthis is a comment\n*/\ndef foo(){\n    bar();\n}\n";
    let output = format_nwpython(input);
    assert_eq!(output, expected);
}

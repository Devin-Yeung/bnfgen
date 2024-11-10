pub mod alt;
pub mod checked;
pub mod graph;
pub mod production;
pub mod raw;
pub mod rule;
pub mod state;
pub mod symbol;

#[cfg(test)]
mod test {
    use crate::grammar::raw::RawGrammar;
    use crate::report::{Reporter, Style};
    use miette::{Diagnostic, Report};
    use std::sync::Arc;

    fn report_with_unnamed_source<T: Diagnostic + Sync + Send + 'static, S: ToString>(
        err: T,
        source: S,
    ) -> String {
        let source = Arc::new(source.to_string());
        let diagnostic = Report::from(err).with_source_code(source);

        let mut reporter = Reporter::new(Style::NoColor);
        reporter.push(diagnostic);
        reporter.report_to_string()
    }

    #[test]
    fn brainfuck() {
        let text = include_str!("../../examples/brainfuck.bnfgen");
        let grammar = RawGrammar::parse(text).unwrap();
        insta::assert_debug_snapshot!(grammar);
    }

    #[test]
    fn typed() {
        let text = r#"
            <E> ::= <E: "int"> "+" <E: "int"> ;
        "#;
        let grammar = RawGrammar::parse(text).unwrap();
        insta::assert_debug_snapshot!(grammar);
    }

    #[test]
    fn repeat() {
        let text = r#"
            <E> ::= "a" {1, 10} | "b" {2, } | "c" {3} ;
        "#;
        let grammar = RawGrammar::parse(text).unwrap();
        insta::assert_debug_snapshot!(grammar);
    }

    #[test]
    fn unexpected_eof() {
        let text = "<start> ::= \"Hello\" | \"World\""; // no semi
        let err = RawGrammar::parse(text).err().unwrap();
        let ui = report_with_unnamed_source(err, text);
        insta::assert_snapshot!(ui);
    }

    #[test]
    fn invalid_token() {
        let text = "*";
        let err = RawGrammar::parse(text).err().unwrap();
        let ui = report_with_unnamed_source(err, text);
        insta::assert_snapshot!(ui);
    }

    #[test]
    fn invalid_re() {
        let text = r#"<R> ::= re("["); "#;
        let err = RawGrammar::parse(text).err().unwrap();
        let ui = report_with_unnamed_source(err, text);
        insta::assert_snapshot!(ui);
    }

    #[test]
    fn undefined_nt() {
        let text = "<E> ::= <S>;";
        let err = RawGrammar::parse(text).unwrap().to_checked().err().unwrap();
        let ui = report_with_unnamed_source(err, text);
        insta::assert_snapshot!(ui);
    }

    #[test]
    fn duplicated_def() {
        let text = r#"
            <E> ::= <S>;
            <S> ::= <E>;
            <E> ::= "?";
        "#;
        let err = RawGrammar::parse(text).unwrap().to_checked().err().unwrap();
        let ui = report_with_unnamed_source(err, text);
        insta::assert_snapshot!(ui);
    }

    #[test]
    fn invalid_repeat() {
        let text = r#"
            <E> ::= "a" {10, 1};
        "#;
        let err = RawGrammar::parse(text).unwrap().to_checked().err().unwrap();
        let ui = report_with_unnamed_source(err, text);
        insta::assert_snapshot!(ui);
    }

    #[test]
    fn unreachable_nt() {
        let text = r#"
            <E> ::= "Hello" | <A> ;
            <W> ::= "World" ;
            <A> ::= <B> ;
            <B> ::= <A> ;
            <C> ::= <W> ;
        "#;
        let err = RawGrammar::parse(text)
            .unwrap()
            .graph()
            .check_unused("E")
            .err()
            .unwrap();
        let ui = report_with_unnamed_source(err, text);
        insta::assert_snapshot!(ui);
    }

    // TODO: bug, this is also a dead loop
    // #[test]
    // fn common_dead_loop() {
    //     let text = r#"
    //         <S> ::= <S> | <S> <E> ;
    //         <E> ::= "Terminal" ;
    //     "#;
    //     let err = RawGrammar::parse(text)
    //         .unwrap()
    //         .graph()
    //         .check_trap_loop()
    //         .err()
    //         .unwrap();
    //     let ui = report_with_unnamed_source(err, text);
    //     insta::assert_snapshot!(ui);
    // }

    #[test]
    fn trap_loop() {
        let text = r#"
            <E> ::= <D> | <F>;
            <C> ::= <D> ;
            <D> ::= <C> ;
            <F> ::= <G> ;
            <G> ::= <F> | "Terminal" ;
        "#;
        let err = RawGrammar::parse(text)
            .unwrap()
            .graph()
            .check_trap_loop()
            .err()
            .unwrap();
        let ui = report_with_unnamed_source(err, text);
        insta::assert_snapshot!(ui);
    }

    #[test]
    fn tri_loop() {
        let text = r#"
            <no-trap-01> ::= <A> | "Terminal" ;
            <no-trap-02> ::= <A> | <term> ;
            <term> ::= "Terminal" ;
            <A> ::= <B> ;
            <B> ::= <C> ;
            <C> ::= <A> ;
        "#;
        let err = RawGrammar::parse(text)
            .unwrap()
            .graph()
            .check_trap_loop()
            .err()
            .unwrap();
        let ui = report_with_unnamed_source(err, text);
        insta::assert_snapshot!(ui);
    }
}

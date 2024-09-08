use bnfgen::generator::Generator;
use bnfgen::grammar::RawGrammar;
fn main() {
    let input = include_str!("bnf.bnfgen");
    let grammar = RawGrammar::parse(input)
        .expect("Fail to parse the grammar")
        .to_checked()
        .expect("Grammar does not satisfy the requirements");
    let gen = Generator::builder().grammar(grammar).build();
    let out = gen.generate("syntax", &mut rand::thread_rng());
    println!("{}", out);
}

use bnfgen::generator::Generator;
use bnfgen::grammar::RawGrammar;
fn main() {
    let input = include_str!("bnf.bnfgen");
    let grammar = RawGrammar::parse(input).to_checked();
    let gen = Generator::builder().grammar(grammar).build();
    let out = gen.generate("syntax", &mut rand::thread_rng());
    println!("{}", out);
}

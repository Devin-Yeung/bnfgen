use rand::Rng;
use regex_syntax::hir::{Class, Hir, HirKind};

#[repr(transparent)]
#[derive(Debug)]
pub struct Regex {
    hir: Hir,
}

impl Regex {
    pub fn new(input: &str) -> Self {
        let hir = regex_syntax::Parser::new().parse(input).unwrap();
        Regex { hir }
    }

    pub fn generate<R: Rng>(&self, rng: &mut R) -> String {
        Self::helper(&self.hir, rng)
    }

    fn helper<R: Rng>(re: &Hir, rng: &mut R) -> String {
        match re.kind() {
            HirKind::Empty => String::new(),
            HirKind::Literal(lit) => String::from_utf8(lit.0.clone().into()).unwrap(),
            HirKind::Repetition(rep) => {
                let mut buf = Vec::new();
                // todo: allow manually set the max reps
                for _ in 0..rng.gen_range(rep.min..=rep.max.unwrap_or(10)) {
                    buf.push(Self::helper(&rep.sub, rng));
                }
                buf.join("")
            }
            HirKind::Concat(cat) => cat.iter().map(|h| Self::helper(h, rng)).collect(),
            HirKind::Alternation(alt) => {
                let idx = rng.gen_range(0..alt.len());
                Self::helper(&alt[idx], rng)
            }
            HirKind::Class(cls) => match cls {
                Class::Unicode(unicode) => {
                    let idx = rng.gen_range(0..unicode.iter().count());
                    let range = unicode.iter().nth(idx).unwrap();
                    let pick = rng.gen_range(range.start()..=range.end()) as char;
                    pick.to_string()
                }
                Class::Bytes(bytes) => {
                    let idx = rng.gen_range(0..bytes.iter().count());
                    let range = bytes.iter().nth(idx).unwrap();
                    let pick = rng.gen_range(range.start()..=range.end()) as char;
                    pick.to_string()
                }
            },
            HirKind::Look(_) => todo!(),
            HirKind::Capture(cap) => Self::helper(&cap.sub, rng),
        }
    }
}

#[cfg(test)]
mod test {
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn it_works() {
        let mut rng = StdRng::seed_from_u64(42);
        let re = super::Regex::new("[a-zA-Z0-9]*");
        let generated = (0..10).map(|_| re.generate(&mut rng)).collect::<Vec<_>>();
        insta::assert_debug_snapshot!(generated);
    }
}

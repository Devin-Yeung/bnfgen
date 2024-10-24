use crate::grammar::alt::Alternative;
use crate::grammar::symbol::SymbolKind;
use rand::distributions::Distribution;
use rand::distributions::WeightedIndex;
use rand::Rng;

#[derive(Debug)]
#[repr(transparent)]
pub struct WeightedProduction {
    pub(crate) alts: Vec<Alternative>,
}

impl WeightedProduction {
    pub fn choose<R: Rng>(&self, rng: &mut R) -> Vec<SymbolKind> {
        let dist = WeightedIndex::new(self.alts.iter().map(|a| a.weight)).unwrap();
        let idx = dist.sample(rng);
        self.alts[idx]
            .symbols
            .iter()
            .map(|s| s.kind.clone())
            .collect()
    }

    pub fn non_re_terminals(&self) -> Vec<&str> {
        self.alts
            .iter()
            .flat_map(|a| a.non_re_terminals())
            .collect()
    }
}

use crate::grammar::alt::Alternative;
use crate::grammar::state::State;
use crate::grammar::symbol::SymbolKind;
use rand::distr::weighted::WeightedIndex;
use rand::distr::Distribution;
use rand::Rng;

#[derive(Debug)]
#[repr(transparent)]
pub struct WeightedProduction {
    pub(crate) alts: Vec<Alternative>,
}

impl WeightedProduction {
    pub(crate) fn choose_by_state<R: Rng>(&self, state: &mut State<R>) -> Vec<SymbolKind> {
        let candidates = match self.alts.iter().any(|alt| alt.lose_invoke_limit(state)) {
            true => self
                .alts
                .iter()
                .filter(|alt| alt.lose_invoke_limit(state))
                .collect::<Vec<_>>(),
            false => self
                .alts
                .iter()
                .filter(|alt| !alt.exceeds_invoke_limit(state))
                .collect::<Vec<_>>(),
        };

        let dist = WeightedIndex::new(candidates.iter().map(|a| a.weight)).unwrap();
        let idx = dist.sample(state.rng());

        // tracking the selected alternative
        if candidates[idx].has_invoke_limits() {
            state.track(candidates[idx].id());
        }

        candidates[idx]
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

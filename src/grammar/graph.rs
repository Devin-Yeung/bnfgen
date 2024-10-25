use crate::error::Error;
use crate::grammar::rule::Rule;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::prelude::Dfs;
use std::collections::{HashMap, HashSet};

pub struct GrammarGraph<'rule> {
    pub(crate) rules: &'rule Vec<Rule>,
    pub(crate) graph: DiGraph<String, ()>,
    pub(crate) nodes: HashMap<String, NodeIndex>,
}

impl<'rule> GrammarGraph<'rule> {
    pub fn check_unused<S: AsRef<str>>(&self, start: S) -> crate::error::Result<&Self> {
        let all_nts = self.nodes.keys().collect::<HashSet<_>>();
        // find the reachable nodes for a given start symbol
        let start = self
            .nodes
            .get(start.as_ref())
            .expect("The start symbol does not exist");

        let mut dfs = Dfs::new(&self.graph, *start);
        let mut reachable = HashSet::new();
        while let Some(nx) = dfs.next(&self.graph) {
            let name = &self.graph[nx];
            reachable.insert(name);
        }
        let unreachable = all_nts.difference(&reachable).collect::<HashSet<_>>();
        // find the unreachable spans
        if !unreachable.is_empty() {
            let spans = self
                .rules
                .iter()
                .filter(|rule| unreachable.contains(&&rule.name))
                .map(|rule| rule.span)
                .collect::<Vec<_>>();
            return Err(Error::UnreachableRules { spans });
        }
        Ok(self)
    }

    pub fn check_trap_loop(&self) -> crate::error::Result<&Self> {
        let sccs = petgraph::algo::tarjan_scc(&self.graph);
        for scc in sccs {
            if self.is_trap_loop(&scc) {
                let spans = scc
                    .iter()
                    .map(|nx| {
                        self.rules
                            .iter()
                            .find(|rule| rule.name == self.graph[*nx])
                            .unwrap()
                            .span
                    })
                    .collect::<Vec<_>>();
                return Err(Error::TrapLoop { spans });
            }
        }
        Ok(&self)
    }

    fn is_trap_loop(&self, scc: &Vec<NodeIndex>) -> bool {
        let produce_t = scc.iter().map(|nx| self.graph[*nx].as_str()).any(|name| {
            // check if rule produce a terminal
            self.rules
                .iter()
                .find(|rule| rule.name == name)
                .unwrap()
                .produce_terminals()
        });
        if produce_t {
            return false;
        }
        let out_deg: HashSet<NodeIndex> = scc
            .iter()
            .map(|nx| self.graph.neighbors(*nx))
            .flatten()
            .collect();
        out_deg == scc.iter().map(|n| *n).collect()
    }
}

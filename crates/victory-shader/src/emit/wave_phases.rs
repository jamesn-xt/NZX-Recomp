//! Rendezvous sites for a workgroup of independently scheduled guest waves.

use std::collections::HashSet;

use crate::{ShaderError, ir};

type Cursor = (usize, usize);

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Phase {
    pub(super) barrier: usize,
    pub(super) first_back_edge: Option<usize>,
}

/// Find a common sequence of original barrier addresses before emitting a
/// workgroup scheduler. Follow both branch edges and stop each path at its
/// first barrier. Paths within a phase can loop without advancing the phase.
pub(super) fn plan(program: &ir::Program) -> Result<Vec<Phase>, ShaderError> {
    let mut phases = Vec::new();
    let mut released = HashSet::new();
    let mut pending = vec![(0, 0)];
    loop {
        let mut visited: HashSet<Cursor> = HashSet::new();
        let mut arrivals = Vec::new();
        let mut barrier = None;
        let mut ended = false;
        let mut first_back_edge = None::<usize>;
        while let Some((block, start)) = pending.pop() {
            if !visited.insert((block, start)) {
                continue;
            }
            let current = &program.blocks[block];
            if let Some((index, inst)) = current
                .insts
                .iter()
                .enumerate()
                .skip(start)
                .find(|(_, inst)| inst.name() == "s_barrier")
            {
                let site = inst.machine.offset;
                if ended || barrier.is_some_and(|other| site != other) {
                    return Err(unsupported(
                        site,
                        "guest paths rendezvous at different group barrier sites",
                    ));
                }
                barrier = Some(site);
                arrivals.push((block, index + 1));
                continue;
            }

            match current.exit {
                ir::Exit::End => {
                    if let Some(site) = barrier {
                        return Err(unsupported(
                            site,
                            "a guest wave may exit before the group barrier",
                        ));
                    }
                    ended = true;
                }
                ir::Exit::Indirect => {
                    return Err(unsupported(
                        current.offset,
                        "an indirect guest branch in a group barrier phase",
                    ));
                }
                exit => {
                    let offset = current
                        .insts
                        .last()
                        .map_or(current.offset, |inst| inst.machine.offset);
                    for next in exit.successors() {
                        if program.blocks[next].offset <= offset {
                            first_back_edge = Some(
                                first_back_edge.map_or(offset, |earliest| earliest.min(offset)),
                            );
                        }
                        pending.push((next, 0));
                    }
                }
            }
        }
        let Some(site) = barrier else {
            return Ok(phases);
        };
        if !released.insert(site) {
            return Err(unsupported(
                site,
                "a guest group barrier repeats across phases",
            ));
        }
        phases.push(Phase {
            barrier: site,
            first_back_edge,
        });
        pending = arrivals;
    }
}

pub(super) fn unsupported(offset: usize, what: &'static str) -> ShaderError {
    ShaderError::Unsupported { offset, what }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan_words(words: &[u32]) -> Result<Vec<Phase>, ShaderError> {
        let bytes: Vec<_> = words.iter().flat_map(|word| word.to_le_bytes()).collect();
        plan(&ir::build(&bytes, true, 1).unwrap())
    }

    #[test]
    fn wave_local_loop_finishes_before_the_same_original_barrier() {
        let phases = plan_words(&[
            0xBF8A_0000, // s_barrier
            0x7D84_0080, // v_cmp_eq_u32 vcc, 0, v0
            0xBF86_FFFE, // s_cbranch_vccz back to the compare
            0xBF8A_0000, // s_barrier
            0xBF81_0000, // s_endpgm
        ])
        .unwrap();
        assert_eq!(
            phases,
            [
                Phase {
                    barrier: 0,
                    first_back_edge: None
                },
                Phase {
                    barrier: 12,
                    first_back_edge: Some(8)
                },
            ]
        );
    }

    #[test]
    fn different_barrier_addresses_do_not_form_a_rendezvous() {
        let error = plan_words(&[
            0xBF06_0100, // s_cmp_eq_u32 s0, s1
            0xBF85_0001, // s_cbranch_scc1 to the second barrier
            0xBF8A_0000, // s_barrier
            0xBF8A_0000, // s_barrier
            0xBF81_0000, // s_endpgm
        ])
        .unwrap_err();
        assert!(matches!(
            error,
            ShaderError::Unsupported {
                what: "guest paths rendezvous at different group barrier sites",
                ..
            }
        ));
    }
}

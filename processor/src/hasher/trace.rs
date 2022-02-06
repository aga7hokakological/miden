use super::{Felt, FieldElement, HasherState, Selectors, TraceFragment, TRACE_WIDTH};
use vm_core::hasher::{apply_round, NUM_ROUNDS, STATE_WIDTH};

// HASHER TRACE
// ================================================================================================

/// TODO: add docs
pub struct HasherTrace {
    selectors: [Vec<Felt>; 3],
    row_addr: Vec<Felt>,
    hasher_state: [Vec<Felt>; STATE_WIDTH],
    node_index: Vec<Felt>,
}

impl HasherTrace {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// TODO: add docs
    pub fn new() -> Self {
        let state = (0..STATE_WIDTH).map(|_| Vec::new()).collect::<Vec<_>>();

        Self {
            selectors: [Vec::new(), Vec::new(), Vec::new()],
            row_addr: Vec::new(),
            hasher_state: state.try_into().expect("failed to convert vector to array"),
            node_index: Vec::new(),
        }
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns current length of this execution trace.
    pub fn trace_len(&self) -> usize {
        self.row_addr.len()
    }

    /// Returns next row address. The address is equal to the current trace length.
    pub fn next_row_addr(&self) -> Felt {
        Felt::new(self.trace_len() as u64)
    }

    // TRACE MUTATORS
    // --------------------------------------------------------------------------------------------

    /// Appends 8 rows to the execution trace describing a single permutation of the hash function.
    ///
    /// TODO: add detail
    pub fn append_permutation(
        &mut self,
        state: &mut HasherState,
        init_selectors: Selectors,
        final_selectors: Selectors,
        init_index: Felt,
        rest_index: Felt,
    ) {
        self.append_row(init_selectors, state, init_index);

        let next_selectors = [Felt::ZERO, init_selectors[1], init_selectors[2]];
        for i in 0..NUM_ROUNDS - 1 {
            apply_round(state, i);
            self.append_row(next_selectors, state, rest_index);
        }

        apply_round(state, NUM_ROUNDS - 1);
        self.append_row(final_selectors, state, rest_index);
    }

    /// Appends a new row to the execution trace based on the supplied parameters.
    fn append_row(&mut self, selectors: Selectors, state: &HasherState, index: Felt) {
        self.row_addr.push(self.next_row_addr());
        for (trace_col, selector_val) in self.selectors.iter_mut().zip(selectors) {
            trace_col.push(selector_val);
        }
        for (trace_col, &state_val) in self.hasher_state.iter_mut().zip(state) {
            trace_col.push(state_val);
        }
        self.node_index.push(index);
    }

    // EXECUTION TRACE GENERATION
    // --------------------------------------------------------------------------------------------

    /// Fills the provide trace fragment with trace data from this bitwise helper instance.
    #[allow(dead_code)]
    pub fn fill_trace(self, trace: &mut TraceFragment) {
        // make sure fragment dimensions are consistent with the dimensions of this trace
        debug_assert_eq!(self.trace_len(), trace.len(), "inconsistent trace lengths");
        debug_assert_eq!(TRACE_WIDTH, trace.width(), "inconsistent trace widths");

        // collect all trace columns into a single vector
        let mut columns = Vec::new();
        self.selectors.into_iter().for_each(|c| columns.push(c));
        columns.push(self.row_addr);
        self.hasher_state.into_iter().for_each(|c| columns.push(c));
        columns.push(self.node_index);

        // copy trace into the fragment column-by-column
        // TODO: this can be parallelized to copy columns in multiple threads
        for (out_column, column) in trace.columns().zip(columns) {
            out_column.copy_from_slice(&column);
        }
    }
}

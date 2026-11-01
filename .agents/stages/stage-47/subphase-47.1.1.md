# Sub-Phase 47.1.1: Modern Dynamic Array & Lambda Functions

## Objective
Implement industrial-grade modern dynamic array formulas, dynamic array manipulation primitives, and first-class lexical closure evaluation in `excel-engine` (AetherCalc) conforming to commercial Excel / ECMA-376 parity.

## Scope of Functions
1. **Dynamic Array Generators & Reshaping:**
   - `SEQUENCE(rows, [columns = 1], [start = 1], [step = 1])`: dynamic matrix generation.
   - `RANDARRAY([rows = 1], [columns = 1], [min = 0], [max = 1], [whole_number = false])`: pseudo-random matrix generator with deterministic bounds.
   - `CHOOSEROWS(array, row_num1, [row_num2, ...])`: selective row extraction with negative indexing support.
   - `CHOOSECOLS(array, col_num1, [col_num2, ...])`: selective column extraction with negative indexing support.
   - `TOCOL(array, [ignore = 0], [scan_by_column = false])`: flatten array to column vector with blank/error filtering.
   - `TOROW(array, [ignore = 0], [scan_by_column = false])`: flatten array to row vector with blank/error filtering.
   - `TAKE(array, rows, [columns])`: contiguous subset extraction from start or end.
   - `DROP(array, rows, [columns])`: contiguous exclusion of rows/columns.
   - `EXPAND(array, rows, [columns], [pad_with = #N/A])`: dimension expansion with padding.
   - `SORTBY(array, by_array1, [order1 = 1], [by_array2, order2 = 1], ...)`: multi-tier secondary sorting.
2. **Enhanced Array Filtering & Deduplication:**
   - `FILTER(array, include, [if_empty = #CALC!])`: row-wise and column-wise filtering with dimension validation and empty fallback.
   - `UNIQUE(array, [by_col = false], [exactly_once = false])`: full support for column-wise deduplication and occurrence frequency filtering.
   - `SORT(array, [sort_index = 1], [sort_order = 1], [by_col = false])`: full column-wise and row-wise sorting.
3. **Advanced Exact, Wildcard & Binary Lookups:**
   - `XLOOKUP(lookup_value, lookup_array, return_array, [if_not_found], [match_mode = 0], [search_mode = 1])`: multi-dimensional return arrays (returning rows or columns), exact/wildcard/binary matches.
   - `XMATCH(lookup_value, lookup_array, [match_mode = 0], [search_mode = 1])`: 1-based index lookup with exact, approximate, wildcard, and binary search.
4. **First-Class Lexical Closures & Higher-Order Functions:**
   - `LET(name1, val1, ..., expr)`: multi-variable lexical scope binding supporting both values and named closures.
   - `LAMBDA(param1, ..., body)`: closure creation, stored in scope or invoked immediately.
   - `ISOMITTED(parameter)`: detects omitted optional parameters in closures.
   - `MAP(array1, [array2, ...], lambda)`: maps elements (or tuples of elements) across one or more arrays to lambda.
   - `REDUCE(initial_value, array, lambda)`: accumulates elements using binary lambda.
   - `SCAN(initial_value, array, lambda)`: generates running accumulation matrix.
   - `MAKEARRAY(rows, cols, lambda)`: constructs calculated array via `(row_idx, col_idx)` lambda.
   - `BYROW(array, lambda)`: applies unary lambda to each row vector.
   - `BYCOL(array, lambda)`: applies unary lambda to each column vector.

## Engineering Invariants (GEC-v7.0)
- `#![forbid(unsafe_code)]` enforced across all Rust crates.
- Zero panics, unwrap, or expect in non-test library code.
- Clean-room implementation of ECMA-376 and open formula evaluation semantics.
- Error code fidelity: `#CALC!`, `#VALUE!`, `#N/A`, `#REF!`, `#SPILL!`.
- 100% test pass rate across all Rust crates and frontend test suites.


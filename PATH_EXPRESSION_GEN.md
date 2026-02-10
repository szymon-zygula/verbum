# Path Expression Generation Binary

## Overview

The `path_expression_gen` binary generates random expressions and extracts abelianized path data for term rewriting system analysis.

## Usage

```bash
cargo run --bin path_expression_gen -- \
  -n <SIZE> \
  -v <VARIABLES> \
  -r <ARITIES_JSON> \
  -t <TRS_DIR> \
  -a <APPLICATIONS> \
  -o <OUTPUT_JSON>
```

## Arguments

- `-n, --size <SIZE>`: Expression size (number of nodes in the expression tree)
- `-v, --variables <VARIABLES>`: Variable count (maximum variable ID will be v-1)
- `-r, --arities <ARITIES>`: Path to JSON file with arities
- `-t, --trs <TRS>`: Path to directory containing TRS JSON files (language.json and trs.json)
- `-a, --applications <APPLICATIONS>`: Number of random rewrite applications
- `-o, --output <OUTPUT>`: Output JSON file path

## Example

```bash
cargo run --bin path_expression_gen -- \
  -n 15 \
  -v 3 \
  -r jsons/simple-math/arities.json \
  -t jsons/simple-math \
  -a 2 \
  -o output.json
```

## Process

The binary performs the following steps:

1. Loads the term rewriting system (TRS) and language from JSON
2. Loads arities from JSON
3. Generates a random expression E of specified size with up to v variables
4. Applies a random rewrite applications to E, creating E'
5. Creates an abelianized stringified matrix A for the TRS
6. Chooses a random variable k appearing in both E and E'
7. Extracts all paths from root to leaves that are variable k in both E and E'
8. Maps these paths to their abelianized versions (P_a and P'_a)
9. Saves P_a, P'_a, A, and metadata to a JSON file

## Output Format

The output JSON contains:

- `P_a`: Array of abelianized path vectors from E to variable k
- `P_a_prime`: Array of abelianized path vectors from E' to variable k
- `A`: Abelianized TRS matrix (rows = string language symbols, columns = induced rules)
- `k`: The chosen variable ID
- `E`: Original expression as string (for reference)
- `E_prime`: Rewritten expression as string (for reference)

## Input File Formats

### Arities JSON

```json
{
  "map": {
    "0": [2],
    "1": [1],
    ...
  }
}
```

Each key is a symbol ID, and the value is an array of allowed arities.

### TRS Directory

The TRS directory should contain:
- `language.json`: Language definition with symbols
- `trs.json`: Rewrite rules

See `jsons/simple-math/` for examples.

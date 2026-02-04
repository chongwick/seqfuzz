# SeqFuzz

**A Blackbox IR-based PHP Fuzzer**

SeqFuzz is a blackbox fuzzer specifically designed for the **PHP interpreter**.  
It discovers bugs by performing mutations on a custom intermediate representation called **SeqIR**, rather than mutating raw PHP source code.

## What is SeqFuzz?

SeqFuzz fuzzes the PHP interpreter by:

- Maintaining a corpus of programs represented in **SeqIR**
- Applying structured mutations to SeqIR sequences
- Converting mutated SeqIR back to valid PHP source code
- Executing the generated PHP code in the target interpreter
- Observing for crashes, assertion failures, memory errors, or other anomalous behavior

## What is SeqIR?

**SeqIR** (Sequence IR) is a custom **sequence-based intermediate representation** created specifically for fuzzing PHP.

### How SeqIR is generated

SeqIR is produced by:

1. Parsing PHP source code into an **Abstract Syntax Tree (AST)**
2. Performing a **post-order traversal** of the AST
3. Translating each AST node into one or more **SeqIR nodes**
4. Arranging nodes into hierarchical sequences.

### Node Types

SeqIR consists of exactly two kinds of nodes:

- **Control nodes**  
  Represent flow-altering constructs (conditionals, loops, function calls, etc.)

- **Data nodes**  
  Represent value-producing or value-modifying operations (literals, variables, arithmetic, array operations, etc.)

Nodes are annotated with their data/structural dependencies, enabling semantically safe splicing and forming an internal def-use chain. 

### Hierarchical Sequences

Nodes are arranged into hierarchical sequences where each sequence represents a scoping level. Sequences have the following characteristics:

1. Each sequence (except the primary sequence) begins with a control node, indicating its execution condition. 
2. Nodes in every sequence are visible to children sequences. 
3. Sequences are partially ordered with the only constraint being a starting control node (except the primary sequence)

### How SeqIR is mutated

The sequential structure and node annotations enable maximum flexibility with mutation operators on SeqIR. Node values can be altered, subsequences can be spliced, and nodes can be spliced. This enables fine- and coarse- grained mutations.

### How SeqIR is lifted to PHP

SeqIR nodes are visited sequentially. Nodes dependencies are recursively printed to prevent undefined references.

## Requirements

- PHP 7.4+
- A build of PHP compiled with debug symbols / sanitizers recommended for best bug detection

## Installation
# Prepare the starting corpus
python prepare.py

# Convert the starting corpus into SeqIR
python preload.py

# Build PHP with desired options

# Run fuzzer
python fuzzer.py


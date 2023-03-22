# Data formatter concept in WASM

This is a 'concept' repo for a 'schema agnostic' package scanner and formatter built entirely in Rust -> WASM,
and built for interactions with JS. It naively copies a JSON-style nested structure, copies it, and passes all
non-iterable values through a custom callback defined by the user.

Most of this readme is about the findings after building this.

TL;DR (whole document)

While it does the same job as its JS counterpart, its not as performant. For small-scale tests, the library proved
to be about 1.1 milliseconds slower than JS code which does the same thing. The more complex the input, the larger
the time disparity.

## Features

- Scans an entire data package of arrays and objects
- If the scanner detects an array or an object, then it iterates over its children
- If the scanner encounters a non-iterable or string, then it passes the value to a 
javascript callback and adds its returned value to the data package
- Data types do *not* need to be consistent within the structure
- There's *no* recursion limit, so the depth for this library is theoretically infinite

## Examples

### Example: simple string

    const input = "test"

    callback = (item) => `${item}_SUCCESSFUL`

    const result = wasm.get_data_modified( input, callback )

    /*
    result = "test_SUCCESSFUL
    */

### Example: object with array children

    const input = {
      "array_0" : [ "A", "B", "C" ],
      "array_1" : [ "D", "E", "F" ],
      "array_2" : [ "G", "H", "I" ],
    }
    
    callback = (item) => `${item}_SUCCESSFUL`

    const result = wasm.get_data_modified( input, callback )

    /* 
    result = {
      "array_0" : [ "A_SUCCESSFUL", "B_SUCCESSFUL", "C_SUCCESSFUL" ],
      "array_1" : [ "D_SUCCESSFUL", "E_SUCCESSFUL", "F_SUCCESSFUL" ],
      "array_2" : [ "G_SUCCESSFUL", "H_SUCCESSFUL", "I_SUCCESSFUL" ],
    }
    */

## Why isn't this a crate?

Its slower than the competition. See below for analysis.

## Performance notes

Benchmark used: window.performance

For small data sets, this noticeably takes about one millisecond compared to its javascript counterpart. (0.9 - 1.1)

The JS equivalent library's execution time rounds down to 0 in all test cases.

In large scale cases, multiple nested objects and arrays.
WASM: 3.5
JS: 1

Notable fluctuations:
In cases with small data sets, initial page loads registered 3.5-4 millisecond execution times. Subsequent page loads
immediately upon refresh dropped to 1.1.

These fluctuations happened proportionately with the size of the inputs used.

At no point during the tests did the JS execution times exceed 1 millisecond.

## Optimizations

All benchmarks recorded here are with the '-release' binary output and all mentioned optimizations already included.

See cargo for detailed compiler optimizations, plus comments one what seemed to extend execution times.

Removed 'wee_alloc' in favor of Rust's default allocator. This helped with getting reliable execution times, but
didn't lead to an overall decrease in execution time.

Switching HashMap libraries shaved off about 0.4 off the execution time (down from 1.5).

Removed 'wee_alloc' in favor of Rust's native allocator. This didn't so much 'speed up the library' as much as it cut
the longer-execution time outliers.

In place of Rust's std HashMap, this library uses rustc_hash. There are higher risks of collisions, but in a mostly
normal JSON tree, this shouldn't be a major issue.
https://crates.io/crates/rustc-hash

To further exploit this library's efficiency, the Node struct makes heavy use of enums to support u32 keys where-ever
possible.

## Performance notes: likely bottlenecks

The strongest candidate for bottlenecks here is the full data structure conversion, followed by re-packing all the 
data back into the root JsValue.

After optimizing most other operations, this leaves the serialization and JS function calls as the two most likely
contributors.

The library makes 'Reflect' calls when creating an object to convert into a JsValue. Forum discussions report this is
a 'time-sink' call, however other call types either require structs with static fields, or at the very least, won't
tolerate HashMaps with JsValues. This would mean the library would need to convert these values to Rust-data types,
which would essentially be replacing one expensive operation with more than one smaller operations, while also 
increasing the potential for conversion errors.

Even with performance gains from these changes, they would need to get the execution times at or below JS' in order
for these gains to count as a 'win'.

## Workflow

This library uses the 'iterative/stack' approach to traversing the data. Full recursion is avoided due to the hard 
limits. While these limits can always increase, they're never 'gone.'

Every JsValue within the data structure leads to the creation of a Node.

Definition for 'iterable': A value is considered 'iterable' if it is an Array or and Object. All other data types, 
including Strings, and considered 'not-iterable.' A Node that does not contain an Array or and Object will *not* have 
children. Maps are ignored in this library since they're not commonly found in general JSON-syle structures.

Each iteration works in LIFO order, always taking the right-most Node off the stack. If a Node has children, and hasn't 
already been processed, then it will add those children to the stack. If the right most child also has children, then 
those are added to the stack in the subsequent iteration.

This continues until a node without children is detected. Once detected, the library grabs the value from the child. 
If the child doesn't have any children of its own, then it will pass its stored value into the callback before updating 
its parent. Otherwise, its will do a straight transfer.

The library finds the child's parent by iterating backwards through the stack and checking for the first node from the 
next layer up who also reports that its children were 'added to the stack.'

Definition for 'layer': Each node added to the stack is assigned a layer that is +1 compared to its parent. 
(Example: the root node always has layer 0. Its immediate children are assigned layer 1.)

This approach gets around Rust's mutable reference restrictions, as well as navigates around issues where not all 
children have children of their own. The library updates the parent with the key stored in the child. If the child 
never had children, then its stored value is passed through the javascript callback before being passed to the parent. 
Following this sequence, the child is dropped from memory.

This process repeats itself until the stack is empty, and the last node (the root) reports that its children were 
already processed.

TL;DR

The stack uses FIFO, and some iterative searching to pass values up from the lowest-most children until all nodes have 
been processed.

## Conclusions

Obviously, this is a library which makes heavy use of JS <-> Rust interoperability, but this is also sort a 'reality
check' for WASM performance: The conversions come with a cost. Compiled code would need to disproportionately offset
any conversion costs to justify its use within browsers.

While I wouldn't discount the potential benefit of this technology in the future, as of this writing, any potential 
WASM gains are still largely marginal at best and limit the potential business use for this tool.

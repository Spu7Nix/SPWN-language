# Register Allocation Optimisation

An interference graph is a data structure used for determining the interference relationship between variables in a program.
It can identify which variables cannot be assigned to the same register simultaneously, thereby guiding register allocation.

Given the code below:

```py
a = 2
b = 40
c = a
d = 42
c += b
print c
```

It uses 3 registers, which within the interference graph looks like:

<p align="center">
<img src="https://cdn.discordapp.com/attachments/1017864804561588246/1118626262425026570/Frame_2.png" height="150px" style="object-fit: contain" alt="interference graph example"></p>

-   `a` interferes with `b` because the declaration of `b` occurs before `c = a`, and `b` is used later, so the value of b must not be replaced
-   `c` interferes with `b` due to the plus equals operation which requires that `b` is still available after `c` is declared
-   `b` interferes with `d` othwerwise the assignment would replace `b` which is used later
-   `c` interferes with `d` othwerwise the assignment would replace `b` which is used later

Once a graph colouring algorithm is applied to the interference graph, the code would simplify to:

```py
a = 2
b = 40
c = 42
a += b
print a
```

In order to actually construct the intereference graph, there are some prerequisites we must first understand.

## `Use` and `Def` sets

The `Use` set contains every register that a single opcode _uses_ (any register the opcode reads from)
The `Def` set contains every register that a single single _defines_ (any register the opcode modifies / writes to)

For example:

```py
Opcode::Add { left: R0, right: R1, output: R2 }
```

The `Use` set would contain `{R1, R2}`.
The `Def` set would contain `{R2}`.

## Liveness

Liveness, refers to the lifetimes or ranges during which a registers hold meaningful values. It determines the points in a program where registers are used and where they becomes no longer needed.

If two registers are live at at the same time, that means they both have meaningful values, therefore you cannot use the same register for both of them.
Two registers that are _not_ live at the same time can share a register.

For every opcode in the program, we can analyse liveness immediately before and after it. These are named:

-   Live in
-   Live out

### Live in

The `LiveIn` set is defined as the set of registers that are live _in_ to an opcode. "In" is refering to _before_ the opcode has been executed.

### Live out

The `LiveOut` set is defined as the set of registers that are live _out_ of an opcode. "Out" is refering to _after_ the opcode has been executed.

For example:

```py
#       |
#       |
#       V  <- Going in to the opcode
Opcode::Add { ... }
#       |  <- Coming out of the opcode
#       |
#       V  <- Going in to the opcode
Opcode::Out { ... }
```

As these sets just represent liveness before and after opcodes, the `LiveIn` set of the second opcode is the same as the `LiveOut` set of the first.

Constructing the `LiveIn` set requires a bit of work:

## Control Flow Graph

A control flow graph (CFG) is a representation of the flow of control in a program. It shows how the program's execution moves from one instruction to another, capturing the control flow paths and decision points.

Each opcode has arrows to its successors. Branches, such as conditional jumps (`if`), will have multiple arrows to each of the possible successive opcodes. Others, like `return`, will have no successors.

The CFG will allow us to easily get the successor of each opcode, which is important for constructing the `LiveIn` and `LiveOut` sets.

<p align="center">
<img src="https://cdn.discordapp.com/attachments/1017864804561588246/1118617078748946583/gagagaga.png" height="300px" style="object-fit: contain" alt="control flow graph example">
<br>
<i>Example of a CFG</i>
</p>

## Tree Traversal of Opcodes

To compute the liveness of registers we must visit every opcode in the program. The order in which the opcodes are visited does not affect the final result of the algorithm. However, some algorithms make it faster to compute the liveness.
The algorithm _we_ chose is depth-first post-order traversal:

<p align="center">
<img src="https://cdn.discordapp.com/attachments/1017864804561588246/1118620295805272104/fsdgsdfgsdfg.png" height="300px" style="object-fit: contain" alt="tree traversal example">
<br>
<i>Starting at the square, follow the line and visit only the blue nodes. I.E. <code>A, C, E, D, B, H, I, G, F</code></i>
</p>

An advantage to this algorithm is that it follows the control flow of the normal program. Information flows from beginning to end, and post-order traversal mimics that flow, allowing the liveness to propogate correctly.
This ensures that when we encounter a definition of a variable, we have already visited all the uses of that variable in subsequent nodes.

Using the same image as above, if we consider node (opcode) `A` as a definition, we know there are no uses of the register contained within as there are no child nodes of it. On the other hand, if we consider node (opcode) `B` as a defintion, both `E` and `D` could be uses of the register in `B`, therefore we would have visited the uses _before_ the definition.

## Constructing the `LiveIn` and `LiveOut` set

Traversing the CFG in post-order, for each opcode we visit, we:

-   Calculate the `LiveOut` set:

    -   Loop over all the successors and over all the registers within their `LiveIn` sets.
    -   Insert every register into our `LiveOut` set.

    For the very bottom nodes, this loop will not run as they will not have any successors. If an opcode later in the program uses a currently live register (I.E. it is live _in_ to that opcode) that means it is going to be live _out_ of the opcode too.

-   Following that, we compute the `LiveIn` set:
    -   Get the `Use` set for the current opcode and make that the start of our `LiveIn` set.
    -   Loop over every register in the `LiveOut` set and add it to our `LiveIn` set if it's _not_ contained withing the `Def` set of the current opcode.

All of the operations performed can be defined in the following equations:

<p align="center">
<img src="https://cdn.discordapp.com/attachments/1017864804561588246/1118625499187531847/sfgdsfvx_x_c_fdgdffdgfdg.png" height="100px" style="object-fit: contain" alt="set theory for liveness">
<br>
<i><code>Succs</code> is the successors of the current opcode</i>
</p>

## Graph Colouring

The final step is to apply a graph colouring algorithm to the interference graph. This will allow us to optimally pick new registers for each of the old registers.

To colour the graph, we take at most N colours and apply the colours to each node such that no two nodes connected by an edge have the same colour.

Going back to the original interference graph, if we apply a graph colouring algorithm to it, it will look like:

<p align="center">
<img src="https://cdn.discordapp.com/attachments/1017864804561588246/1118625884434346014/Frame_2.png" height="150px" style="object-fit: contain" alt="graph colouring example"></p>

Each colour represents a new register. We can see that it has been optimised from 4 registers down to 3, as `A` and `C` share a register as they do not interfere with each other. Although this is a small example, on a bigger scale this will optimise far more, especially with multiple passes.

### Resources:

-   https://www.cse.chalmers.se/edu/year/2014/course/TDA282_Compiler_Construction/lect07-2x2.pdf
-   https://www.cs.york.ac.uk/fp/cgo/lectures/chapter5.pdf
-   https://www.cs.york.ac.uk/fp/cgo/lectures/chapter6.pdf
-   https://users.cs.northwestern.edu/~simonec/files/Teaching/CC/slides/Interference_graph.pdf

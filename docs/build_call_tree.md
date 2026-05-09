# build_call_tree in DFS
learn the power of algorithm in real development
build a map between classic algorithm and specific development demand

## model
a very classic DFS model
```text
A -> B -> C -> E
     |
      -> D 

A -> B
B -> C
C -> E
B -> D

A depth 0
B depth 1
C depth 2
E depth 3
D depth 2

enter A
  enter B
    enter C
      enter E
      exit E
    exit C
    enter D
    exit D
  exit B
exit A

```

```rust
fn build_call_tree(calls: &[CallTrace]) -> Vec<CallTreeNode> {
    let mut roots = Vec::new();
    let mut stack: Vec<CallTreeNode> = Vec::new();

    for call in calls {
        while stack.len() > call.depth {
            flush_call_tree_node(&mut stack, &mut roots);
        }

        stack.push(call_tree_node(call));
    }

    while !stack.is_empty() {
        flush_call_tree_node(&mut stack, &mut roots);
    }

    roots
}

fn flush_call_tree_node(stack: &mut Vec<CallTreeNode>, roots: &mut Vec<CallTreeNode>) {
    let Some(node) = stack.pop() else {
        return;
    };

    if let Some(parent) = stack.last_mut() {
        parent.children.push(node);
    } else {
        roots.push(node);
    }
}
```
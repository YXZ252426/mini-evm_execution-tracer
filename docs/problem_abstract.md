这是很关键的一步。你刚才做的不是“会写 Rust”，而是把问题抽象成了一个通用模型：

```text
事件流 + depth 信息 -> 树
```

这比记住某个 API 重要得多。

你可以这样继续练。

**1. 每次先问：这是哪类数据结构问题**

比如 call tree 不是 EVM 独有问题，它本质是：

```text
preorder sequence with depth -> tree
```

类似问题还有：

```text
HTML tokens -> DOM tree
Markdown headings -> document outline
function enter/exit logs -> flame graph
filesystem paths -> directory tree
trace spans -> span tree
```

你要训练自己看到具体业务时，主动问一句：

```text
这个问题是不是某个经典结构？
```

**2. 把代码前先写出状态机**

比如 call tree 可以先不写 Rust，只写：

```text
state:
  roots
  stack

on new node:
  while stack.len > node.depth:
    pop child
    attach to parent or roots

  push node

finish:
  flush stack
```

这一步非常重要。能写出状态机，代码通常只是翻译。

以后遇到复杂逻辑，先写：

```text
输入是什么？
输出是什么？
中间状态是什么？
每个事件发生时状态怎么变？
什么时候收尾？
```

**3. 练“最小反例”**

不要只用简单 case：

```text
A
  B
```

要刻意构造边界：

```text
A
  B
  C
D
```

```text
A
  B
    C
      E
    D
```

```text
A
B
C
```

```text
A
  B
    C
```

每个算法你都手推 3 到 5 个例子。能过这些例子，你才真正理解了。

**4. 建自己的模式库**

你可以开一个 `notes/patterns.md`，每次总结一个模式：

```md
## Preorder + depth -> tree

Use when:
- input is a flat sequence
- each item has depth/level
- output is nested tree

State:
- roots
- stack

Algorithm:
- pop while stack.len > depth
- attach popped node to parent
- push current node
- flush stack at end

Examples:
- EVM call trace
- Markdown headings
- file tree
```

你以后会越来越快，因为不是每次从零思考。

**5. 刻意做跨领域迁移**

现在你学的是 call tree。马上可以用同一个算法做两个小练习：

Markdown heading tree：

```text
# A
## B
### C
## D
# E
```

转成：

```text
A
  B
    C
  D
E
```

文件路径 tree：

```text
src/main.rs
src/executor.rs
examples/sstore.hex
```

转成目录树。

同一个算法在不同领域跑通，你就会真正内化它。

**6. 读源码时找“模型”，不是找函数名**

比如你之前困惑：

```rust
interp.bytecode.opcode()
```

实战中确实不可能一开始知道所有 API。正确方法是先定位模型：

```text
我要的是当前执行指令
当前执行指令应该属于 interpreter state
interpreter state 里应该有 bytecode / pc / gas / stack / memory
```

然后搜：

```bash
rg "fn opcode"
rg "pc(&self)"
rg "struct Interpreter"
```

你不是在背 API，而是在根据模型缩小搜索范围。

**7. 复盘时用这张表**

每做完一个功能，写 5 行：

```text
具体问题：EVM call trace 要输出树
抽象模型：DFS preorder + depth -> tree
输入：Vec<CallTrace>
状态：stack + roots
关键不变量：stack 表示当前还未挂载完成的路径
边界：同级节点、回到根节点、最后 flush
```

这张表会逼你从“我写出来了”升级到“我知道它为什么对”。

下一步建议你亲手实现一个小函数：

```rust
fn headings_to_tree(headings: Vec<(usize, String)>) -> Vec<Node>
```

不要看现有代码，自己从状态机写。写完后再和 call tree 对照。这个练习很值。
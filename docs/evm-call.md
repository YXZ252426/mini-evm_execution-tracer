# EVM Call Notes
- [official opcode defination and playground excution of CREATE and CALL](https://www.evm.codes/#:~:text=F1-,CALL,-100)
## 1. Call 的核心概念

EVM 里的 `CALL` 不是普通语言里的函数调用。

它更接近：

```text
当前执行中的合约
  -> 请求 EVM 创建一个新的执行 frame
  -> 在另一个账户地址的代码上下文里继续执行
  -> 执行结束后把结果返回给上一个 frame
```

所以 EVM 执行不是单层线性流程，而是一棵 call tree。

最外层交易本身也可以看成一个 root call：

```text
EOA(from) -> Contract(to)
```

如果这个合约内部又执行 `CALL`，就会多一层：

```text
depth 0: EOA      -> Contract A
depth 1: Contract A -> Contract B
depth 2: Contract B -> Contract C
```

我们现在 Milestone E 第一版记录的是扁平 call list，不是树。

## 2. Call Frame

每一次 call 都会创建一个新的 call frame。

一个 frame 可以理解为一次独立执行上下文，里面有：

```text
caller
callee / target
call value
input calldata
gas limit
return data
stack
memory
program counter
current bytecode
```

注意：每个 frame 有自己的 stack 和 memory。

Storage 不属于 frame。Storage 属于账户地址。

## 3. 常见 Call Opcode

### CALL

普通外部调用。

```text
Contract A CALL Contract B
```

特点：

- 会切换到 B 的代码执行
- `msg.sender` 是 A
- `msg.value` 是 A 传过去的 value
- storage 修改的是 B 的 storage

### STATICCALL

只读调用。

特点：

- 不允许修改 state
- 不能 `SSTORE`
- 不能发 log
- 常用于 view/pure 风格调用

### DELEGATECALL

用别人的代码，改自己的 storage。

```text
Contract A DELEGATECALL Contract Library
```

特点：

- 执行的是 Library 的 bytecode
- `msg.sender` 保持为 A 的 caller
- `msg.value` 保持原值
- storage 修改的是 A 的 storage

这就是 proxy 合约、upgradeable contract 的基础。

### CALLCODE

老版本机制，现代项目基本不用。

可以粗略理解成 `DELEGATECALL` 的历史前身。

## 4. from / to / caller / target

容易混淆的几个词：

```text
tx.from
tx.to
call.caller
call.target
```

例子：

```text
EOA Alice -> Contract A -> Contract B
```

交易层：

```text
tx.from = Alice
tx.to   = Contract A
```

root call：

```text
caller = Alice
target = Contract A
depth  = 0
```

internal call：

```text
caller = Contract A
target = Contract B
depth  = 1
```

所以 `tx.from` 只在交易入口固定一次，但 `caller` 会随着每层 call 改变。

## 5. Gas 在 Call 里的含义

每个 call frame 都有自己的 gas limit。

父 frame 调子 frame 时，会给子 frame 分配一部分 gas：

```text
Contract A has gas
  -> gives some gas to Contract B
  -> B uses gas
  -> unused gas returns to A
```

如果子调用失败，不一定代表整个交易失败。

合约代码可以选择：

```solidity
(bool ok, bytes memory ret) = target.call(data);
```

如果 `ok == false`，合约可以：

- revert 整个交易
- 忽略失败继续执行
- 根据返回值走其他逻辑

所以 tracer 里每个 call 都需要单独记录 `success`。

## 6. value 在 Call 里的含义

`CALL` 可以携带 ETH value。

```text
Contract A CALL Contract B with 1 ETH
```

这会尝试把 balance 从 A 转给 B。

如果 A balance 不足，call 会失败。

`DELEGATECALL` 不会真的转 value，它使用 apparent value。

## 7. Calldata 和 Returndata

每个 call 都有 input calldata。

例如 Solidity 函数调用：

```solidity
transfer(address,uint256)
```

底层会变成：

```text
4-byte selector + ABI encoded args
```

call 执行后会产生 return data。

第一版 call trace 通常先记录 input，不急着记录 output。后面可以扩展：

```json
{
  "input": "0x...",
  "output": "0x..."
}
```

## 8. 和当前 MiniTracer 的关系

我们现在的 tracer 用 revm 的 inspector hook：

```rust
fn call(&mut self, context, inputs) -> Option<CallOutcome>
fn call_end(&mut self, context, inputs, outcome)
```

执行顺序大概是：

```text
revm 准备进入 call frame
  -> Inspector::call(...)
  -> 执行子 frame
  -> Inspector::call_end(...)
```

所以：

```text
call()     适合记录 from / to / value / input / gas_limit
call_end() 适合回填 success / gas_used
```

当前扁平 call trace 示例：

```json
[
  {
    "depth": 0,
    "kind": "CALL",
    "from": "0x1000...",
    "to": "0x2000...",
    "value": "0x0",
    "input": "0x",
    "gas_limit": 29979000,
    "success": true,
    "gas_used": 22106
  }
]
```

## 9. 为什么后面要升级成树

扁平 list 能看出发生了哪些 call，但不方便看父子关系。

树结构更接近真实执行：

```json
{
  "depth": 0,
  "from": "Alice",
  "to": "A",
  "children": [
    {
      "depth": 1,
      "from": "A",
      "to": "B",
      "children": []
    }
  ]
}
```

实现方式：

```text
call()     -> push frame
call_end() -> pop frame, attach to parent.children
```

## 10. 一句话总结

EVM call trace 记录的不是“函数调用栈”，而是“账户之间创建执行 frame 的过程”。

理解 call trace 的关键是：

```text
每一层 call = 一个新的执行 frame
每个 frame 有自己的 stack/memory/gas
storage 属于账户地址
CALL 改 callee storage
DELEGATECALL 用 callee code 改 caller storage
```

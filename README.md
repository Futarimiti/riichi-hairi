# Japanese Mahjong Theory

日麻牌理分析器。[![996.icu](https://img.shields.io/badge/link-996.icu-red.svg)](https://996.icu)

用来练手Rust的项目。

## 重要更新日志

##### Ver 1.17 2020/4/11

* 完全重构代码
* 完成预想中的所有功能

##### Ver 1.05 2020/3/1

* 实现少量命令行启动参数和交互式命令。
* 实现json输出模式，以作为后端使用。
* 支持三麻。
* 修复8z9z不报错的问题。

##### Ver 1.0 2020/2/26

* 基本解决BUG，大量牌谱分析结果均与天凤牌理一致。

##### Ver 0.91 2020/2/25

* 第一个能够算得上是程序的版本。
* 完成基本的牌理功能，支持任意3*k+2的输入。

## 编译

因为使用了 bindings_after_at 特性，该特性在 1.54.0 之后才 stable，因此要求使用大于该版本的 rust。

然后 `cargo build --release` 就完事了。

## 使用

打开程序后输入牌谱即可，按照约定俗称的缩写：

* `m`->万子
* `p`->饼子
* `s`->索子
* `z`->字牌

作为扩展，允许使用`[]`表示副露的牌，这些牌的数量会从听牌数中减掉。

#### 输入样例

* 比较标准的形式：`1m2m3m5m9m9m2p2p4s5s1z[5z5z5z]`
* 省略多余的标记：`123599m22p45s1z[555z]`
* 空格将会被无视：`123599m 22p 45s 1z [555z]`
* 3*k+2不包含副露，可以加入杠：`123599m 22p 45s 1z [5555z]`
* 输入顺序可以随便：`99m2p [5555z] 1z12m 2p45s35m`

#### 命令行启动参数

* `-V`,`--version` 打印版本信息
* `-h`,`--help` 打印启动参数列表
* `-i`,`--interactive` 以交互模式启动
* `-f=<type>`,`--format=<type>` 设置输出模式，现支持standard（标准模式，默认）和json（用于后端模式）。
* `-p=<num>`,`--player=<num>` 设置游戏人数为4（四麻，默认）或3（三麻），三麻缺少2~8万。

#### 可用命令

* `i`,`interactive` 进入交互模式。如果已经处于交互模式，则重新初始化。
* `ni`,`noninteractive` 退出交互模式，回到普通模式。
* `3pl`,`3-player`,`4pl`,`4-player` 切换四麻或三麻。交互模式下会重新初始化。
* `std`,`standard` 使用标准输出模式。
* `json` 使用json输出模式。
* `q`,`quit`,`exit` 退出程序。
* `h`,`help` 打印可用命令列表。

仅在交互模式下可用的命令：

* `+` 摸一张牌，例如`+4m`。摸牌后会自动分析并输出牌理。
* `-` 从手牌中打出一张牌，例如`-1s`。
* `*+` 向牌山中增加任意张牌，用于纠正误操作。每种牌的牌山存量上限是4张（不计手牌）。
* `*-` 从牌山中移除任意张牌，可能是别家打出、副露，或者是翻出宝牌指示，或者是摸切时不想输入两次`+`和`-`等原因。例如`*-1s777z`。注意自家副露不需要写`*-`表示别家打出。
* `>` 表示吃、碰或杠。如果是吃，则默认将第三张牌视为上家舍牌，如`>465s`表示用自己的4条6条吃上家的5条。如果是杠，则需要摸岭上牌，可以先`>4444p`再`+5s`，也可以直接以`>4444p5s`表示。你无需把岭上牌放在最后，事实上`>44p5s44p`也能被正常识别为杠4筒摸5索。注意大明杠，加杠，暗杠的区别（当手牌是13张时）：`>4444p`是大明杠，`+4p`再`>4444p`表示加杠或暗杠，具体是哪个由程序检测是否存在明刻决定。
* `b`,`back` 撤销上一次操作。程序会记录所有操作，你可以一直回退到任意过去的状态，以便于研究牌理。
* `s`,`state` 打印游戏状态，包含牌山，舍牌种类，手牌。
* `d`,`display` 通常，当操作后（不包含`back`、`state`操作）手牌数为14时，程序会打印出牌理分析结果。你也可以用`display`命令让程序再次打印牌理分析结果。
* `log`,`history` 打印所有操作历史。

任何时候，如果你的操作会导致牌山中某种牌存量低于0或大于4，该操作会失败，牌山和手牌会恢复到之前的状态，本次操作不被记录。但是，程序仍然提供一些命令可以无视牌山的报错，仍然执行操作。这些命令都带有`!`，它们可能破坏程序的稳定性：

* `+!` 无视牌山报错的`+`，当牌山中某种牌存量为0时，使用`+!`不会报错，牌山存量仍然保持0张。
* `-!` 你可以这么写，但是它和`-`是完全没有区别的。
* `*!+` 无视牌山报错的`*+`，当牌山中某种牌存量为4时，继续`*!+`不会报错，而是保持4张。注意，使用`back`回退该操作时总是会减少牌的数量。
* `*!-` 无视牌山报错的`*-`，当牌山中某种牌存量为0时，继续`*!-`不会报错，而是保持0张。注意，使用`back`回退该操作时总是会增加牌的数量。
* `>!` 不做边界检测的`>`。如果被吃/被碰/被杠的牌的山存量实际为0，不会报错并且仍然能吃/碰/杠成功。对于杠而言，岭上牌的数量也不做边界检测。如`>!555z`。
* `b!`,`back!` 当使用`back`回退上述带有`!`的操作时，仍然会视作不带`!`的版本操作并且重视牌山的报错，这可能会导致你回退失败。使用`b!`和`back!`则仍然无视牌山的报错（即使是回退不带`!`的操作），例如，如果山存量为4时回退`*-`或`*!-`，仍保持4张而不报错，如果山存量为0回退`*+`或`*!+`，则仍保持0张而不报错。

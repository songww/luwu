# 分布式事务管理服务

Luwu(陆吾)是首款Rust实现的开源分布式事务管理器，优雅的解决了幂等、空补偿、悬挂等分布式事务难题。提供了简单易用、高性能、易水平扩展的分布式事务解决方案。

## 亮点

* 极易接入
  - 支持HTTP，提供非常简单的接口，极大降低上手分布式事务的难度，新手也能快速接入
* 使用简单
  - 开发者不再担心悬挂、空补偿、幂等各类问题，框架层代为处理
* 跨语言
  - 可适合多语言栈的公司使用。方便 python、nodejs、C/C++ 各类语言使用。
* 易部署、易扩展
  - 仅依赖 PostgreSQL，部署简单，易集群化，易水平扩展
* 多种分布式事务协议支持
  - TCC、SAGA、XA、事务消息

## 与其他框架对比

目前开源的分布式事务框架，暂未看到非Java语言有成熟的框架。而Java语言的较多，有阿里的SEATA、华为的ServiceComb-Pack，京东的shardingsphere，以及himly，tcc-transaction，ByteTCC等等，其中以seata应用最为广泛。

下面是Luwu和seata的主要特性对比：

|  特性| Luwu | SEATA |备注|
|:-----:|:----:|:----:|:----:|
| 支持语言 |<span style="color:green">Rust、Python、C/C++及其他</span>|<span style="color:orange">Java</span>|Luwu 可轻松接入一门新语言|
|异常处理| <span style="color:green">[子事务屏障自动处理](https://zhuanlan.zhihu.com/p/388444465)</span>|<span style="color:orange">手动处理</span> |Luwu 解决了幂等、悬挂、空补偿|
| TCC事务| <span style="color:green">✓</span>|<span style="color:green">✓</span>||
| XA事务|<span style="color:green">✓</span>|<span style="color:green">✓</span>||
|AT事务|<span style="color:red">✗</span>|<span style="color:green">✓</span>|AT与XA类似，性能更好，但有脏回滚|
| SAGA事务 |<span style="color:orange">简单模式</span> |<span style="color:green">状态机复杂模式</span> |Luwu的状态机模式在规划中|
|事务消息|<span style="color:green">✓</span>|<span style="color:red">✗</span>|Luwu提供类似rocketmq的事务消息|
|通信协议|HTTP|dubbo等协议，无HTTP|Luwu后续将支持grpc类协议|

从上面对比的特性来看，如果您的语言栈包含了Java之外的语言，那么Luwu是您的首选。如果您的语言栈是Java，您也可以选择接入Luwu，使用子事务屏障技术，简化您的业务编写。

## [各语言客户端及示例](./doc/sdk.md)

## 快速开始

### 安装

`cargo install luwu`

### Luwu 依赖于 PostgreSQL

配置 PostgreSQL：

TODO ``

### 启动并运行saga示例
`luwu saga`

## 开始使用

### 使用
```Python
  # 具体业务微服务地址
  callback = "http://localhost:8081/api/saga"
  payload = {"amount": 30} # 微服务的载荷
  # luwu 为 Luwu 服务的地址
  luwu = Luwu("http://127.0.0.1:3000/api/")
  saga = luwucli.Saga();
    # 添加一个TransOut的子事务，正向操作为url: qsBusi+"/TransOut"， 补偿操作为url: qsBusi+"/TransOutCompensate"
  saga.add(callback + "/TransOut", callback + "/TransOutCompensate", payload).
    # 添加一个TransIn的子事务，正向操作为url: qsBusi+"/TransIn"， 补偿操作为url: qsBusi+"/TransInCompensate"
  saga.add(callback + "/TransIn", callback + "/TransInCompensate", payload)
  // 提交saga事务，dtm会完成所有的子事务/回滚所有的子事务
  luwu.call(saga.submit())
```

成功运行后，可以看到TransOut、TransIn依次被调用，完成了整个分布式事务

### 时序图

上述saga分布式事务的时序图如下：

<img src="https://pic3.zhimg.com/80/v2-b7d98659093c399e182a0173a8e549ca_1440w.jpg" height=428 />

### 完整示例
参考[examples/simple/main.rs](./examples/simple/main.rs)

## 文档与介绍(更新中)
  * [分布式事务简介](https://zhuanlan.zhihu.com/p/387487859)
  * 分布式事务模式
    - [XA事务模式](https://zhuanlan.zhihu.com/p/384756957)
    - [SAGA事务模式](https://zhuanlan.zhihu.com/p/385594256)
    - [TCC事务模式](https://zhuanlan.zhihu.com/p/388357329)
    - 可靠消息事务模式
  * [子事务屏障](https://zhuanlan.zhihu.com/p/388444465)
  * [通信协议](./protocol.md)
  * [FAQ](./FAQ.md)
  * [Issue](https://github.com/songww/luwu/issues/1)
  * 部署指南


如果您觉得[luwu](https://github.com/songww/luwu)不错，或者对您有帮助，请赏颗星吧！

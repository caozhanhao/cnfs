# cnfs

[![Cargo Build & Test](https://github.com/caozhanhao/cnfs/actions/workflows/ci.yml/badge.svg)](https://github.com/caozhanhao/cnfs/actions/workflows/ci.yml)

A virtual file system for a task in CNSS Recruit 2024.

Documentation: https://cnfs.mkfs.tech

## Task

### 🧐 看看你的心

> 某次偶然，ssg 偷了 XuKaFy 的硬盘插入自己体内，他惊讶的发现无论插入哪个格式的硬盘都能精准识别 XuKaFy 的小簧片，他是怎么做到的呢？

#### 📚 题目描述

柳苏明要求你实现一个兼容不同文件系统的兼容层，具体要求如下：

- 应用对文件的读写直接使用该兼容层的接口即可实现；换言之，可以代替 Linux 下的`open`、`read`等用户层接口
- 该兼容层提供接口供文件系统注册，通过对这些接口的调用实现文件系统的功能
- 兼容层在内存中缓存各个文件系统的信息（文件节点、文件信息、目录树等）并组织在一起
- 兼容层对每个打开或者未打开的文件和文件夹在内存中用一个节点`struct inode`表示，该结构体记录了该节点的属性（如是否为文件夹）、目录项数组或者链表等
- 兼容层对每个目录项用一个结构体`struct dentry`表示，记录了这个目录项对应的文件或者目录的名称、关联的`struct inode`等属性
- 兼容层管理文件打开之后在内存中创建的结构抽象`struct file`
  ，该结构记录了文件用于在文件系统中进行读写的信息，并且需要包含一个指针，该指针指向包含文件节点信息的结构体。以上几个结构体都由你自己实现。

作为一个可能的规范，你的兼容层需要向用户层提供以下的接口：

- `int vopen(const char* path)`：**打开**相应的文件，在文件系统中找到该文件的位置并且注册相应的`struct file`
  ，返回该`struct file`的句柄/fd（你自己定义的定位这个结构的数或者指针）
- `void vclose(int fd)`：通过 fd **关闭**该文件，清理资源
- `int vread(int fd, size_t pos, size_t count, char* buffer)`：在文件的相应偏移**读**出相应大小的数据写入缓存
- `int vwrite(int fd, size_t pos, size_t count, char* buffer)`：从缓存中在文件的相应偏移**写**入相应大小的数据
- `int vmkdir(const char* path, const char* dname)`：在某个目录下**创建**一个给定名字的**目录**
- `int vrmdir(const char* path, const char* dname)`：在某个目录下**删除**一个给定名字的**目录**
- `int vcreat(const char* path, const char* fname)`：在某个目录下**创建**一个给定名字的**空文件**
- `int vremov(const char* path, const char* dname)`：在某个目录下**删除**一个给定名字的**文件**

此外，你还应该提供对文件系统进行**挂载**的接口`vmount`，用于在有新的文件系统加入时将该文件系统的根目录挂载在特定的空目录上（称为挂载点）。

你的兼容层应该能够在不同挂载点**同时挂载多个文件系统**，并能够在同时挂载多个文件系统的情况下，可以访问各个文件系统的文件。

对于额外`30%`的分数，由于内存总是比外设具备更强的 IO 性能，你的兼容层需要具有**文件内容的缓存功能**，具体而言：

- `struct file`里包含了对文件内容的 IO 缓冲区
- 对文件的读操作中，数据总会被先放到该缓存区中，然后拷贝给用户程序，之后相同位置的读操作就不会再调用外设 IO
- 写操作中总会先写入缓冲区，标记相应的缓存区域为 **dirty**，在`close`的时机写回外设

完成之后，你需要写一个测试程序来展示你的代码的正确性。你的文件系统可以自己实现一两个简单的实例（比如直接用系统 API
读写文件），不需要真的把真实的文件系统驱动给写了。

#### ❓ 得分细则

- 写出接口规范文档，并且按照你的接口规范实现了文件系统功能，可以获得 `20%` 的分数
- 在此之上实现了兼容层并且能联动文件系统执行接口功能，可以获得 `80%` 的分数
- 在此之上兼容层能挂载不同文件系统并同时起作用，可以获得 `100%` 的分数\n*
  在此之上兼容层带有文件缓存的功能，可以获得 `130%` 的分数

#### ✍提交要求

完成题目后将如下内容 **用 Markdown 写完后导出为 pdf**，统一命名为`[CNSS Recruit] 用户名 - 题目名称`
发送至 `ignotusjee@qq.com`：

- 你的代码
- 你的接口规范文档
- 你的测试截图和说明

#### 💡 Hints

- 作为 OS 领域大神，你需要使用 OS 内核常用的语言来写这个程序(c / rust)
- 你的兼容层需要注册具体文件系统的一些必要接口来实现以上的功能。作为启发，提供一些可能的接口规范：

```c
int (*open) (struct inode*, struct file*);
void (*release) (struct inode*, struct file*);
int (*read) (struct file*, size_t, size_t, char*);
int (*write) (struct file*, size_t, size_t, char*);
int (*lookup) (struct inode*, struct dentry*);
int (*create) (struct inode*, struct dentry*);
int (*mkdir) (struct inode*, struct dentry*);
int (*rmdir) (struct inode*, struct dentry*);
int (*remove) (struct inode*, struct dentry*);
```

以上的接口由你的兼容层调用，具体文件系统提供实现，用于实现上述的应用层接口。其中`lookup`
接口比较特别，它没有名称对应的用户层接口，它可以用于在文件系统（硬盘）中找到相应的文件或目录，返回文件的位置、大小等信息或者`struct dentry`
，这取决于你的实现。

- 以上描述的结构体和接口规范都是根据 Linux vfs 的实现方式提供，如果喜欢的话，你也可以实现一个类 Windows IFS
  的接口规范和实现方式，甚至参照其他小众的 OS 或者自己设计一套（前提是你的设计是合理的并且功能完备的）
- 了解操作系统（如Linux）中的IO 栈，尤其是 Linux 中的 inode, address_space 等相关概念，相信你会对这个题有更多的灵感
- 参考资料：
    - [Linux source code (early version is recomended)](https://elixir.bootlin.com/linux/v5.15/source)
    - [Overview of the Linux Virtual File System](https://www.kernel.org/doc/html/latest/filesystems/vfs.html)
    - [File Systems and Filter Driver Design Guide for Windows](https://learn.microsoft.com/en-us/windows-hardware/drivers/ifs/)

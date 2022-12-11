use alloc::boxed::Box;
use erhino_shared::{mem::Address, proc::Tid};

use crate::sync::hart::HartReadWriteLock;

// 块的字节数
const BLOCK_SIZE: usize = 8 * 1024;

// 线程栈区是分块的，一个块大小 8MB，4k对齐

// 模拟一个链表，要求 (i.start + i.count) <= (i + 1).start
#[derive(Clone)]
struct ThreadStack {
    pub id: Tid,
    // 第几个块， 栈指针地址由 mid + 8MB * (start + count) 算出
    pub start: Address,
    // 块的数量
    pub count: usize,
    pub next: Option<Box<ThreadStack>>,
}

impl ThreadStack {
    pub fn new(id: Tid) -> Self {
        Self {
            id,
            start: 0,
            count: 1,
            next: None,
        }
    }
}

#[derive(Clone)]
pub struct MemoryLayout {
    top: Address,
    mid: Address,
    stacks: Option<Box<ThreadStack>>,
    // 这把锁在复制时就应该考虑到一个问题：多个 hart 在执行进程的不同线程，有可能这个进程处于并行执行状态，fork 会导致前后失去同步。原先单线程时执行系统调用意味着进程被彻底停止所以没这个问题。
    // TODO: 需要一个手段来在其执行需要 hart 同步的操作时通知其他 hart 放弃这个进程的线程。所以还得写 IPI
    stacks_lock: HartReadWriteLock,
}

impl MemoryLayout {
    pub fn new(top: Address) -> Self {
        Self {
            top,
            mid: top / 2,
            stacks: None,
            stacks_lock: HartReadWriteLock::new(),
        }
    }

    pub fn top(self) -> Address {
        self.top
    }

    pub fn new_stack(&mut self, id: Tid) -> Result<Address, ()> {
        if let Some(start) = {
            if let Some(node) = &mut self.stacks {
                Self::find_block_free(node, 1, id)
            } else {
                let mut stack = ThreadStack::new(id);
                stack.start = 0;
                stack.count = 1;
                self.stacks = Some(Box::new(stack));
                Some(0)
            }
        } {
            Ok(self.block_to_address(start, 1))
        } else {
            Err(())
        }
    }

    fn block_to_address(&self, start: usize, count: usize) -> Address {
        (self.mid + BLOCK_SIZE * (start + count)) as Address
    }

    fn find_block_free(node: &mut ThreadStack, count: usize, for_whom: Tid) -> Option<usize> {
        if let Some(next) = &mut node.next {
            let top = node.start + node.count;
            if top < next.start {
                if next.start - top >= count {
                    Some(top)
                } else {
                    Self::find_block_free(next.as_mut(), count, for_whom)
                }
            } else {
                Self::find_block_free(next.as_mut(), count, for_whom)
            }
        } else {
            let mut stack = ThreadStack::new(for_whom);
            let start = node.start + node.count;
            stack.start = start;
            node.next = Some(Box::new(stack));
            Some(start)
        }
    }

    pub fn finalize_stack(&mut self, id: Tid) {
        let mut kill = false;
        if let Some(node) = &mut self.stacks {
            if node.id == id {
                if let Some(next) = node.next.take() {
                    self.stacks = Some(next);
                } else {
                    kill = true;
                }
            } else {
                Self::set_block_free(node.as_mut(), id);
            }
        }
        if kill {
            self.stacks = None;
        }
    }

    fn set_block_free(node: &mut ThreadStack, owner: Tid) {
        if let Some(next) = &mut node.next {
            if next.id == owner {
                if let Some(next_of_next) = next.next.take() {
                    node.next = Some(next_of_next);
                } else {
                    node.next = None;
                }
            }
        }
    }
}

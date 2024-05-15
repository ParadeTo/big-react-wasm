use std::any::Any;

pub trait Comparable {
    // self lower than Self, return true
    fn compare(&self, b: &dyn Comparable) -> bool;
    fn as_any(&self) -> &dyn Any;
}

fn push(mut heap: &mut Vec<Box<dyn Comparable>>, node: Box<dyn Comparable>) {
    heap.push(node);
    sift_up(heap, heap.len() - 1);
}

fn peek(heap: &Vec<Box<dyn Comparable>>) -> Option<&Box<dyn Comparable>> {
    if heap.is_empty() {
        return None;
    }
    return Some(&heap[0]);
}

fn pop(mut heap: &mut Vec<Box<dyn Comparable>>) -> Option<Box<dyn Comparable>> {
    if heap.is_empty() {
        None
    } else {
        let min = heap.swap_remove(0);
        if !heap.is_empty() {
            bubble_down(heap, 0);
        }
        Some(min)
    }
}

fn bubble_down(mut heap: &mut Vec<Box<dyn Comparable>>, index: usize) {
    let mut parent = index;

    loop {
        let mut child = 2 * parent + 1;
        if child >= heap.len() {
            break;
        }
        if child + 1 < heap.len() && heap[child + 1].compare(&*heap[child]) {
            child += 1;
        }
        if heap[parent].compare(&*heap[child]) {
            break;
        }
        heap.swap(parent, child);
        parent = child;
    }
}

fn sift_up(mut heap: &mut Vec<Box<dyn Comparable>>, i: usize) {
    let mut child = i;
    if child <= 0 {
        return;
    }
    let mut parent = (child - 1) / 2;

    while child > 0 && !&heap[parent].compare(&*heap[child]) {
        heap.swap(parent, child);
        child = parent;
        parent = ((child as isize - 1) / 2) as usize;
    }
}

#[cfg(test)]
mod tests {
    use std::any::Any;

    use crate::heap::{Comparable, pop, push};

    struct Task {
        id: u32,
        sort_index: f64,
    }

    impl Task {
        fn new(id: u32, sort_index: f64) -> Self {
            Self {
                id,
                sort_index,
            }
        }
    }

    // 实现 Task 的 Debug trait，以便在测试失败时能够打印 Task 的值。
    impl std::fmt::Debug for Task {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "Task {{ id: {}, sort_index: {} }}", self.id, self.sort_index)
        }
    }

    // 实现 Task 的 PartialEq trait，以便能够在断言中比较 Task 的值。
    impl PartialEq for Task {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }

    impl Comparable for Task {
        fn compare(&self, other: &dyn Comparable) -> bool {
            let other = other.as_any().downcast_ref::<Task>().unwrap();
            let diff = self.sort_index - other.sort_index;
            if diff != 0.0 {
                return diff < 0.0;
            }
            (self.id as i32 - other.id as i32) < 0
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[test]
    fn test_min_heap() {
        let mut heap = &mut vec![];


        // 添加任务到堆中
        push(heap, Box::new(Task::new(1, 2.0)));
        push(heap, Box::new(Task::new(2, 1.0)));
        push(heap, Box::new(Task::new(3, 3.0)));

        // 按预期顺序弹出任务
        assert_eq!(pop(heap).unwrap().as_any().downcast_ref::<Task>().unwrap().id, 2);
        assert_eq!(pop(heap).unwrap().as_any().downcast_ref::<Task>().unwrap().id, 1);
        assert_eq!(pop(heap).unwrap().as_any().downcast_ref::<Task>().unwrap().id, 3);

        // 堆应该为空
        assert!(heap.pop().is_none());
    }
}



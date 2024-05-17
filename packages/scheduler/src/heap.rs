// 向堆中插入元素
pub fn push<T: Ord>(heap: &mut Vec<T>, value: T) {
    heap.push(value);
    sift_up(heap, heap.len() - 1);
}

// 从堆中取出最小的元素
pub fn pop<T: Ord>(heap: &mut Vec<T>) -> Option<T> {
    if heap.is_empty() {
        return None;
    }
    let last_index = heap.len() - 1;
    heap.swap(0, last_index);
    let result = heap.pop();
    if !heap.is_empty() {
        sift_down(heap, 0);
    }
    result
}

// 向上调整堆
fn sift_up<T: Ord>(heap: &mut Vec<T>, mut index: usize) {
    while index != 0 {
        let parent = (index - 1) / 2;
        if heap[parent] <= heap[index] {
            break;
        }
        heap.swap(parent, index);
        index = parent;
    }
}

// 向下调整堆
fn sift_down<T: Ord>(heap: &mut Vec<T>, mut index: usize) {
    let len = heap.len();
    loop {
        let left_child = index * 2 + 1;
        let right_child = left_child + 1;

        // 找出当前节点和它的子节点中最小的节点
        let mut smallest = index;
        if left_child < len && heap[left_child] < heap[smallest] {
            smallest = left_child;
        }
        if right_child < len && heap[right_child] < heap[smallest] {
            smallest = right_child;
        }

        // 如果当前节点是最小的，那么堆已经是正确的了
        if smallest == index {
            break;
        }

        // 否则，交换当前节点和最小的节点
        heap.swap(index, smallest);
        index = smallest;
    }
}

pub fn peek<T: Ord>(heap: &Vec<T>) -> Option<&T> {
    heap.get(0)
}

pub fn is_empty<T: Ord>(heap: &Vec<T>) -> bool {
    heap.is_empty()
}

pub fn peek_mut<T: Ord>(heap: &mut Vec<T>) -> Option<&mut T> {
    if heap.is_empty() {
        None
    } else {
        Some(&mut heap[0])
    }
}


#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use crate::heap::{pop, push};

    #[derive(Clone)]
    struct Task {
        id: u32,
        sort_index: f64,
    }

    impl Task {
        fn new(id: u32, sort_index: f64) -> Self {
            Self { id, sort_index }
        }
    }

    impl std::fmt::Debug for Task {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(
                f,
                "Task {{ id: {}, sort_index: {} }}",
                self.id, self.sort_index
            )
        }
    }

    impl Eq for Task {}

    impl PartialEq for Task {
        fn eq(&self, other: &Self) -> bool {
            self.id.cmp(&other.id) == Ordering::Equal
        }
    }

    impl PartialOrd for Task {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            let mut sort_index_ordering;

            if self.sort_index.is_nan() {
                if other.sort_index.is_nan() {
                    sort_index_ordering = Ordering::Equal
                } else {
                    sort_index_ordering = Ordering::Less
                }
            } else if other.sort_index.is_nan() {
                sort_index_ordering = (Ordering::Greater)
            } else {
                sort_index_ordering = self.sort_index.partial_cmp(&other.sort_index).unwrap()
            }

            if sort_index_ordering != Ordering::Equal {
                return Some(sort_index_ordering);
            }
            return self.id.partial_cmp(&other.id);
        }
    }

    impl Ord for Task {
        fn cmp(&self, other: &Self) -> Ordering {
            self.partial_cmp(other).unwrap_or(Ordering::Equal)
        }
    }

    #[test]
    fn test_min_heap() {
        let mut heap = vec![];

        let task3 = Task::new(3, 3.0);
        let task2 = Task::new(2, 2.0);
        let task1 = Task::new(1, 1.0);
        let task4 = Task::new(4, 4.0);

        push(&mut heap, task3);
        push(&mut heap, task2);
        push(&mut heap, task1);
        push(&mut heap, task4);

        // 按预期顺序弹出任务
        assert_eq!(pop(&mut heap).unwrap().id == 1, true);
        assert_eq!(pop(&mut heap).unwrap().id == 2, true);
        assert_eq!(pop(&mut heap).unwrap().id == 3, true);
        assert_eq!(pop(&mut heap).unwrap().id == 4, true);

        // 堆应该为空
        assert!(heap.pop().is_none());
    }
}

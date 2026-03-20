/// A generic undo stack that stores snapshots of state.
///
/// Use with any `Clone`-able state type. Push a snapshot before making
/// a change, then pop to restore the previous state.
///
/// # Example
///
/// ```
/// use louie::util::undo::UndoStack;
///
/// let mut undo: UndoStack<String> = UndoStack::new(50);
/// undo.push("hello".to_string());
/// undo.push("hello world".to_string());
///
/// assert_eq!(undo.pop(), Some("hello world".to_string()));
/// assert_eq!(undo.pop(), Some("hello".to_string()));
/// assert_eq!(undo.pop(), None);
/// ```
#[derive(Debug, Clone)]
pub struct UndoStack<S> {
    stack: Vec<S>,
    max_depth: usize,
}

impl<S> UndoStack<S> {
    /// Create a new undo stack with the given maximum depth.
    pub fn new(max_depth: usize) -> Self {
        Self {
            stack: Vec::new(),
            max_depth: max_depth.max(1),
        }
    }

    /// Push a snapshot onto the stack. If the stack exceeds `max_depth`,
    /// the oldest entry is removed.
    pub fn push(&mut self, state: S) {
        if self.stack.len() >= self.max_depth {
            self.stack.remove(0);
        }
        self.stack.push(state);
    }

    /// Pop the most recent snapshot, or `None` if the stack is empty.
    pub fn pop(&mut self) -> Option<S> {
        self.stack.pop()
    }

    /// Peek at the most recent snapshot without removing it.
    pub fn peek(&self) -> Option<&S> {
        self.stack.last()
    }

    /// Returns the number of snapshots in the stack.
    pub fn len(&self) -> usize {
        self.stack.len()
    }

    /// Returns `true` if no snapshots are stored.
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Clear all snapshots.
    pub fn clear(&mut self) {
        self.stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_pop() {
        let mut undo: UndoStack<i32> = UndoStack::new(10);
        undo.push(1);
        undo.push(2);
        undo.push(3);
        assert_eq!(undo.pop(), Some(3));
        assert_eq!(undo.pop(), Some(2));
        assert_eq!(undo.pop(), Some(1));
        assert_eq!(undo.pop(), None);
    }

    #[test]
    fn max_depth_enforced() {
        let mut undo: UndoStack<i32> = UndoStack::new(3);
        undo.push(1);
        undo.push(2);
        undo.push(3);
        undo.push(4); // pushes out 1
        assert_eq!(undo.len(), 3);
        assert_eq!(undo.pop(), Some(4));
        assert_eq!(undo.pop(), Some(3));
        assert_eq!(undo.pop(), Some(2));
        assert_eq!(undo.pop(), None);
    }

    #[test]
    fn peek_does_not_remove() {
        let mut undo: UndoStack<i32> = UndoStack::new(10);
        undo.push(42);
        assert_eq!(undo.peek(), Some(&42));
        assert_eq!(undo.len(), 1);
    }

    #[test]
    fn clear_empties_stack() {
        let mut undo: UndoStack<i32> = UndoStack::new(10);
        undo.push(1);
        undo.push(2);
        undo.clear();
        assert!(undo.is_empty());
    }
}

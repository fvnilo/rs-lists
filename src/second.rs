/*
 * This is described as being an ok stack
 */
pub struct List<T> {
    head: Link<T>,
}

/*
 * This is a type alias. The problem with the implementation
 * in `first.rs` is that we re-invented the wheel. An Option,
 * is exactly what we want.
 */
type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

// Tuple structs are an alternative form of struct,
// useful for trivial wrappers around other types.
pub struct IntoIter<T>(List<T>);

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        // access fields of a tuple struct numerically
        self.0.pop()
    }
}

/* 'a is a lifetime parameter. A lifetime is the name of a region (~block/scope)
 * of code somewhere in a program. When a reference is tagged with a lifetime,
 * we're saying that it has to be valid for that entire region.
 *
 * Note: Rust can imply the lifetime for some cases:
 *
 * fn foo(&A) -> &B; // sugar for:
 * fn foo<'a>(&'a A) -> &'a B;
 *
 * // Many inputs, assume they're all independent
 * fn foo(&A, &B, &C); // sugar for:
 * fn foo<'a, 'b, 'c>(&'a A, &'b B, &'c C);
 *
 * // Methods, assume all output lifetimes are derived from `self`
 * fn foo(&self, &B, &C) -> &D; // sugar for:
 * fn foo<'a, 'b, 'c>(&'a self, &'b B, &'c C) -> &'a D;
 *
 * So what does fn foo<'a>(&'a A) -> &'a B mean? In practical terms, all it means is that the input must live at least as long as the output. So if you keep the output around for a long time, this will expand the region that the input must be valid for. Once you stop using the output, the compiler will know it's ok for the input to become invalid too.
 */
pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.as_ref().map::<&Node<T>, _>(|node| &node);
            &node.elem
        })
    }
}

pub struct IterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node| {
            self.next = node.next.as_mut().map(|node| &mut **node);
            &mut node.elem
        })
    }
}

impl<T> List<T> {}

impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None }
    }

    pub fn push(&mut self, elem: T) {
        /*
         * The operation mem::replace(&mut option, None)
         * is actually implemented in Option.take()
         */
        let new_node = Box::new(Node {
            elem: elem,
            next: self.head.take(),
        });

        self.head = Some(new_node);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.head.take().map(|node| {
            self.head = node.next;
            node.elem
        })
    }

    pub fn peek(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.elem)
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|node| &mut node.elem)
    }

    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }

    // This is equivalent to  pub fn iter<'a>(&'a self) -> Iter<'a, T>
    // because of lifetime elision.
    pub fn iter(&self) -> Iter<T> {
        Iter {
            next: self.head.as_ref().map(|node| &**node),
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            next: self.head.as_mut().map(|node| &mut **node),
        }
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut cur_link = self.head.take();
        while let Some(mut boxed_node) = cur_link {
            cur_link = boxed_node.next.take();
        }
    }
}

// This line means that this module will only be
// compiled when running tests.
#[cfg(test)]
mod test {
    // We need to import the list.
    use super::List;

    #[test]
    fn basics() {
        // Note: You do not need to specify the type for generics here.
        let mut list = List::new();

        // Check empty list behaves right
        assert_eq!(list.pop(), None);

        // Populate list
        list.push(1);
        list.push(2);
        list.push(3);

        // Check normal removal
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push(4);
        list.push(5);

        // Check normal removal
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), None);
    }

    #[test]
    fn peek() {
        let mut list = List::new();
        assert_eq!(list.peek(), None);
        assert_eq!(list.peek_mut(), None);
        list.push(1);
        list.push(2);
        list.push(3);

        assert_eq!(list.peek(), Some(&3));
        assert_eq!(list.peek_mut(), Some(&mut 3));

        list.peek_mut().map(|value| *value = 42);

        assert_eq!(list.peek(), Some(&42));
        assert_eq!(list.pop(), Some(42));
    }

    #[test]
    fn into_iter() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
    }

    #[test]
    fn iter_mut() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.iter_mut();
        assert_eq!(iter.next(), Some(&mut 3));
        assert_eq!(iter.next(), Some(&mut 2));
        assert_eq!(iter.next(), Some(&mut 1));
    }
}

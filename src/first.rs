/*
 * This is described as being a bad stack
 */
use std::mem;
/*
 * List is a struct with a single field, its size is the same as that field.
 * The `List` is also implemented in such a way because:
 *
 *  > The problem is that the internals of an enum are totally public,
 *  > and we're not allowed to publicly talk about private types.
 *
 * My understanding is that the if our Linked List is:
 *
 * ```
 * struct Node {
 *   elem: i32,
 *   next: List,
 * }
 *
 * pub enum List {
 *  Empty,
 *  More(Box<Node>),
 * }
 * ```
 *
 * All of the details of an enum are public but Node does not
 * have the `pub` qualifier. So we switched it to a struct.
 */
pub struct List {
    head: Link,
}

/*
 * A box is a "heap allocation" unit. My understanding is that
 * it is a feature or Rust's memory safety mecanism. Otherwise
 *
 * The very initial implementation was this:
 * ```
 * pub enum List {
 *   Empty,
 *   Elem(i32, List),
 * }
 * ```
 *
 * This will not compile as it is a recursite data type and
 * nothing informs us of what the size of List is.
 *
 * That being said adding a box isn't the best idea because
 * this is what the stack will look like:
 *
 * ```
 * [Elem A, ptr] -> (Elem B, ptr) -> (Empty *junk*)
 * ```
 *
 * The tail element is a "fake" node. Also, the way Rust memory
 * management works, from what I have read, the `Empty` element
 * will still allocate memory to be ready to become a `Elem`.
 *
 */
enum Link {
    Empty,
    More(Box<Node>),
}

struct Node {
    elem: i32,
    next: Link,
}

/* Ownership 101
 * There are 3 primary forms that self can take: self, &mut self, and &self.
 *
 * These 3 forms represent the three primary forms of ownership in Rust:
 * - self - Value
 * - &mut self - mutable reference
 * - &self - shared reference
 *
 * A value represents true ownership. You can do whatever you want with a value: move it, destroy it, mutate it, or loan it out via a reference. When you pass something by value, it's moved to the new location. The new location now owns the value, and the old location can no longer access it. For this reason most methods don't want self -- it would be pretty lame if trying to work with a list made it go away!
 *
 * A mutable reference represents temporary exclusive access to a value that you don't own. You're allowed to do absolutely anything you want to a value you have a mutable reference to as long you leave it in a valid state when you're done (it would be rude to the owner otherwise!). This means you can actually completely overwrite the value. A really useful special case of this is swapping a value out for another, which we'll be using a lot. The only thing you can't do with an &mut is move the value out with no replacement. &mut self is great for methods that want to mutate self.
 *
 * A shared reference represents temporary shared access to a value that you don't own. Because you have shared access, you're generally not allowed to mutate anything. Think of & as putting the value out on display in a museum. & is great for methods that only want to observe self.
 */
impl List {
    pub fn new() -> Self {
        List { head: Link::Empty }
    }

    pub fn push(&mut self, elem: i32) {
        /*
         * A naive implementation would be:
         *
         * pub fn push(&mut self, elem: i32) {
         *   let new_node = Box::new(Node {
         *     elem: elem,
         *     next: self.head,
         *   });
         *
         *   self.head = Link::More(new_node);
         * }
         *
         * The problem here is that, we try to move out self.head into another place
         * which Rust doesn't let us do because, as mentionned above, we need to
         * return the reference to its owner in a valid state, which suddenly isn't the case.
         * Even re-assigning `self.head` isn't enough because [exception
         * safety](https://doc.rust-lang.org/nightly/nomicon/exception-safety.html)
         *
         * Memory replacement isn't the best solution neither, but it'll do for now.
         */
        let new_node = Box::new(Node {
            elem: elem,
            next: mem::replace(&mut self.head, Link::Empty),
        });

        self.head = Link::More(new_node);
    }

    pub fn pop(&mut self) -> Option<i32> {
        /*
         * What we are trying to achieve is:
         *
         * - Check if the list is empty.
         * - If it's empty, just return None
         * - If it's not empty
         *   - remove the head of the list
         *   - remove its elem
         *   - replace the list's head with its next
         *   - return Some(elem)
         */
        match mem::replace(&mut self.head, Link::Empty) {
            Link::Empty => None,
            Link::More(node) => {
                self.head = node.next;
                Some(node.elem)
            }
        }
    }
}

// A type has a destructor if it implements a trait called Drop.
impl Drop for List {
    fn drop(&mut self) {
        let mut cur_link = mem::replace(&mut self.head, Link::Empty);
        // `while let` == "do this thing until this pattern doesn't match"
        while let Link::More(mut boxed_node) = cur_link {
            cur_link = mem::replace(&mut boxed_node.next, Link::Empty);
            // boxed_node goes out of scope and gets dropped here;
            // but its Node's `next` field has been set to Link::Empty
            // so no unbounded recursion occurs.
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
}

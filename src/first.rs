// Chapter 2

// 2.1
// Layout

/* Layout 1 */
// pub enum List {
    // Empty, 
    // Elem(i32, Box<List>),
// }
// Issue:   Allows mixed allocation Heap/Stack (first elem may be stack allocated);
//          also always ends with node allocation for something that is not actually a node

/* BUT: 
This previous layout took advantage of the null pointer optimization.
We previously saw that every enum has to store a tag to specify which 
variant of the enum its bits represent. 
However, if we have a special kind of enum:

    enum Foo {  
        A,  
        B(ContainsANonNullPtr), 
    }   

the null pointer optimization kicks in, which eliminates the space needed for the tag. 
If the variant is A, the whole enum is set to all 0's. 
Otherwise, the variant is B. 
This works because B can never be all 0's, since it contains a non-zero pointer. Slick!

*/


/* Layout 2 */
// pub enum List {
//     Empty,
//     ElemThenEmpty(i32),
//     ElemThenNotEmpty(i32, Box<List>),
// }
// Issue:   Invalid State `ElemThenNotEmpty(i32, Box<Empty>)` is possible; 
//          also, still does not uniformly allocate on heap

/* Layout 3 */
// struct Node {
    // elem: i32,
    // next: List,
// }
// 
// pub enum List {
    // Empty, 
    // More(Box<Node>),
// }
// Tail of list never allocates extra junk: Check!
// `enum` is in "delicious" null-pointer-optimized form: Check!
// all elements are uniformly allocated on the heap: Check!
// `List` is a recursive type: Check!
// `Node` is a recursive type: Check!
// BUUUUUTTT: Compiler complains -> List is pub, but Node is not, but Node contains the pub List

use std::mem;

/* Layout 4 */
pub struct List {
    // List is a struct with a single field
    // --> the size of List is the size of the field
    // --> Zero Cost Abstraction 
    head: Link,
}

enum Link {
    Empty,
    More(Box<Node>),
}

struct Node {
    elem: i32, 
    next: Link,
}

// 2.2
// Constructor (for empty list)
impl List {
    pub fn new() -> Self {
        List { head: Link::Empty }
    }

    // 2.3 
    // Push
    pub fn push(&mut self, elem:i32) {
        let new_node = Box::new(
            Node {
                elem: elem,
                next: mem::replace(&mut self.head, Link::Empty),
            }
        );
        self.head = Link::More(new_node)
    }

    // 2.4
    // Pop
    pub fn pop(&mut self) -> Option<i32> {
        match mem::replace(&mut self.head, Link::Empty) {
            Link::Empty => None,
            Link::More(node) => {
                self.head = node.next;
                Some(node.elem)
            }
        }
    }
}

impl Drop for List {
    fn drop(&mut self) {
        // pull the current head link from its Box, replace it with an empty value
        let mut cur_link = mem::replace(&mut self.head, Link::Empty);
        // `while let` == "do this thing until this pattern doesn't match" -> while not empty
        while let Link::More(mut boxed_node) = cur_link { // move the value, NOT a reference
            cur_link = mem::replace(&mut boxed_node.next, Link::Empty);
            // boxed_node goes out of scope at the end of every step in the loop
            // --> implicitely gets dropped
            // its Node's `next` field is set to Link::Empty, so no unbound recursion occurs 
            // --> iterates over full list, moving non-trivial structures out to temp variables and implicitely dropping the rest
        }
    }
    /* 
        Alternate idea: use 
            while let Some(_) = self.pop() { }
        Difference:
            Pop returns Option<i32>, while the above implementation only manipulates Links (i.e. Box<Node>)
            -> only moves around pointers, while pop moves values
        Problem:
            Moving values can become very expensive in a generalized list, where values can become big instances of VeryBigThingWithADropImpl (VBTWADI).
            Boxes can run the drop implementation of their contents in-place, eliminating these issues. 
            Since VBTWADI is exactly what makes using linked-lists desirable over arrays in the first place, this bad performance would let the entire concept down.
        Solution:
            Best-of-both-worlds implementation: 
            Add new method `fn pop_node(&mut self) -> Link`, from which both `pop` and `drop` can be cleanly derived.

     */
}

// 2.5
// Testing

#[cfg(test)]
mod test {
    use crate::first::List;

    #[test]
    fn basics() {
        let mut list = List::new();
        // check to see if empty lists behave correctly
        assert_eq!(list.pop(), None);

        list.push(1);
        list.push(2);
        list.push(3);

        // check normal removal
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(2));

        // push some more values in just to make sure nothing gets corrupted
        list.push(4);
        list.push(5);

        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), Some(4));
        assert_eq!(list.pop(), Some(1));
        // check list exhaustiong
        assert_eq!(list.pop(), None);
    }
}
/* 
    Initial implementation of singly-linked list in first.rs is kinda sucky, init. 
    Let's make it less sucky, by trying again and: 
    - de-inventing the wheel
    - making list able to handly ANY element type
    - adding peeking
    - making list iterable

    And in the process, we will learn about: 
    - Advanced Option useage
    - Generics
    - Lifetimes
    - Iterators

    Initially, all code from first.rs is copied over (taken from reference implementation in the book).
    It will be unrecognisable when refactoring/implematation is done.
*/

// 3.2 
// making it generic, using T type substitute
pub struct List<T> {
    head: Link<T>,
}

// 3.1 + 3.2
// Link implementation was basically a worse re-invention of Option<>
// use type aliasing for readability
type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None }
    }

    pub fn push(&mut self, elem: T) {
        let new_node = Box::new(Node {
            elem: elem,
            // mem::replace(&mut TARGET_VALUE, None) is SO incredibly common, that Option comes with a dedicated method for it
            next: self.head.take(),
        });

        self.head = Some(new_node);
    }

    pub fn pop(&mut self) -> Option<T> {
        // mem::replace(&mut TARGET_VALUE, None) is SO incredibly common, that Option comes with a dedicated method for it
        // match option { None => None, Some(x) => Some(y) } is equally common, so there is `map` for that
        self.head.take().map( |node| {
            self.head = node.next;
            node.elem
        })
    }

    // 3.3
    pub fn peek(&self) -> Option<&T> {
        // map takes the `self` by value, which would move the Option val out -> we need to use the `as_ref` method for Option<T>
        // this demotes the Option<T> to a reference to its internals
        // we need to use an extra dereference to cut through this extra indirection though.
        self.head.as_ref().map(|node| {
            &node.elem
        })
    }
    // 3.3
    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|node| {
            &mut node.elem
        })
    }
}


impl<T> Drop for List<T> {
    fn drop(&mut self) {
        // mem::replace(&mut TARGET_VALUE, None) is SO incredibly common, that Option comes with a dedicated method for it
        let mut cur_link = self.head.take();

        while let Some(mut boxed_node) = cur_link {
            // mem::replace(&mut TARGET_VALUE, None) is SO incredibly common, that Option comes with a dedicated method for it
            cur_link = boxed_node.next.take();
        }
    }
}

// Preamble to 3.4, 3.5, 3.6
/* 
    Collections in Rust use the Iterator Trait; it's a bit more complicated than Drop:
    pub trait Iterator {
        type Item;
        fn next(&mut self) -> Option<Self::Item>;
    }
    --> new kid on the block: Item
    Item aliases a mandatory associtated type

    Iterators yield Option<Self::Item>, because the interfaces coalesces `has_next` & `get_next` concepts:
    if there is a next value, return Some(next), otherwise None.

    Rust does not actually have a `yield` statement (yet, as of 2025-04-07). 
    So we implement that logic ourselves. 
    Also, 3 different kinds of iterator should be implemented for all collections:
    - IntoIter  - T
    - IterMut   - &mut T
    - Iter      - &T

    We use the `pop` method from our own interface to implement IntoIter:
*/

// 3.4

// Tuple structs are an alternative form of struct, useful for trivial wrappers around other types
pub struct IntoIter<T>(List<T>);

impl<T> List<T> {
    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        // access fields of a tuple struct numerically
        self.0.pop()    // tuple index can be queried with dot notation
    }
}

// 3.5

// Iter implementation cannot rely on pre-existing List features
// basic logic: hold a pointer to the current node we want to yield next
// !: Node may not exist (empty list or otherwise done iterating)
// --> Reference must be Option, and we traverse to current node's next node after every yield

// here, we start requiring Lifetimes!
// Iter is generic over *some* lifetime, it does not care
pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

// no lifetimes here - List does not have any associated lifetimes
impl<T> List<T> {
    // we declare a fresh lifetime here, though, for the *exact* borrow that creates the Iter;
    // now, &self needs to be valid as long as the Iter is around!
    pub fn iter<'a>(&'a self) -> Iter<'a, T> {
        // note: lifetime elision COULD be applied here; `pub fn iter(&self) -> Iter<'T> {…}` is equivalent to our signature
        Iter {
            // Option<T>.as_deref() does just that, while considering the possibility of a None
            next: self.head.as_deref()
        }
    }
    // also: while using elision, one can hint at the hidden presence of a lifetime by using
    // the Rust 2018 "explicitely elided lifetime" syntax: `'_`
    // --> pub fn iter(&self) -> Iter<'_, T> {…}
}

impl<'a, T> Iterator for Iter<'a, T> {
    // type declarations need lifetimes
    type Item = &'a T;
    // no lifetime needed here though, handled by the lifetime above
    // Self continues to be incredibly hype and amazing (sic)
    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            // next is a Box inside the Option, which we need to unpack
            // Option<T>.as_deref() does just that, while considering the possibility of a None
            self.next = node.next.as_deref();
            &node.elem
        })
    }
}

/* 
    Regarding `.as_deref()`:
    The as_deref and as_deref_mut functions are stable as of Rust 1.40. 
    Before that you would need to do map(|node| &**node) and map(|node| &mut**node). 
    You may be thinking "wow that &** thing is really janky", and you're not wrong, 
    but like a fine wine Rust gets better over time and we no longer need to do such. 
    
    Normally Rust is very good at doing this kind of conversion implicitly, 
    through a process called deref coercion, where basically it can insert *'s throughout your code to make it type-check. 
    It can do this because we have the borrow checker to ensure we never mess up pointers!

    But in this case the closure in conjunction with the fact that we have an Option<&T> instead of &T 
    is a bit too complicated for it to work out, so we need to help it by being explicit. 
    Thankfully this is pretty rare, in my experience.

    Just for completeness' sake, we could give it a different hint with the turbofish:

        self.next = node.next.as_ref().map::<&Node<T>, _>(|node| &node);

    See, map is a generic function:

        pub fn map<U, F>(self, f: F) -> Option<U>

    The turbofish, ::<>, lets us tell the compiler what we think the types of those generics should be. 
    In this case ::<&Node<T>, _> says "it should return a &Node<T>, and I don't know/care about that other type".

    This in turn lets the compiler know that &node should have deref coercion applied to it, 
    so we don't need to manually apply all those *'s!

    But in this case I don't think it's really an improvement, 
    this was just a thinly veiled excuse to show off deref coercion and the sometimes-useful turbofish. 
*/

// 3.6
// quote: IterMut is going to be WILD!!
// Iter returns shared (immutable) references, which may coexist in unlimited numbers
// IterMut's mutable references CANNOT coexist; the whole point is that they are exclusive. 

// Start by taking the Iter code and making EVERYTHING mutable!

pub struct IterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
}

impl<T> List<T> {
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut { next: self.head.as_deref_mut() }  // deref must be mut now, and so must the ref to self
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T; 

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map( |node| {      // to avoid the tedium of sharing mut references-- just TAKE the value, i.e. the mut ref to the Node
            // now we have exclusive ownership over the mut ref, which has been removed from its original location - while its value stays in the List
            self.next = node.next.as_deref_mut();
            &mut node.elem
        })
    }
    /* 
        &mut isn't Copy (if you copied an &mut, you'd have two &mut's to the same location in memory, which is forbidden). 
        Instead, we take the Option to get it. 
        We take the Option<&mut> so we have exclusive access to the mutable reference. No need to worry about someone looking at it again.
        Rust understands that it's ok to shard a mutable reference into the subfields of the pointed-to struct, 
        because there's no way to "go back up", and they're definitely disjoint.

        It turns out that you can apply this basic logic to get a safe IterMut for an array or a tree as well! 
        You can even make the iterator DoubleEnded, so that you can consume the iterator from the front and the back at once! 
        Woah!
     */
}


#[cfg(test)]
mod test {
    use super::List;

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

    #[test]
    fn peek() {
        // test peeking empty list
        let mut list = List::new();
        assert_eq!(list.peek(), None);
        assert_eq!(list.peek_mut(), None);
        // add some elements
        list.push(1); list.push(2); list.push(3);
        assert_eq!(list.peek(), Some(&3));
        assert_eq!(list.peek_mut(), Some(&mut 3));
        // test correct re-assignment of element value
        list.peek_mut().map(|value| {
            *value = 42
        });
        assert_eq!(list.peek(), Some(&42));
        assert_eq!(list.pop(), Some(42));
    }

    #[test]
    fn into_iter() {
        let mut list = List::new();
        list.push(1); list.push(2); list.push(3);

        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter() {
        let mut list = List::new();
        list.push(1); list.push(2); list.push(3);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_mut() {
        let mut list = List::new();
        list.push(1); list.push(2); list.push(3);

        let mut iter = list.iter_mut();
        assert_eq!(iter.next(), Some(&mut 3));
        iter.next().map(|value| *value=42);
        assert_eq!(iter.next(), Some(&mut 1));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);

        let mut iter = list.iter_mut();
        assert_eq!(iter.next(), Some(&mut 3));
        assert_eq!(iter.next(), Some(&mut 42));
        assert_eq!(iter.next(), Some(&mut 1));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);

    }
}

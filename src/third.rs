// Chapter 4 -  Moving from single ownership to shared ownership
//              --> Writing a persistent immutable singly-linked list

/* 
Common functional workflow manipulating list tails with persistent lists:
    list1 = A -> B -> C -> D
    list2 = tail(list1) = B -> C -> D
    list3 = push(list2, X) = X -> B -> C -> D
Target Memora Layout:
    list1 -> A ---+
                  |
                  v
    list2 ------> B -> C -> D
                  ^
                  |
    list3 -> X ---+

But this cannot work with Boxes, since ownership of B is shared here; who should free its memory!?
Garbage collected languages will free this memory once EVERYBODY stopped looking at it. 
How to do this in Rust!? -> Reference Counting --> Rc

    Rc is just like Box, but we can duplicate it,
    and its memory will only be freed when all the Rc's derived from it are dropped. 
    
    Unfortunately, this flexibility comes at a serious cost: 
        we can only take a shared reference to its internals. 
    This means we can't ever really get data out of one of our lists, nor can we mutate them.
*/

// Chapter 4.1

use std::rc::Rc;

pub struct List<T> {
    head: Link<T>,
}

type Link<T> = Option<Rc<Node<T>>>;

struct Node<T> {
    elem: T, 
    next: Link<T>,
}

// Chapter 4.2
// Basics - Methods

impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None }
    }

    // Replace `push` and `pop`  with  `prepend` and `tail`

    // Prepend takes a list and an element, returns a List
    // create new node and put existing List as its `next`
    // however, next needs to be `Clone`d
    pub fn prepend(&self, elem: T) -> List<T> {
        List { head: Some(Rc::new(
            Node {
                elem: elem, 
                next: self.head.clone(),
                // Clone is implemented for almost every type; 
                // Rc uses Clone as a way to increment its reference count
                //  --> we don't move a box to a sublist, but instead we clone the head of the old list
                // no matching of the head needed, Option exposes a Clone implementation for us <3
            }
        )) }
    }

    // tail is the logical inverse -> takes a list and returns it with the first element removed
    pub fn tail(&self) -> List<T> {
        // make use of the `and_then` pattern for Options -> lets us return an Option
        List { head: self.head.as_ref().and_then(|node| node.next.clone()) }
    }
    
    // in addition to tail, we also need a way to get the head, to return a reference to the current first element
    pub fn head(&self) -> Option<&T> {
        // very much like peek 
        self.head.as_ref().map(|node| &node.elem)
    }

}

// Iter is the same as it was for the mutable list from Chapter 3
pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<T> List<T> {
    pub fn iter(&self) -> Iter<'_, T> {
        Iter { next: self.head.as_deref() }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.as_deref();
            &node.elem
        })
    }
}

/*  Important Note: 
        We CANNOT implement IntoIter or IterMut for this type, since we ONLY have SHARED access!
*/


// Chapter 4.3
// Drop

/*  
    Like the mutable list, we have a recursive destructor problem; but not as bad. 
    If we ever hit another node that's the head of another list, somewhere, wo will NOT recursively drop it. 
    Still, it is a thing we should care about, but it is not immediately clear how to deal with it. 
*/
impl<T> Drop for List<T> {
    fn drop(&mut self) {
        // take the original head, and move it into scope to implicitely drop it later 
        let mut head = self.head.take();
        // then, while we actually have valid nodes
        while let Some(node) = head {
            // try unwrap the value at the Rc pointer location
            if let Ok(mut node) = Rc::try_unwrap(node) {
                // if we are the LAST watcher of the value behind the Rc, it lets us take the value;
                // we then move it into scope, and that's the last we'll ever hear of it. 
                head = node.next.take();
            } else {
                // if anyone else is still watching the Rc, we leave them alone and go on a nice long
                break;
            }
        }
    }
}

/*  Chapter 4.4 - Arc

    Immutable linked lists are awesome to make data available across threads, BUT our implementation is unsafe due to shared mutable state.
    We need to count references ATOMICALLY to make the list thread-safe; otherwise two threads could increment the reference count at the same time, 
    creating a race condition and only incrementing once. The list could then get freed too soon. 

    --> Arc (atomic reference counter) is the solution! 

    Arc are on the surface exactly the same as Rc, but under the hood guarantee thread safety as they are modified to be atomic. 
    Only use when needed due to the added overhead. [std::sync::Arc]

    Of course, you can't magically make a type thread safe by putting it in Arc. Arc can only derive thread-safety like any other type.
*/

#[cfg(test)]
mod test {
    use crate::third::List;

    #[test]
    fn basics() {
        let list = List::new();
        assert_eq!(list.head(), None);

        let list = list.prepend(1).prepend(2).prepend(3);
        assert_eq!(list.head(), Some(&3));

        let list = list.tail();
        assert_eq!(list.head(), Some(&2));

        let list = list.tail();
        assert_eq!(list.head(), Some(&1));

        let list = list.tail();
        assert_eq!(list.head(), None);

        // Make sure that empty tail also works
        let list = list.tail();
        assert_eq!(list.head(), None);
    }

    #[test]
    fn iter() {
        let list = List::new().prepend(1).prepend(2).prepend(3);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

}
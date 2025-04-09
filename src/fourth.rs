// Chapter 5 - Bad Safe Deque

// Chapter 5.1 : Layout
/* 
    New design key: RefCell Type, at the heart of which are 2 methods:
    fn borrow(&self) -> Ref<'_, T>:
    fn borrow_mut(&self) -> RefMut<'_, T>:

    The rules of borrow and borrow_mut are equal to & and &mut
    However, they are only enforced at Runtime, NOT statically at compile time.
    Break the rules, and RefCells will panic! and crash the program. 

    Now with Rc and RefCell, Rust can become... an incredibly verbose pervasively mutable garbage collected language that can't collect cycles! 
    Y-yaaaaay...

    Alright, we want to be doubly-linked. This means each node has a pointer to the previous and next node. 
    Also, the list itself has a pointer to the first and last node. This gives us fast insertion and removal on both ends of the list.
 */

use std::rc::Rc;
use std::cell::{RefCell, Ref, RefMut};

pub struct List<T> {
    head: Link<T>,
    tail: Link<T>,
}

type Link<T> = Option<Rc<RefCell<Node<T>>>>;

struct Node<T> {
    elem: T, 
    next: Link<T>,
    prev: Link<T>,
}

// Chapter 5.2 : Building Up

impl <T> Node<T> {
    fn new(elem: T) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(
            Node {
                elem: elem, 
                prev: None,
                next: None,
            }
        ))
    }
}

impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None, tail: None }
    }

    /* 
        Doubly-linked lists are more complicated; pushing to the front needs more work.
        When transitioning to and/or from the empty list, BOTH head and tail must be edited AT ONCE. 

        An easy way to validate our methods is the following invariant:
                - each node should have exactly two pointers to it -
        
        -> each node in the middle of the list is pointed at by its predecessor AND successor, 
        -> while nodes on the ends are pointed to by the list itself
     */
    pub fn push_front(&mut self, elem: T) {
        // new node needs +2 links, everything else should be +0
        let new_head = Node::new(elem);
        match self.head.take() {
            Some(old_head) => {
                // non-empty list -> needs to connect to old_head
                old_head.borrow_mut().prev = Some(new_head.clone()); // +1 new_head
                new_head.borrow_mut().next = Some(old_head);         // +1 old_head
                self.head = Some(new_head);             // +1 new_head, -1 old_head
                // total: +2 new_head, +0 old_head -- OK!
            }
            None => {
                // empty list -> needs to set the tail
                self.tail = Some(new_head.clone());     // +1 new_head
                self.head = Some(new_head);             // +1 new_head
                // total: +2 new_head -- OK!
            }
        }
    }

    // Chapter 5.3 : Breaking Down

    // pop_front has same basic logic as push_front, but backward
    pub fn pop_front(&mut self) -> Option<T> {
        // needs to take the old head and ensure it's -2
        self.head.take().map(|old_head|  {                  // -1 old (happing in any case)
            match old_head.borrow_mut().next.take() {
                Some(new_head) => {                         // -1 new (only if exists)
                    // when not emptying list
                    new_head.borrow_mut().prev.take();      // -1 old
                    self.head = Some(new_head);             // +1 new
                }
                None => {
                    // when emptying list
                    self.tail.take();                       // -1 old
                    // total: -2 old, (no new)
                }
            }
            Rc::try_unwrap(old_head).ok().unwrap().into_inner().elem
        })
    }


    // Chapter 5.4 : Peeking

    pub fn peek_front(& self) -> Option<Ref<T>> {
        self.head.as_ref().map(|node| {
            Ref::map(node.borrow(), |node| &node.elem)
        })
    }

    pub fn peek_front_mut(&mut self) -> Option<RefMut<T>> {
        self.head.as_ref().map(|node| {
            RefMut::map(node.borrow_mut(), |node| &mut node.elem)
        })
    }

    // Chapter 5.5 : Symmetric Cases - Implement everything again, but from the back
    /* 
        tail <-> head
        next <-> prev
        front -> back
     */

    pub fn push_back(&mut self, elem: T) {
        // new node needs +2 links, everything else should be +0
        let new_tail = Node::new(elem);
        match self.tail.take() {
            Some(old_tail) => {
                // non-empty list -> needs to connect to old_head
                old_tail.borrow_mut().next = Some(new_tail.clone()); // +1 new_head
                new_tail.borrow_mut().prev = Some(old_tail);         // +1 old_head
                self.tail = Some(new_tail);             // +1 new_head, -1 old_head
                // total: +2 new_head, +0 old_head -- OK!
            }
            None => {
                // empty list -> needs to set the tail
                self.head = Some(new_tail.clone());     // +1 new_head
                self.tail = Some(new_tail);             // +1 new_head
                // total: +2 new_head -- OK!
            }
        }
    }

    // pop_front has same basic logic as push_front, but backward
    pub fn pop_back(&mut self) -> Option<T> {
        // needs to take the old head and ensure it's -2
        self.tail.take().map(|old_tail|  {                  // -1 old (happing in any case)
            match old_tail.borrow_mut().prev.take() {
                Some(new_tail) => {                         // -1 new (only if exists)
                    // when not emptying list
                    new_tail.borrow_mut().next.take();      // -1 old
                    self.tail = Some(new_tail);             // +1 new
                }
                None => {
                    // when emptying list
                    self.head.take();                       // -1 old
                    // total: -2 old, (no new)
                }
            }
            Rc::try_unwrap(old_tail).ok().unwrap().into_inner().elem
        })
    }

    pub fn peek_back(& self) -> Option<Ref<T>> {
        self.tail.as_ref().map(|node| {
            Ref::map(node.borrow(), |node| &node.elem)
        })
    }

    pub fn peek_back_mut(&mut self) -> Option<RefMut<T>> {
        self.tail.as_ref().map(|node| {
            RefMut::map(node.borrow_mut(), |node| &mut node.elem)
        })
    }


}


impl<T> Drop for List<T> {
    fn drop(&mut self) {
        // pop until None, do nothing with it -> let Nodes & Links just go out of scope
        while self.pop_front().is_some() {}
    }
}

// Chapter 5.6 : Iteration

// IntoIter
pub struct IntoIter<T>(List<T>);

impl<T> List<T> {
    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front()
    }
}

impl <T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        self.0.pop_back()
    }
}

/* 
// Iter
pub struct Iter<'a, T>(Option<Ref<'a, Node<T>>>);

impl<T> List<T> {
    pub fn iter(&self) -> Iter<T> {
        Iter(self.head.as_ref().map(|head| head.borrow()))
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = Ref<'a, T>;
    fn next (&mut self) -> Option<Self::Item> {
        self.0.take().map(|node_ref| {
            let (next, elem) = Ref::map_split(node_ref, |node| {
                (&node.next, &node.elem)
            });
            self.0 = if next.is_some() {
                Some(Ref::map(next, |next| &**next.as_ref().unwrap()))
                ... 
            } else {
                None
            };
            elem
        })
    }
    c<RefCell> has really truly finally failed us. 
    Interestingly, we've experienced an inversion of the persistent stack case. 
    Where the persistent stack struggled to ever reclaim ownership of the data but could get references all day every day, 
    our list had no problem gaining ownership, but really struggled to loan our references.

    Although to be fair, most of our struggles revolved around wanting to hide the implementation details and have a decent API.
    We could do everything fine if we wanted to just pass around Nodes all over the place.

    Heck, we could make multiple concurrent IterMuts that were runtime checked to not be mutable accessing the same element!
    
    Really, this design is more appropriate for an internal data structure that never makes it out to consumers of the API.
    Interior mutability is great for writing safe applications. Not so much safe libraries.

    Anyway, that's me giving up on Iter and IterMut. We could do them, but ugh.
} 
 */

#[cfg(test)]
mod test {
    use crate::fourth::List;

    #[test]
    fn basics() {
        let mut list = List::new();

        // check empty list behaves right
        assert_eq!(list.pop_front(), None);

        // Populate the list
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        // check normal removal
        assert_eq!(list.pop_front(), Some(3));
        assert_eq!(list.pop_front(), Some(2));

        // push some more to make sure nothing gets corrupted
        list.push_front(4);
        list.push_front(5);
        
        // check normal removal again
        assert_eq!(list.pop_front(), Some(5));
        assert_eq!(list.pop_front(), Some(4));

        // check depletion
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.pop_front(), None);

        // ---- back -----
    
        // Check empty list behaves right
        assert_eq!(list.pop_back(), None);
    
        // Populate list
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);
    
        // Check normal removal
        assert_eq!(list.pop_back(), Some(3));
        assert_eq!(list.pop_back(), Some(2));
    
        // Push some more just to make sure nothing's corrupted
        list.push_back(4);
        list.push_back(5);
    
        // Check normal removal
        assert_eq!(list.pop_back(), Some(5));
        assert_eq!(list.pop_back(), Some(4));
    
        // Check exhaustion
        assert_eq!(list.pop_back(), Some(1));
        assert_eq!(list.pop_back(), None);        
        assert_eq!(list.pop_back(), None);        

        // ---- mixed -----
        list.push_front(33); list.push_front(66);
        assert_eq!(list.pop_back(), Some(33));
        assert_eq!(list.pop_back(), Some(66));
        assert_eq!(list.pop_back(), None);
        assert_eq!(list.pop_front(), None);

        list.push_back(33); list.push_back(66);
        assert_eq!(list.pop_front(), Some(33));
        assert_eq!(list.pop_front(), Some(66));
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.pop_back(), None);

    }

    #[test]
    fn peek() {
        let mut list = List::new();
        assert!(list.peek_front().is_none());
        assert!(list.peek_back().is_none());
        assert!(list.peek_front_mut().is_none());
        assert!(list.peek_back_mut().is_none());


        list.push_front(1); list.push_front(2); list.push_front(3);
        assert_eq!(&*list.peek_front().unwrap(), &3);
        assert_eq!(&mut *list.peek_front_mut().unwrap(), &mut 3);
        assert_eq!(&*list.peek_back().unwrap(), &1);
        assert_eq!(&mut *list.peek_back_mut().unwrap(), &mut 1);
    }

    #[test]
    fn into_iter() {
        let mut list = List::new();
        list.push_front(1); list.push_front(2); list.push_front(3);

        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next_back(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next_back(), None);
        assert_eq!(iter.next(), None);        
    }
}
// Chapter 6 - An OK Unsafe Singly-Linked Queue

/* 
    Ok that reference-counted interior mutability stuff got a little out of control.
    Surely Rust doesn't really expect you to do that sort of thing in general? 
    Well, yes and no. Rc and Refcell can be great for handling simple cases, but they can get unwieldy.
    Especially if you want to hide that it's happening. There's gotta be a better way!
    
    In this chapter we're going to roll back to singly-linked lists 
    and implement a singly-linked queue to dip our toes into raw pointers and Unsafe Rust.
 */

//  6.1 : Layout

/* 
    To build a Queue (FIFO rather than the previous Stack's LIFO), either push or pop has to be moved to the opposite end.
    In the previous layouts, this would require traversing the entire list per each operation chosen to move to the tailend.
    --> store a "head"-pointer, AND a "tailend"-pointer!

    BUUUUUUT initial layout is self-referencing -> we borrow self from ourselves as mutable for push, and free ourselves in pop.
    After some explaining, the author suggests to dive into UNSAFE RUST and raw pointers
 */

 /* 
 pub struct List<'a, T> {
    head: Link<T>,
    tailend: Option<&'a mut Node<T>>, 
 }

 type Link<T> = Option<Box<Node<T>>>;

 struct Node<T> {
    elem: T, 
    next: Link<T>,
 }

 impl<'a, T> List<'a, T> {
    pub fn new() -> Self {
        List { head: None, tailend: None }
    }

    pub fn push(&'a mut self, elem: T) {
        let new_tailend = Box::new(Node {
            elem: elem,
            // when you push onto the tail, your next is always None
            next: None,
        });
        
        // Put the Box in the right place, then grab a reference to its Node
        let new_tailend = match self.tailend.take() {
            Some(old_tailend) => {
                // if there was a non-empty tailend, update it to the new tailend
                old_tailend.next = Some(new_tailend);
                old_tailend.next.as_deref_mut()
            }
            None => {
                // otherwise, update head to point to the new tailend
                self.head = Some(new_tailend);
                self.head.as_deref_mut()
            }
        };
        self.tailend = new_tailend
    }

    pub fn pop(&'a mut self) -> Option<T> {
        // grab the lists current head
        self.head.take().map(|head| {
            let head = *head;
            self.head = head.next;
            // if we're out of `head`s, make sure the tailend also goes
            if self.head.is_none() {
                self.tailend = None;
            } 
            head.elem
        })
    }
}
 */

/* 
    Chapter 6.2 : Unsafe Rust

    The main Unsafe tool we'll be using are raw pointers. Raw pointers are basically C's pointers.
    They have no inherent aliasing rules. They have no lifetimes. They can be null. 
    They can be misaligned. They can be dangling. They can point to uninitialized memory. 
    They can be cast to and from integers. They can be cast to point to a different type. 
    Mutability? Cast it. Pretty much everything goes, and that means pretty much anything can go wrong.

    This is some bad stuff and honestly you'll live a happier life never having to touch these. 
    Unfortunately, we want to write linked lists, and linked lists are awful. 
    That means we're going to have to use unsafe pointers.

    There are two kinds of raw pointer: *const T and *mut T. 
    These are meant to be const T* and T* from C, but we really don't care about what C thinks they mean that much. 
    You can only dereference a *const T to an &T, but much like the mutability of a variable, this is just a lint against incorrect usage. 
    At most it just means you have to cast the *const to a *mut first. 
    Although if you don't actually have permission to mutate the referent of the pointer, you're gonna have a bad time.

    Anyway, we'll get a better feel for this as we write some code. 
    For now:    *mut T == &unchecked mut T
 */

/* 
    Chapter 6.3 : Basics (of unsafe Rust for our Linked List) 

    NARRATOR: This section has a looming fundamental error in it, because that's the whole point of the book. 
    However once we start using unsafe it's possible to do things wrong and still have everything compile and seemingly work. 
    The fundamental mistake will be identified in the next section. Don't actually use the contents of this section in production code!   

    It turns out that Rust is a massive rules-lawyer pedant when it comes to unsafe. 
    We quite reasonably want to maximize the set of Safe Rust programs, because those are programs we can be much more confident in. 
    To accomplish this, Rust carefully carves out a minimal surface area for unsafety. 
    Note that all the other places we've worked with raw pointers has been assigning them, or just observing whether they're null or not.

    Raw pointer dereferencing MUST be done within an `unsafe` block.

    If you never actually dereference a raw pointer those are totally safe things to do. 
    You're just reading and writing an integer! [ this makes the hardware engineers very sad :-( ]
    The only time you can actually get into trouble with a raw pointer is if you actually dereference it. 
    So Rust says only that operation is unsafe, and everything else is totally safe.

    Having only some of the pointer operations be actually unsafe raises an interesting problem: 
        | although we're supposed to delimit the scope of the unsafety with the unsafe block, 
        | it actually depends on state that was established outside of the block. 
        | Outside of the function, even!

    -> This is what I call unsafe taint. 
    As soon as you use unsafe in a module, that whole module is tainted with unsafety. 
    Everything has to be correctly written in order to make sure all invariants are upheld for the unsafe code.

    This taint is manageable because of privacy. 
    Outside of our module, all of our struct fields are totally private, so no one else can mess with our state in arbitrary ways. 
    As long as no combination of the APIs we expose causes bad stuff to happen, as far as an outside observer is concerned, all of our code is safe! 
    And really, this is no different from the FFI case. 
    No one needs to care if some python math library shells out to C as long as it exposes a safe interface.
 */

/*
    Chapter 6.4 : Miri  -  Das Good Shit

    Tests written by the author for push / pop operation using unsafe raw pointers worked like a charm. 
    However...
    We're writing unsafe code now, so the compiler can't help us catch mistakes as well. 
    It could be that the tests happened to work, but were actually doing something non-deterministic. Something Undefined Behavioury.

    But what can we do? We've pried open the windows and snuck out of rustc's classroom. No one can help us now.

    Except miri!?


    miri is an experimental interpreter for Rust's mid-level intermediate representation (MIR). 
    It can run binaries and test suites of cargo projects and detect certain classes of undefined behavior, for example:

        Out-of-bounds memory accesses and use-after-free
        Invalid use of uninitialized data
        Violation of intrinsic preconditions (an unreachable_unchecked being reached, calling copy_nonoverlapping with overlapping ranges, ...)
        Not sufficiently aligned memory accesses and references
        Violation of some basic type invariants (a bool that is not 0 or 1, for example, or an invalid enum discriminant)
        Experimental: Violations of the Stacked Borrows rules governing aliasing for reference types
        Experimental: Data races (but no weak memory effects)

    On top of that, Miri will also tell you about memory leaks: 
    when there is memory still allocated at the end of the execution, and that memory is not reachable from a global static, Miri will raise an error.

    ...

    However, be aware that Miri will not catch all cases of undefined behavior in your program, and cannot run all programs
 */

 /* 
    Chapter 6.5 : Stacked Borrows - the issue found be miri in Ch6.4 for the code from Ch6.3

    

  */


#[cfg(test)]
mod test {
    use crate::fifth::List; 

    #[test]
    fn basics() {
        let mut list = List::new();
        
        // check correct behaviour for empty list state
        assert_eq!(list.pop(), None);

        // populate the list
        list.push(1); list.push(2); list.push(3);

        // check normal removal
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), Some(2));

        // push more values to ensure nothing's corrupted
        list.push(4); list.push(5);

        // double check removal to ensure nothing's corrupted
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(4));

        // check correct behaviour upon depletion
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), None);
        assert_eq!(list.pop(), None);
    }
}
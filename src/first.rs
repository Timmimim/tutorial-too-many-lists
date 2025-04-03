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
struct Node {
    elem: i32,
    next: List,
}

pub enum List {
    Empty, 
    More(Box<Node>),
}
// Tail of list never allocates extra junk: Check!
// `enum` is in "delicious" null-pointer-optimized form: Check!
// 
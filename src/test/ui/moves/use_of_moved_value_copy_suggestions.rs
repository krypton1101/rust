// run-rustfix
#![allow(dead_code)]

fn duplicate_t<T>(t: T) -> (T, T) {
    //~^ HELP consider restricting type parameter `T`
    (t, t) //~ use of moved value: `t`
}

fn duplicate_opt<T>(t: Option<T>) -> (Option<T>, Option<T>) {
    //~^ HELP consider restricting type parameter `T`
    (t, t) //~ use of moved value: `t`
}

fn duplicate_tup1<T>(t: (T,)) -> ((T,), (T,)) {
    //~^ HELP consider restricting type parameter `T`
    (t, t) //~ use of moved value: `t`
}

fn duplicate_tup2<A, B>(t: (A, B)) -> ((A, B), (A, B)) {
    //~^ HELP consider restricting type parameters
    (t, t) //~ use of moved value: `t`
}

fn duplicate_custom<T>(t: S<T>) -> (S<T>, S<T>) {
    //~^ HELP consider restricting type parameter `T`
    (t, t) //~ use of moved value: `t`
}

struct S<T>(T);
trait Trait {}
impl<T: Trait + Clone> Clone for S<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T: Trait + Copy> Copy for S<T> {}

trait A {}
trait B {}

// Test where bounds are added with different bound placements
fn duplicate_custom_1<T>(t: S<T>) -> (S<T>, S<T>) where {
    //~^ HELP consider restricting type parameter `T`
    (t, t) //~ use of moved value: `t`
}

fn duplicate_custom_2<T>(t: S<T>) -> (S<T>, S<T>)
where
    T: A,
    //~^ HELP consider further restricting this bound
{
    (t, t) //~ use of moved value: `t`
}

fn duplicate_custom_3<T>(t: S<T>) -> (S<T>, S<T>)
where
    T: A,
    T: B,
    //~^ HELP consider further restricting type parameter `T`
{
    (t, t) //~ use of moved value: `t`
}

fn duplicate_custom_4<T: A>(t: S<T>) -> (S<T>, S<T>)
where
    T: B,
    //~^ HELP consider further restricting this bound
{
    (t, t) //~ use of moved value: `t`
}

fn main() {}
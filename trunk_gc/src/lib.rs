#[allow(dead_code, unused_variables)]

mod gc;

#[cfg(test)]
mod tests {
    use crate::gc::{Gc, GBox};

    #[derive(Debug, PartialEq)]
    pub struct Foo {
        pub i: i64,
    }

    impl Drop for Foo {
        fn drop(&mut self) {
            println!("I'm being dropped!");
        }
    }

    #[test]
    fn it_returns_a_gbox() {
        let mut gc: Gc<Foo> = Gc::new();

        assert!(matches!(gc.hold(Foo { i: 100 }), GBox { .. }));
    }

    #[test]
    fn it_can_retrieve_the_value_of_a_gbox() {
        let mut gc: Gc<Foo> = Gc::new();

        let my_gbox = gc.hold(Foo { i: 100 });

        assert_eq!(gc.get(my_gbox), Some(&Foo { i: 100 }));
    }

    #[test]
    fn it_can_return_a_mutable_ref_to_a_gbox_value() {
        let mut gc: Gc<Foo> = Gc::new();

        let my_gbox = gc.hold(Foo { i: 100 });
        (*gc.get_mut(my_gbox).unwrap()).i = 200;

        assert_eq!(gc.get(my_gbox), Some(&Foo { i: 200 }));
    }
}
use std::marker::PhantomData;

use bevy_ecs::system::{Res, SystemParam};

use crate::prelude::*;

pub struct Flush<S>(bool, PhantomData<S>);

struct ExampleState;

impl RawState for ExampleState {
    type GetNext<'w> = Res<'w, NextState_<Self>>;

    fn get_next<'a, 'w>(source: &'a Self::GetNext<'w>) -> Option<&'a Self> {
        source.get()
    }
}

// Is this version of the trait even desired? Do I even want to support computing a state
// in such a way that it doesn't actually exist anywhere in the world, but you construct it
// on demand in here?
// Mm... I think yes, but I can hold off on implementing this for now.
/*trait ComputeNext<'w>: PyriState + Sized {
    type Source: SystemParam;

    fn compute_next(source: Self::Source) -> Option<Self>;
}*/

#[derive(SystemParam)]
struct Foo;

trait HasParam: 'static {
    type Param<'w, 's>: SystemParam<Item<'w, 's> = <Self as HasParam>::Param<'w, 's>>;
}

#[derive(SystemParam)]
struct Bar<'w, 's, T: HasParam>(<T as HasParam>::Param<'w, 's>);

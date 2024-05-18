use std::{convert::Infallible, fmt::Debug, hash::Hash, marker::PhantomData};

use bevy_ecs::schedule::{ScheduleLabel, SystemSet};

#[derive(ScheduleLabel, Clone, Hash, PartialEq, Eq, Debug)]
pub struct OnTrans;

#[derive(SystemSet, Clone, PartialEq, Eq, Default)]
pub enum HandleTrans<S> {
    #[default]
    Any,
    Exit,
    Enter,
    _PhantomData(PhantomData<S>, Infallible),
}

impl<S> Hash for HandleTrans<S> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl<S> Debug for HandleTrans<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Any => write!(f, "Any"),
            Self::Exit => write!(f, "Exit"),
            Self::Enter => write!(f, "Enter"),
            _ => unreachable!(),
        }
    }
}

use crate::{type_system::Infer, SideEffects, Traverse, Type, TypeSystem};
use by_address::ByAddress;
use derive_more::From;
use enum_dispatch::enum_dispatch;
use parking_lot::Mutex;
use std::{
    cell::Cell,
    cmp::Ordering,
    fmt::{self, Display},
    hash::{Hash, Hasher},
};
use triomphe::Arc;

#[derive(Debug, Default, From, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct Local(pub Option<String>);

impl Local {
    pub fn new(name: Option<String>) -> Self {
        Self(name)
    }
}

impl fmt::Display for Local {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            Some(name) => write!(f, "{}", name),
            None => write!(f, "UNNAMED_LOCAL"),
        }
    }
}

thread_local! {
    static NEXT_ID: Cell<u64> = const { Cell::new(0) };
}

// Locals hash and sort by an id rather than their address, so map iteration order and
// sorting don't depend on where the allocator put them. Seed the ids before each unit of
// work that runs on its own thread (lifting or decompiling one function) with a value
// unique to that unit, and they come out the same whichever thread does the work.
pub fn seed_local_ids(seed: u64) {
    NEXT_ID.with(|id| id.set(seed));
}

#[derive(Debug, Clone)]
pub struct RcLocal(pub ByAddress<Arc<Mutex<Local>>>, u64);

impl Default for RcLocal {
    fn default() -> Self {
        Self::new(Local::default())
    }
}

// identity stays the allocation, clones share their id so this agrees with Hash
impl PartialEq for RcLocal {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for RcLocal {}

impl Hash for RcLocal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.1.hash(state);
    }
}

impl PartialOrd for RcLocal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RcLocal {
    fn cmp(&self, other: &Self) -> Ordering {
        self.1.cmp(&other.1).then_with(|| self.0.cmp(&other.0))
    }
}

impl Infer for RcLocal {
    fn infer<'a: 'b, 'b>(&'a mut self, system: &mut TypeSystem<'b>) -> Type {
        system.type_of(self).clone()
    }
}

impl Display for RcLocal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 .0.lock().0 {
            Some(name) => write!(f, "{}", name),
            None => write!(f, "UNNAMED_{}", self.1),
        }
    }
}

impl SideEffects for RcLocal {}

impl Traverse for RcLocal {}

impl RcLocal {
    pub fn new(local: Local) -> Self {
        let id = NEXT_ID.with(|id| {
            let next = id.get();
            id.set(next + 1);
            next
        });
        Self(ByAddress(Arc::new(Mutex::new(local))), id)
    }
}

impl LocalRw for RcLocal {
    fn values_read(&self) -> Vec<&RcLocal> {
        vec![self]
    }

    fn values_read_mut(&mut self) -> Vec<&mut RcLocal> {
        vec![self]
    }

    fn push_values_read<'a>(&'a self, out: &mut Vec<&'a RcLocal>) {
        out.push(self);
    }

    fn push_values_read_mut<'a>(&'a mut self, out: &mut Vec<&'a mut RcLocal>) {
        out.push(self);
    }
}

#[enum_dispatch]
pub trait LocalRw {
    fn values_read(&self) -> Vec<&RcLocal> {
        Vec::new()
    }

    fn values_read_mut(&mut self) -> Vec<&mut RcLocal> {
        Vec::new()
    }

    // Appends to `out` instead of returning a fresh vec, so a whole expression tree
    // fills one allocation. Types with children override these and build values_read
    // on top of them.
    fn push_values_read<'a>(&'a self, out: &mut Vec<&'a RcLocal>) {
        out.extend(self.values_read());
    }

    fn push_values_read_mut<'a>(&'a mut self, out: &mut Vec<&'a mut RcLocal>) {
        out.extend(self.values_read_mut());
    }

    fn values_written(&self) -> Vec<&RcLocal> {
        Vec::new()
    }

    fn values_written_mut(&mut self) -> Vec<&mut RcLocal> {
        Vec::new()
    }

    fn values(&self) -> Vec<&RcLocal> {
        self.values_read()
            .into_iter()
            .chain(self.values_written())
            .collect()
    }

    fn replace_values_read(&mut self, old: &RcLocal, new: &RcLocal) {
        for value in self.values_read_mut() {
            if value == old {
                *value = new.clone();
            }
        }
    }

    fn replace_values_written(&mut self, old: &RcLocal, new: &RcLocal) {
        for value in self.values_written_mut() {
            if value == old {
                *value = new.clone();
            }
        }
    }

    fn replace_values(&mut self, old: &RcLocal, new: &RcLocal) {
        self.replace_values_read(old, new);
        self.replace_values_written(old, new);
    }
}

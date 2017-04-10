use memory::alloc;
use ::core;

// TODO: make private
pub struct Pair<T> {
    head: T,
    tail: *mut LinkedList<T>
}

pub enum LinkedList<T> {
    Empty,
    List(Pair<T>)
}

fn cons<T>(head: T, tail: LinkedList<T>) -> core::result::Result<Pair<T>, (T, LinkedList<T>)> {
    match alloc::alloc_type::<LinkedList<T>>(tail) {
        Ok(tailref) => Ok(Pair::<T> { head: head, tail: tailref }),
        Err(tail) => Err((head, tail))
    }
}

impl<T> Pair<T> {
    fn split(self) -> (T, LinkedList<T>) {
        unsafe {
            let head: T = core::ptr::read(&self.head as *const T);
            let tail: *mut LinkedList<T> = core::ptr::read((&self.tail) as *const *mut LinkedList<T>);
            assert!(!tail.is_null());
            core::mem::forget(self);
            (head, alloc::dealloc_type(tail))
        }
    }

    fn head(&self) -> &T {
        &self.head
    }

    fn headmut(&mut self) -> &mut T {
        &mut self.head
    }

    fn tail(&self) -> &LinkedList<T> {
        assert!(!self.tail.is_null());
        unsafe {
            &*self.tail
        }
    }

    fn tailmut(&mut self) -> &mut LinkedList<T> {
        assert!(!self.tail.is_null());
        unsafe {
            &mut *self.tail
        }
    }

    fn view(&self) -> (&T, &LinkedList<T>) {
        (self.head(), self.tail())
    }

    fn viewmut(&mut self) -> (&mut T, &mut LinkedList<T>) {
        assert!(!self.tail.is_null());
        (&mut self.head, unsafe {
            &mut *self.tail
        })
    }
}

impl<T> Drop for Pair<T> {
    fn drop(&mut self) {
        core::mem::drop(unsafe {
            alloc::dealloc_type(self.tail)
        });
    }
}

impl<T> LinkedList<T> {
    pub const fn empty() -> LinkedList<T> {
        LinkedList::Empty
    }

    pub fn collect<I: Iterator<Item=T> + Sized>(iter: I) -> Option<LinkedList<T>> {
        let mut base = LinkedList::empty();
        {
            let mut cur_end: &mut LinkedList<T> = &mut base;
            for item in iter {
                let last_end = cur_end;
                if last_end.pushmut(item).is_err() {
                    return None;
                }
                cur_end = last_end.tailmut().unwrap();
            }
        }
        Some(base)
    }

    pub fn is_empty(&self) -> bool {
        match self {
            &LinkedList::Empty => true,
            &LinkedList::List(_) => false
        }
    }

    pub fn push(self, x: T) -> core::result::Result<LinkedList<T>, (LinkedList<T>, T)> {
        match cons(x, self) {
            Ok(pair) => {
                Ok(LinkedList::List(pair))
            }
            Err((x, nself)) => {
                Err((nself, x))
            }
        }
    }

    pub fn pushmut<'a>(&mut self, x: T) -> core::result::Result<(), T> {
        let removed_self = core::mem::replace(self, LinkedList::Empty);
        match cons(x, removed_self) {
            Ok(pair) => {
                *self = LinkedList::List(pair);
                Ok(())
            }
            Err((x, removed_self)) => {
                *self = removed_self;
                Err(x)
            }
        }
    }

    pub fn pop(self) -> Option<(T, LinkedList<T>)> {
        if let LinkedList::List(pair) = self {
            Some(pair.split())
        } else {
            None
        }
    }

    pub fn popmut(&mut self) -> Option<T> {
        let removed_self = core::mem::replace(self, LinkedList::Empty);
        if let LinkedList::List(pair) = removed_self {
            let (head, tail) = pair.split();
            *self = tail;
            Some(head)
        } else {
            *self = removed_self;
            None
        }
    }

    pub fn head(&self) -> Option<&T> {
        match self {
            &LinkedList::List(ref pair) => Some(pair.head()),
            &LinkedList::Empty => None
        }
    }

    pub fn headmut(&mut self) -> Option<&mut T> {
        match self {
            &mut LinkedList::List(ref mut pair) => Some(pair.headmut()),
            &mut LinkedList::Empty => None
        }
    }

    pub fn tail(&self) -> Option<&LinkedList<T>> {
        match self {
            &LinkedList::List(ref pair) => Some(pair.tail()),
            &LinkedList::Empty => None
        }
    }

    pub fn tailmut<'a>(&'a mut self) -> Option<&'a mut LinkedList<T>> {
        match self {
            &mut LinkedList::List(ref mut pair) => Some(pair.tailmut()),
            &mut LinkedList::Empty => None
        }
    }

    pub fn next(&self) -> Option<(&T, &LinkedList<T>)> {
        match self {
            &LinkedList::List(ref pair) => Some(pair.view()),
            &LinkedList::Empty => None
        }
    }

    pub fn nextmut<'a>(&'a mut self) -> Option<(&'a mut T, &'a mut LinkedList<T>)> {
        match self {
            &mut LinkedList::List(ref mut pair) => Some(pair.viewmut()),
            &mut LinkedList::Empty => None
        }
    }

    pub fn get(&self, i: usize) -> Option<&T> {
        let mut cur: &LinkedList<T> = &self;
        for ii in 0..i {
            if let Some(ncur) = cur.tail() {
                cur = ncur;
            } else {
                return None;
            }
        }
        cur.head()
    }

    pub fn get_mut<'a>(&'a mut self, i: usize) -> Option<&'a mut T> {
        let mut cur: &mut LinkedList<T> = self;
        for ii in 0..i {
            let tmp = cur;
            if let Some(ncur) = tmp.tailmut() {
                cur = ncur;
            } else {
                return None;
            }
        }
        cur.headmut()
    }

    pub fn len(&self) -> usize {
        let mut n = 0;
        let mut cur = self;
        while let Some(pair) = cur.tail() {
            cur = pair;
            n = n + 1;
        }
        n
    }

    pub fn remove_mut<P>(&mut self, predicate: P) -> Option<T> where P: Fn(&T) -> bool {
        let mut cur: &mut LinkedList<T> = self;
        loop {
            let tmp = cur;
            if tmp.is_empty() {
                return None;
            }
            if predicate(tmp.head().unwrap()) {
                return Some(tmp.popmut().unwrap());
            }
            cur = tmp.tailmut().unwrap();
        }
    }

    pub fn find_mut<P>(&mut self, predicate: P) -> Option<&mut T> where P: Fn(&T) -> bool {
        let mut cur: &mut LinkedList<T> = self;
        while true {
            let tmp = cur;
            let ncur = match tmp {
                &mut LinkedList::List(ref mut pair) => {
                    if predicate(pair.head()) {
                        return Some(pair.headmut());
                    }
                    pair.tailmut()
                }
                &mut LinkedList::Empty => {
                    break;
                }
            };
            cur = ncur;
        }
        None
    }

    pub fn find<P>(&self, predicate: P) -> Option<&T> where P: Fn(&T) -> bool {
        let mut cur: &LinkedList<T> = self;
        while true {
            let ncur = match cur {
                &LinkedList::List(ref pair) => {
                    if predicate(pair.head()) {
                        return Some(pair.head());
                    }
                    pair.tail()
                }
                &LinkedList::Empty => {
                    break;
                }
            };
            cur = ncur;
        }
        None
    }

    pub fn find_and_index<P>(&self, predicate: P) -> Option<(usize, &T)> where P: Fn(&T) -> bool {
        let mut cur: &LinkedList<T> = self;
        let mut i: usize = 0;
        loop {
            let ncur = match cur {
                &LinkedList::List(ref pair) => {
                    if predicate(pair.head()) {
                        return Some((i, pair.head()))
                    }
                    pair.tail()
                }
                &LinkedList::Empty => {
                    return None
                }
            };
            cur = ncur;
            i += 1;
        }
    }

    pub fn find_index<P>(&self, predicate: P) -> Option<usize> where P: Fn(&T) -> bool {
        let mut cur: &LinkedList<T> = self;
        let mut i: usize = 0;
        loop {
            let ncur = match cur {
                &LinkedList::List(ref pair) => {
                    if predicate(pair.head()) {
                        return Some(i)
                    }
                    pair.tail()
                }
                &LinkedList::Empty => {
                    return None
                }
            };
            cur = ncur;
            i += 1;
        }
    }
}

pub struct LinkedIter<'a, T: 'a> {
    current: &'a LinkedList<T>
}

impl<'a, T> Iterator for LinkedIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        match self.current.next() {
            Some((head, tail)) => {
                self.current = tail;
                Some(head)
            }
            None => None
        }
    }
}

impl<'a, T> IntoIterator for &'a LinkedList<T> {
    type Item = &'a T;
    type IntoIter = LinkedIter<'a, T>;
    fn into_iter(self) -> LinkedIter<'a, T> {
        LinkedIter::<T> { current: &self }
    }
}

impl<'a, T: core::fmt::Display> core::fmt::Display for &'a LinkedList<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        if let Some((first, rest)) = self.next() {
            write!(f, "[")?;
            write!(f, "{}", first)?;
            for item in rest {
                write!(f, ", {}", item)?;
            }
            write!(f, "]")
        } else {
            write!(f, "[]")
        }
    }
}

impl<'a, T: core::fmt::Debug> core::fmt::Debug for &'a LinkedList<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        if let Some((first, rest)) = self.next() {
            write!(f, "[")?;
            write!(f, "{:?}", first)?;
            for item in rest {
                write!(f, ", {:?}", item)?;
            }
            write!(f, "]")
        } else {
            write!(f, "[]")
        }
    }
}

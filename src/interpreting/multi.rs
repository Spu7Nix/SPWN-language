use super::context::Context;

#[derive(Debug, Clone, PartialEq)]
pub struct Multi<T> {
    vec: Vec<(Context, T)>,
}

impl<T> IntoIterator for Multi<T> {
    type IntoIter = std::vec::IntoIter<(Context, T)>;
    type Item = (Context, T);

    fn into_iter(self) -> Self::IntoIter {
        self.vec.into_iter()
    }
}

impl<T> FromIterator<(Context, T)> for Multi<T> {
    fn from_iter<I: IntoIterator<Item = (Context, T)>>(iter: I) -> Self {
        Self {
            vec: iter.into_iter().collect(),
        }
    }
}

impl<T> Multi<T> {
    pub fn new_single(ctx: Context, v: T) -> Self {
        Self {
            vec: vec![(ctx, v)],
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Context, &T)> {
        self.vec.iter().map(|(v, t)| (v, t))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&mut Context, &mut T)> {
        self.vec.iter_mut().map(|(v, t)| (v, t))
    }

    pub fn into_iter(self) -> impl Iterator<Item = (Context, T)> {
        self.vec.into_iter()
    }

    pub fn map<F, R>(self, mut f: F) -> Multi<R>
    where
        F: FnMut(Context, T) -> (Context, R),
    {
        self.into_iter().map(|(c, v)| f(c, v)).collect()
    }

    // pub fn try_map<F, R, E>(self, mut f: F) -> Result<Multi<R>, E>
    // where
    //     F: FnMut(Context, T) -> Result<(Context, R), E>,
    // {
    //     self.into_iter().map(|(c, v)| f(c, v)).collect()
    // }

    pub fn flat_map<F, R>(self, mut f: F) -> Multi<R>
    where
        F: FnMut(Context, T) -> Multi<R>,
    {
        self.into_iter().flat_map(|(c, v)| f(c, v)).collect()
    }

    // pub fn try_flat_map<F, R, E>(self, mut f: F) -> Result<Multi<R>, E>
    // where
    //     F: FnMut(Context, T) -> Result<Multi<R>, E>,
    // {
    //     let mut vec = vec![];

    //     for (c, v) in self {
    //         for (c, v) in f(c, v)? {
    //             vec.push((c, v))
    //         }
    //     }

    //     Ok(Multi { vec })
    // }
}

impl<T> Multi<Multi<T>> {
    pub fn flatten(self) -> Multi<T> {
        Multi {
            vec: self
                .vec
                .into_iter()
                .flat_map(|(_, v)| v.vec.into_iter())
                .collect(),
        }
    }
}

impl<T, E> Multi<Result<T, E>> {
    pub fn try_map<F, R>(self, mut f: F) -> Multi<Result<R, E>>
    where
        F: FnMut(Context, T) -> (Context, Result<R, E>),
    {
        self.map(|ctx, v| match v {
            Ok(v) => {
                let (ctx, out) = f(ctx, v);
                (ctx, out)
            },
            Err(err) => (ctx, Err(err)),
        })
    }

    pub fn try_flat_map<F, R>(self, mut f: F) -> Multi<Result<R, E>>
    where
        F: FnMut(Context, T) -> Multi<Result<R, E>>,
    {
        let mut out = Multi { vec: vec![] };

        for (ctx, v) in self {
            match v {
                Ok(v) => {
                    for (ctx, v) in f(ctx, v) {
                        out.vec.push((ctx, v))
                    }
                },
                Err(err) => out.vec.push((ctx, Err(err))),
            }
        }

        out
    }
}

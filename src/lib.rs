//! Composable logging with monoids and contravariant functors.
//!
//! A logger is a routine that takes input and has side-effects.
//! Any routine that has the appropriate type will do.
//! A logger can be seen as the opposite or dual of an infinite iterator.
//!
//! The core trait of this crate is [Logger].
//! It has only a single method that must be implemented: log.
//! To log something, pass it to this method.
//! It is up to the logger to decide what to do with the value.
//!
//! Loggers are composable:
//! given two loggers with compatible types,
//! a new logger can be created that forwards
//! its input to both loggers.
//!
//! Loggers can also be transformed using
//! methods such as [map] and [filter].
//!
//! [Logger]: trait.Logger.html
//! [map]: trait.Logger.html#method.map
//! [filter]: trait.Logger.html#method.filter

use std::convert::Infallible;
use std::iter;
use std::marker::PhantomData;

/// A logger is a routine that takes input and has side-effects.
///
/// See the documentation for the [contralog](index.html) crate
/// for a general overview of loggers.
pub trait Logger<I>
{
    type Error;
    fn log(&mut self, item: I) -> Result<(), Self::Error>;

    /// Create a “by reference” adaptor for this logger.
    fn by_ref(&mut self) -> &mut Self
    {
        self
    }

    /// Combine two loggers, creating a new logger
    /// that logs each input to both loggers.
    fn chain<L>(self, other: L) -> Chain<Self, L>
        where Self: Sized
    {
        Chain{fst: self, snd: other}
    }

    /// Apply a function to each input and
    /// pass it to the logger
    /// only if the function returns true for it.
    fn filter<F>(self, f: F) -> Filter<Self, F>
        where Self: Sized, F: FnMut(&I) -> bool
    {
        Filter{inner: self, f}
    }

    /// Apply a function to each input
    /// before passing it to the logger.
    fn map<F, B>(self, f: F) -> Map<Self, F, B>
        where Self: Sized
    {
        Map{inner: self, f, _phantom: PhantomData}
    }

    /// Return a logger that
    /// silently drops errors reported by this logger.
    fn safe<E>(self) -> Safe<Self, E>
        where Self: Sized
    {
        Safe{inner: self, _phantom: PhantomData}
    }
}

impl<'a, I, L> Logger<I> for &'a mut L
    where L: Logger<I>
{
    type Error = L::Error;
    fn log(&mut self, item: I) -> Result<(), Self::Error>
    {
        (**self).log(item)
    }
}

/// Returned from the [Logger::chain](trait.Logger.html#method.chain) method.
pub struct Chain<L, M>
{
    fst: L,
    snd: M,
}

impl<I, L, M> Logger<I> for Chain<L, M>
    where L: Logger<I>, M: Logger<I, Error=L::Error>, I: Clone
{
    type Error = L::Error;
    fn log(&mut self, item: I) -> Result<(), Self::Error>
    {
        self.fst.log(item.clone())?;
        self.snd.log(item)
    }
}

/// A logger that ignores all input.
pub fn empty<I, E>() -> Empty<I, E>
{
    Empty{_phantom: PhantomData}
}

/// Returned from the [empty](fn.empty.html) function.
pub struct Empty<I, E>
{
    _phantom: PhantomData<fn() -> (I, E)>,
}

impl<I, E> Logger<I> for Empty<I, E>
{
    type Error = E;
    fn log(&mut self, _item: I) -> Result<(), E>
    {
        Ok(())
    }
}

/// A logger that collects values into a container.
pub fn extender<C, I>(container: C) -> Extender<C, I>
{
    Extender{container, _phantom: PhantomData}
}

/// Returned from the [extender](fn.extender.html) function.
pub struct Extender<C, I>
{
    pub container: C,
    _phantom: PhantomData<fn() -> I>,
}

impl<C, I> Logger<I> for Extender<C, I>
    where C: Extend<I>
{
    type Error = Infallible;
    fn log(&mut self, item: I) -> Result<(), Self::Error>
    {
        let from = iter::once(item);
        self.container.extend(from);
        Ok(())
    }
}

/// Returned from the [Logger::filter](trait.Logger.html#method.filter) method.
pub struct Filter<L, F>
{
    inner: L,
    f: F,
}

impl<I, L, F> Logger<I> for Filter<L, F>
    where L: Logger<I>, F: FnMut(&I) -> bool
{
    type Error = L::Error;
    fn log(&mut self, item: I) -> Result<(), Self::Error>
    {
        if (self.f)(&item) {
            self.inner.log(item)
        } else {
            Ok(())
        }
    }
}

/// Returned from the [Logger::map](trait.Logger.html#method.map) method.
pub struct Map<L, F, B>
{
    inner: L,
    f: F,
    _phantom: PhantomData<fn() -> B>,
}

impl<I, L, F, B> Logger<B> for Map<L, F, B>
    where L: Logger<I>, F: FnMut(B) -> I
{
    type Error = L::Error;
    fn log(&mut self, item: B) -> Result<(), Self::Error>
    {
        let new_item = (self.f)(item);
        self.inner.log(new_item)
    }
}

/// Returned from the [Logger::safe](trait.Logger.html#method.safe) method.
pub struct Safe<L, E>
{
    inner: L,
    _phantom: PhantomData<fn() -> E>,
}

impl<I, L, E> Logger<I> for Safe<L, E>
    where L: Logger<I>
{
    type Error = E;
    fn log(&mut self, item: I) -> Result<(), Self::Error>
    {
        let result = self.inner.log(item);
        drop(result);
        Ok(())
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_chain()
    {
        let mut fst = extender(Vec::new());
        let mut snd = extender(Vec::new());
        let mut thd = fst.by_ref().chain(&mut snd);
        thd.log(0).unwrap();
        assert_eq!(&fst.container, &[0]);
        assert_eq!(&snd.container, &[0]);
    }

    #[test]
    fn test_filter()
    {
        let mut fst = extender(Vec::new());
        let mut snd = fst.by_ref().filter(|&i| i >= 0);
        snd.log(-1).unwrap();
        snd.log(0).unwrap();
        snd.log(1).unwrap();
        assert_eq!(&fst.container, &[0, 1]);
    }

    #[test]
    fn test_map()
    {
        let mut fst = extender(Vec::new());
        let mut snd = fst.by_ref().map(|i: i32| i.abs());
        snd.log(-1).unwrap();
        snd.log(0).unwrap();
        snd.log(1).unwrap();
        assert_eq!(&fst.container, &[1, 0, 1]);
    }
}
